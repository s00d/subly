import type { Subscription, Expense, AppData } from "@/schemas/appData";
import { validateImportData } from "@/schemas/appData";
import { save, open } from "@tauri-apps/plugin-dialog";
import { writeTextFile, readTextFile, writeFile, readFile } from "@tauri-apps/plugin-fs";
import JSZip from "jszip";
import Papa from "papaparse";

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
  const cycleNames: Record<number, string> = { 1: "Daily", 2: "Weekly", 3: "Monthly", 4: "Yearly" };
  const rows = subscriptions.map((s) => ({
    Name: s.name,
    Price: s.price.toString(),
    Currency: currencies.find((c) => c.id === s.currencyId)?.code || "",
    Cycle: cycleNames[s.cycle] || "",
    Frequency: s.frequency.toString(),
    "Next Payment": s.nextPayment,
    "Start Date": s.startDate,
    "Auto Renew": s.autoRenew ? "Yes" : "No",
    Category: categories.find((c) => c.id === s.categoryId)?.name || "",
    "Payment Method": paymentMethods.find((p) => p.id === s.paymentMethodId)?.name || "",
    "Paid By": household.find((h) => h.id === s.payerUserId)?.name || "",
    Notes: s.notes || "",
    URL: s.url || "",
    Inactive: s.inactive ? "Yes" : "No",
  }));
  const csv = Papa.unparse(rows, { escapeFormulae: true });

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
  const rows = expenses.map((e) => ({
    Name: e.name,
    Amount: e.amount.toString(),
    Currency: currencies.find((c) => c.id === e.currencyId)?.code || "",
    Date: e.date,
    Category: categories.find((c) => c.id === e.categoryId)?.name || "",
    "Payment Method": paymentMethods.find((p) => p.id === e.paymentMethodId)?.name || "",
    "Paid By": household.find((h) => h.id === e.payerUserId)?.name || "",
    Tags: e.tags.join(", "),
    Notes: e.notes || "",
  }));
  const csv = Papa.unparse(rows, { escapeFormulae: true });

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
    const content = (await readTextFile(selected as string)).replace(/^\uFEFF/, "");
    const parsed = Papa.parse<Record<string, string>>(content, {
      header: true,
      skipEmptyLines: true,
      transformHeader: (h: string) => h.toLowerCase().replace(/[^a-z0-9_]/g, "_"),
    });
    if (parsed.errors.length > 0) {
      console.warn("CSV parse errors:", parsed.errors);
    }
    const rows = parsed.data.filter((r: Record<string, string>) =>
      Object.values(r).some((v) => String(v ?? "").trim() !== ""),
    );
    if (rows.length === 0) return [];

    const getField = (row: Record<string, string>, ...keys: string[]) => {
      for (const key of keys) {
        if (row[key] != null) return row[key] ?? "";
      }
      return "";
    };

    const cycleMap: Record<string, number> = {
      daily: 1, weekly: 2, monthly: 3, yearly: 4,
    };

    const subscriptions: Subscription[] = [];

    for (const row of rows) {
      const name = getField(row, "name").trim();
      if (!name) continue; // skip empty rows

      const priceStr = getField(row, "price");
      const price = parseFloat(priceStr) || 0;

      const currencyCode = getField(row, "currency");
      const currencyMatch = ctx.currencies.find(
        (c) => c.code.toLowerCase() === currencyCode.toLowerCase(),
      );
      const currencyId = currencyMatch?.id || ctx.defaultCurrencyId;

      const cycleStr = getField(row, "cycle")?.toLowerCase() || "monthly";
      const cycle = cycleMap[cycleStr || "monthly"] || 3;

      const freqStr = getField(row, "frequency") || "1";
      const frequency = parseInt(freqStr, 10) || 1;

      const nextPayment = normalizeDate(getField(row, "next_payment", "next_payment_date"));
      const startDate = normalizeDate(getField(row, "start_date")) || nextPayment;

      const autoRenewStr = getField(row, "auto_renew")?.toLowerCase() || "yes";
      const autoRenew = autoRenewStr !== "no" && autoRenewStr !== "false" && autoRenewStr !== "0";

      const categoryName = getField(row, "category");
      const categoryMatch = ctx.categories.find(
        (c) => c.name.toLowerCase() === categoryName.toLowerCase(),
      );
      const categoryId = categoryMatch?.id || ctx.defaultCategoryId;

      const pmName = getField(row, "payment_method");
      const pmMatch = ctx.paymentMethods.find(
        (p) => p.name.toLowerCase() === pmName.toLowerCase(),
      );
      const paymentMethodId = pmMatch?.id || ctx.defaultPaymentMethodId;

      const payerName = getField(row, "paid_by");
      const payerMatch = ctx.household.find(
        (h) => h.name.toLowerCase() === payerName.toLowerCase(),
      );
      const payerUserId = payerMatch?.id || ctx.defaultPayerUserId;

      const notes = getField(row, "notes");
      const url = getField(row, "url");

      const inactiveStr = getField(row, "inactive")?.toLowerCase() || "no";
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

    return subscriptions;
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
