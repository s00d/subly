import type { Subscription, Expense, AppData } from "@/schemas/appData";
import { validateImportData } from "@/schemas/appData";
import { save, open } from "@tauri-apps/plugin-dialog";
import { writeTextFile, readTextFile, writeFile, readFile } from "@tauri-apps/plugin-fs";
import JSZip from "jszip";

// =============================================
// Helpers
// =============================================

/**
 * Check if an icon string is a base64-encoded custom image (data: URI).
 */
function isBase64Icon(icon: string): boolean {
  return icon.startsWith("data:");
}

/**
 * Collect all unique custom (base64) icons from app data.
 * Returns a Map: generatedFilename -> base64DataURI.
 */
function collectCustomIcons(data: AppData): Map<string, string> {
  const icons = new Map<string, string>();
  let idx = 0;

  function addIcon(dataUri: string): string {
    // Check if we already stored this icon
    for (const [filename, existing] of icons) {
      if (existing === dataUri) return filename;
    }
    // Determine extension from data URI
    const mimeMatch = dataUri.match(/^data:image\/(\w+)/);
    const ext = mimeMatch ? mimeMatch[1].replace("jpeg", "jpg") : "png";
    const filename = `icon_${idx++}.${ext}`;
    icons.set(filename, dataUri);
    return filename;
  }

  // Subscriptions logos
  for (const sub of data.subscriptions) {
    if (sub.logo && isBase64Icon(sub.logo)) {
      addIcon(sub.logo);
    }
  }

  // Payment method icons
  for (const pm of data.paymentMethods) {
    if (pm.icon && isBase64Icon(pm.icon)) {
      addIcon(pm.icon);
    }
  }

  return icons;
}

/**
 * Replace base64 icons in data with archive-relative paths (icons/filename).
 * Returns cloned data with references replaced.
 */
function replaceIconsWithPaths(data: AppData, iconsMap: Map<string, string>): AppData {
  // Build reverse lookup: base64 -> filename
  const reverse = new Map<string, string>();
  for (const [filename, dataUri] of iconsMap) {
    reverse.set(dataUri, `icons/${filename}`);
  }

  const clone: AppData = JSON.parse(JSON.stringify(data));

  for (const sub of clone.subscriptions) {
    if (sub.logo && reverse.has(sub.logo)) {
      sub.logo = reverse.get(sub.logo)!;
    }
  }

  for (const pm of clone.paymentMethods) {
    if (pm.icon && reverse.has(pm.icon)) {
      pm.icon = reverse.get(pm.icon)!;
    }
  }

  return clone;
}

/**
 * Restore base64 icons from archive paths back to inline data URIs.
 */
function restoreIconsFromArchive(data: AppData, iconsMap: Map<string, string>): AppData {
  for (const sub of data.subscriptions) {
    if (sub.logo && iconsMap.has(sub.logo)) {
      sub.logo = iconsMap.get(sub.logo)!;
    }
  }

  for (const pm of data.paymentMethods) {
    if (pm.icon && iconsMap.has(pm.icon)) {
      pm.icon = iconsMap.get(pm.icon)!;
    }
  }

  return data;
}

/**
 * Convert a base64 data URI to a Uint8Array.
 */
function dataUriToUint8Array(dataUri: string): Uint8Array {
  const base64 = dataUri.split(",")[1] || "";
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

/**
 * Convert a Uint8Array to a base64 data URI.
 */
function uint8ArrayToDataUri(bytes: Uint8Array, filename: string): string {
  const ext = filename.split(".").pop()?.toLowerCase() || "png";
  const mimeMap: Record<string, string> = {
    png: "image/png",
    jpg: "image/jpeg",
    jpeg: "image/jpeg",
    svg: "image/svg+xml",
    webp: "image/webp",
    gif: "image/gif",
  };
  const mime = mimeMap[ext] || "image/png";
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return `data:${mime};base64,${btoa(binary)}`;
}

// =============================================
// Export — .subly archive
// =============================================

/**
 * Export all app data as a .subly archive (ZIP with custom extension).
 * Contains: data.json + icons/ folder with all custom images.
 */
export async function exportAsSubly(data: AppData): Promise<boolean> {
  const zip = new JSZip();

  // Collect custom icons
  const iconsMap = collectCustomIcons(data);

  // Replace base64 in data with archive paths
  const cleanData = replaceIconsWithPaths(data, iconsMap);

  // Add data.json
  zip.file("data.json", JSON.stringify(cleanData, null, 2));

  // Add icons
  for (const [filename, dataUri] of iconsMap) {
    const bytes = dataUriToUint8Array(dataUri);
    zip.file(`icons/${filename}`, bytes);
  }

  // Add metadata
  zip.file("meta.json", JSON.stringify({
    version: 1,
    app: "subly",
    exportedAt: new Date().toISOString(),
    iconCount: iconsMap.size,
  }, null, 2));

  // Generate archive
  const blob = await zip.generateAsync({ type: "uint8array", compression: "DEFLATE", compressionOptions: { level: 6 } });

  const filePath = await save({
    filters: [{ name: "Subly Archive", extensions: ["subly"] }],
    defaultPath: `subly-backup-${new Date().toISOString().split("T")[0]}.subly`,
  });

  if (filePath) {
    await writeFile(filePath, blob);
    return true;
  }
  return false;
}

// =============================================
// Import — .subly archive
// =============================================

/**
 * Import data from a .subly archive.
 * Restores data.json + all custom icons as inline base64.
 */
export async function importFromSubly(): Promise<AppData | null> {
  const selected = await open({
    filters: [{ name: "Subly Archive", extensions: ["subly"] }],
    multiple: false,
  });
  if (!selected) return null;

  try {
    const bytes = await readFile(selected as string);
    const zip = await JSZip.loadAsync(bytes);

    // Read data.json
    const dataFile = zip.file("data.json");
    if (!dataFile) {
      console.warn("Archive missing data.json");
      return null;
    }
    const dataJson = await dataFile.async("string");
    const raw = JSON.parse(dataJson);
    const data = validateImportData(raw);
    if (!data) return null;

    // Read icons and restore inline
    const iconsMap = new Map<string, string>();
    const iconFiles = zip.folder("icons");
    if (iconFiles) {
      const promises: Promise<void>[] = [];
      iconFiles.forEach((relativePath, file) => {
        if (file.dir) return;
        promises.push(
          file.async("uint8array").then((bytes) => {
            const dataUri = uint8ArrayToDataUri(bytes, relativePath);
            iconsMap.set(`icons/${relativePath}`, dataUri);
          })
        );
      });
      await Promise.all(promises);
    }

    // Restore icon references
    restoreIconsFromArchive(data, iconsMap);

    return data;
  } catch (e) {
    console.error("Failed to import .subly archive:", e);
    return null;
  }
}

// =============================================
// Legacy exports (JSON, CSV) — kept for flexibility
// =============================================

export async function exportAsJson(data: AppData): Promise<boolean> {
  const json = JSON.stringify(data, null, 2);
  const filePath = await save({
    filters: [{ name: "JSON", extensions: ["json"] }],
    defaultPath: "subly-export.json",
  });
  if (filePath) {
    await writeTextFile(filePath, json);
    return true;
  }
  return false;
}

export async function exportAsCsv(
  subscriptions: Subscription[],
  categories: { id: string; name: string }[],
  currencies: { id: string; code: string }[],
  paymentMethods: { id: string; name: string }[],
  household: { id: string; name: string }[],
): Promise<boolean> {
  const headers = [
    "Name", "Price", "Currency", "Cycle", "Frequency", "Next Payment",
    "Start Date", "Auto Renew", "Category", "Payment Method", "Paid By",
    "Notes", "URL", "Inactive",
  ];

  const cycleNames: Record<number, string> = { 1: "Daily", 2: "Weekly", 3: "Monthly", 4: "Yearly" };

  const rows = subscriptions.map((s) => [
    `"${s.name.replace(/"/g, '""')}"`,
    s.price.toString(),
    currencies.find((c) => c.id === s.currencyId)?.code || "",
    cycleNames[s.cycle] || "",
    s.frequency.toString(),
    s.nextPayment,
    s.startDate,
    s.autoRenew ? "Yes" : "No",
    categories.find((c) => c.id === s.categoryId)?.name || "",
    paymentMethods.find((p) => p.id === s.paymentMethodId)?.name || "",
    household.find((h) => h.id === s.payerUserId)?.name || "",
    `"${(s.notes || "").replace(/"/g, '""')}"`,
    s.url || "",
    s.inactive ? "Yes" : "No",
  ]);

  const csv = [headers.join(","), ...rows.map((r) => r.join(","))].join("\n");

  const filePath = await save({
    filters: [{ name: "CSV", extensions: ["csv"] }],
    defaultPath: "subly-export.csv",
  });
  if (filePath) {
    await writeTextFile(filePath, csv);
    return true;
  }
  return false;
}

/**
 * Export expenses as CSV.
 */
export async function exportExpensesCsv(
  expenses: Expense[],
  categories: { id: string; name: string }[],
  currencies: { id: string; code: string }[],
  paymentMethods: { id: string; name: string }[],
  household: { id: string; name: string }[],
): Promise<boolean> {
  const headers = ["Name", "Amount", "Currency", "Date", "Category", "Payment Method", "Paid By", "Tags", "Notes"];

  const rows = expenses.map((e) => [
    `"${e.name.replace(/"/g, '""')}"`,
    e.amount.toString(),
    currencies.find((c) => c.id === e.currencyId)?.code || "",
    e.date,
    categories.find((c) => c.id === e.categoryId)?.name || "",
    paymentMethods.find((p) => p.id === e.paymentMethodId)?.name || "",
    household.find((h) => h.id === e.payerUserId)?.name || "",
    `"${e.tags.join(", ")}"`,
    `"${(e.notes || "").replace(/"/g, '""')}"`,
  ]);

  const csv = [headers.join(","), ...rows.map((r) => r.join(","))].join("\n");

  const filePath = await save({
    filters: [{ name: "CSV", extensions: ["csv"] }],
    defaultPath: "subly-expenses.csv",
  });
  if (filePath) {
    await writeTextFile(filePath, csv);
    return true;
  }
  return false;
}

// =============================================
// Import — CSV
// =============================================

/**
 * Parse a CSV line respecting quoted fields.
 */
function parseCsvLine(line: string): string[] {
  const fields: string[] = [];
  let current = "";
  let inQuotes = false;
  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (inQuotes) {
      if (ch === '"') {
        if (i + 1 < line.length && line[i + 1] === '"') {
          current += '"';
          i++;
        } else {
          inQuotes = false;
        }
      } else {
        current += ch;
      }
    } else {
      if (ch === '"') {
        inQuotes = true;
      } else if (ch === ",") {
        fields.push(current.trim());
        current = "";
      } else {
        current += ch;
      }
    }
  }
  fields.push(current.trim());
  return fields;
}

export interface CsvImportContext {
  categories: { id: string; name: string }[];
  currencies: { id: string; code: string }[];
  paymentMethods: { id: string; name: string }[];
  household: { id: string; name: string }[];
  defaultCategoryId: string;
  defaultCurrencyId: string;
  defaultPaymentMethodId: string;
  defaultPayerUserId: string;
}

/**
 * Import subscriptions from a CSV file.
 * Expected columns (header names, case-insensitive):
 *   Name, Price, Currency, Cycle, Frequency, Next Payment,
 *   Start Date, Auto Renew, Category, Payment Method, Paid By,
 *   Notes, URL, Inactive
 *
 * Returns an array of parsed Subscription objects (without IDs — caller assigns them).
 */
export async function importFromCsv(
  ctx: CsvImportContext,
): Promise<Subscription[] | null> {
  const selected = await open({
    filters: [{ name: "CSV", extensions: ["csv"] }],
    multiple: false,
  });
  if (!selected) return null;

  try {
    const content = await readTextFile(selected as string);
    const lines = content.split(/\r?\n/).filter((l) => l.trim());
    if (lines.length < 2) return null; // need header + at least 1 row

    const headerLine = lines[0];
    const headers = parseCsvLine(headerLine).map((h) => h.toLowerCase().replace(/[^a-z0-9_]/g, "_"));

    // Build column index map
    const col = (name: string): number => {
      const variants = [name, name.replace(/ /g, "_")];
      for (const v of variants) {
        const idx = headers.indexOf(v);
        if (idx >= 0) return idx;
      }
      return -1;
    };

    const iName = col("name");
    const iPrice = col("price");
    const iCurrency = col("currency");
    const iCycle = col("cycle");
    const iFrequency = col("frequency");
    const iNextPayment = col("next_payment");
    const iStartDate = col("start_date");
    const iAutoRenew = col("auto_renew");
    const iCategory = col("category");
    const iPaymentMethod = col("payment_method");
    const iPaidBy = col("paid_by");
    const iNotes = col("notes");
    const iUrl = col("url");
    const iInactive = col("inactive");

    if (iName < 0) return null; // Name is required

    const cycleMap: Record<string, number> = {
      daily: 1, weekly: 2, monthly: 3, yearly: 4,
    };

    const subscriptions: Subscription[] = [];

    for (let i = 1; i < lines.length; i++) {
      const fields = parseCsvLine(lines[i]);
      if (fields.length === 0 || (fields.length === 1 && !fields[0])) continue;

      const name = iName >= 0 ? fields[iName] : "";
      if (!name) continue; // skip empty rows

      const priceStr = iPrice >= 0 ? fields[iPrice] : "0";
      const price = parseFloat(priceStr) || 0;

      const currencyCode = iCurrency >= 0 ? fields[iCurrency] : "";
      const currencyMatch = ctx.currencies.find(
        (c) => c.code.toLowerCase() === currencyCode.toLowerCase(),
      );
      const currencyId = currencyMatch?.id || ctx.defaultCurrencyId;

      const cycleStr = iCycle >= 0 ? fields[iCycle]?.toLowerCase() : "monthly";
      const cycle = cycleMap[cycleStr || "monthly"] || 3;

      const freqStr = iFrequency >= 0 ? fields[iFrequency] : "1";
      const frequency = parseInt(freqStr, 10) || 1;

      const nextPayment = iNextPayment >= 0 ? normalizeDate(fields[iNextPayment]) : new Date().toISOString().split("T")[0];
      const startDate = iStartDate >= 0 ? normalizeDate(fields[iStartDate]) : nextPayment;

      const autoRenewStr = iAutoRenew >= 0 ? fields[iAutoRenew]?.toLowerCase() : "yes";
      const autoRenew = autoRenewStr !== "no" && autoRenewStr !== "false" && autoRenewStr !== "0";

      const categoryName = iCategory >= 0 ? fields[iCategory] : "";
      const categoryMatch = ctx.categories.find(
        (c) => c.name.toLowerCase() === categoryName.toLowerCase(),
      );
      const categoryId = categoryMatch?.id || ctx.defaultCategoryId;

      const pmName = iPaymentMethod >= 0 ? fields[iPaymentMethod] : "";
      const pmMatch = ctx.paymentMethods.find(
        (p) => p.name.toLowerCase() === pmName.toLowerCase(),
      );
      const paymentMethodId = pmMatch?.id || ctx.defaultPaymentMethodId;

      const payerName = iPaidBy >= 0 ? fields[iPaidBy] : "";
      const payerMatch = ctx.household.find(
        (h) => h.name.toLowerCase() === payerName.toLowerCase(),
      );
      const payerUserId = payerMatch?.id || ctx.defaultPayerUserId;

      const notes = iNotes >= 0 ? fields[iNotes] || "" : "";
      const url = iUrl >= 0 ? fields[iUrl] || "" : "";

      const inactiveStr = iInactive >= 0 ? fields[iInactive]?.toLowerCase() : "no";
      const inactive = inactiveStr === "yes" || inactiveStr === "true" || inactiveStr === "1";

      subscriptions.push({
        id: crypto.randomUUID(),
        name,
        logo: "",
        price,
        currencyId,
        nextPayment,
        startDate,
        cycle: cycle as 1 | 2 | 3 | 4,
        frequency,
        notes,
        paymentMethodId,
        payerUserId,
        categoryId,
        notify: false,
        notifyDaysBefore: -1,
        lastNotifiedDate: "",
        inactive,
        autoRenew,
        url,
        cancellationDate: null,
        replacementSubscriptionId: null,
        createdAt: new Date().toISOString(),
        tags: [],
        favorite: false,
        paymentHistory: [],
      });
    }

    return subscriptions.length > 0 ? subscriptions : null;
  } catch (e) {
    console.error("Failed to import CSV:", e);
    return null;
  }
}

/**
 * Normalize a date string to YYYY-MM-DD.
 * Handles common formats: YYYY-MM-DD, MM/DD/YYYY, DD.MM.YYYY, etc.
 */
function normalizeDate(str: string): string {
  if (!str) return new Date().toISOString().split("T")[0];

  // Try ISO format first
  if (/^\d{4}-\d{2}-\d{2}/.test(str)) {
    return str.split("T")[0];
  }

  // MM/DD/YYYY
  const usMatch = str.match(/^(\d{1,2})\/(\d{1,2})\/(\d{4})$/);
  if (usMatch) {
    return `${usMatch[3]}-${usMatch[1].padStart(2, "0")}-${usMatch[2].padStart(2, "0")}`;
  }

  // DD.MM.YYYY
  const euMatch = str.match(/^(\d{1,2})\.(\d{1,2})\.(\d{4})$/);
  if (euMatch) {
    return `${euMatch[3]}-${euMatch[2].padStart(2, "0")}-${euMatch[1].padStart(2, "0")}`;
  }

  // Fallback: try Date constructor
  const parsed = new Date(str);
  if (!isNaN(parsed.getTime())) {
    return parsed.toISOString().split("T")[0];
  }

  return new Date().toISOString().split("T")[0];
}

// =============================================
// Import — Legacy JSON
// =============================================

/**
 * Import from legacy JSON format.
 */
export async function importFromJson(): Promise<AppData | null> {
  const selected = await open({
    filters: [{ name: "JSON", extensions: ["json"] }],
    multiple: false,
  });
  if (!selected) return null;

  try {
    const content = await readTextFile(selected as string);
    const raw = JSON.parse(content);
    return validateImportData(raw);
  } catch {
    return null;
  }
}
