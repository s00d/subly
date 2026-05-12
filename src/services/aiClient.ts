import { Channel, invoke } from "@tauri-apps/api/core";
import { callCommand } from "@/services/commandClient";

/**
 * AI providers known to the backend. Keep in sync with
 * `src-tauri/src/commands/ai/providers/mod.rs::provider_descriptors`.
 */
export type AiProviderType =
  | "openrouter"
  | "openai_compat"
  | "openai"
  | "groq"
  | "deepseek"
  | "gemini"
  | "mistral";

export interface AiProviderMeta {
  readonly type: AiProviderType;
  readonly name: string;
  readonly defaultModel: string;
  readonly defaultBaseUrl?: string;
  readonly requiresEndpoint: boolean;
  readonly requiresKey: boolean;
  readonly docsUrl: string;
  readonly description: string;
  readonly recommendedModels: readonly AiModelPreset[];
}

/**
 * Curated model option surfaced in the settings dropdown. Mirrors
 * `commands::ai::providers::presets::ModelPreset` on the backend.
 */
export interface AiModelPreset {
  readonly id: string;
  readonly label: string;
  readonly supportsVision: boolean;
  /** Feature kinds this preset is recommended for. */
  readonly recommendedFor: readonly AiFeatureKind[];
}

/** Feature identifiers used for preset recommendations and guards. */
export type AiFeatureKind =
  | "subscription"
  | "expense"
  | "statement"
  | "receipt";

/**
 * Determine whether the selected model supports image input. Custom models
 * (not matching any preset) return `false` so the UI can warn the user that
 * vision isn't verified.
 */
export function aiModelSupportsVision(
  providers: readonly AiProviderMeta[],
  activeProviderType: AiProviderType | undefined,
  activeModelId: string | undefined,
): boolean {
  if (!activeProviderType || !activeModelId) return false;
  const provider = providers.find((p) => p.type === activeProviderType);
  if (!provider) return false;
  const preset = provider.recommendedModels.find(
    (m) => m.id.toLowerCase() === activeModelId.toLowerCase(),
  );
  return preset?.supportsVision ?? false;
}

export interface AiTestResult {
  readonly ok: boolean;
  readonly latencyMs: number;
  readonly echo: string;
  readonly model: string;
  readonly error?: string;
}

/**
 * Per-feature toggles. The backend uses `config:aiFeatures` for this object —
 * frontend reads/writes it through `getConfigValue`/`setConfigValue`.
 */
export interface AiFeatureToggles {
  subscriptionInput: boolean;
  expenseInput: boolean;
  statementImport: boolean;
  receiptImport: boolean;
}

export const DEFAULT_AI_FEATURES: AiFeatureToggles = {
  subscriptionInput: true,
  expenseInput: true,
  statementImport: true,
  receiptImport: true,
};

/**
 * LLM-extracted subscription. Numeric `cycle` follows `Subscription.cycle`
 * (1=day, 2=week, 3=month, 4=year). Unresolved IDs are empty strings, the
 * accompanying `currencyCode`/`categoryHint` fields keep the original LLM
 * output so the UI can show "we guessed XYZ".
 */
export interface SubscriptionDraft {
  name: string;
  price: number;
  currencyId: string;
  currencyCode: string;
  cycle: number;
  frequency: number;
  categoryId: string;
  categoryHint: string;
  paymentMethodId: string;
  startDate?: string;
  nextPayment?: string;
  notes: string;
  url: string;
  tags: string[];
  warnings: string[];
  confidence: number;
  usage?: AiUsage;
}

/**
 * Token usage echoed back from a single LLM call. Counters are optional
 * because most local OpenAI-compatible servers (Ollama, LM Studio) don't
 * report `usage`. Surfaces here as a hook for future "monthly limit" UI.
 */
export interface AiUsage {
  inputTokens?: number;
  outputTokens?: number;
  reasoningTokens?: number;
  cachedTokens?: number;
}

export async function aiGetProviders(): Promise<AiProviderMeta[]> {
  return callCommand("ai_get_providers");
}

export async function aiTestConnection(): Promise<AiTestResult> {
  return callCommand("ai_test_connection");
}

export async function aiExtractSubscriptionFromText(
  text: string,
  locale?: string,
): Promise<SubscriptionDraft> {
  return callCommand("ai_extract_subscription_from_text", { text, locale });
}

/**
 * LLM-extracted one-off expense. Same conventions as `SubscriptionDraft`:
 * unresolved IDs are empty strings, original LLM hints (`currencyCode`,
 * `categoryHint`) are kept so the UI can surface "we guessed X".
 */
export interface ExpenseDraft {
  name: string;
  amount: number;
  currencyId: string;
  currencyCode: string;
  date: string;
  categoryId: string;
  categoryHint: string;
  paymentMethodId: string;
  tags: string[];
  notes: string;
  url: string;
  lineItems: ExpenseDraftLineItem[];
  warnings: string[];
  confidence: number;
  usage?: AiUsage;
}

export interface ExpenseDraftLineItem {
  name: string;
  amount: number;
}

export async function aiExtractExpenseFromText(
  text: string,
  locale?: string,
): Promise<ExpenseDraft> {
  return callCommand("ai_extract_expense_from_text", { text, locale });
}

/**
 * Vision-LLM receipt extraction. Accepts image/png|jpeg|webp|gif or
 * application/pdf. Returns an `ExpenseDraft` with optional `lineItems`.
 */
export async function aiExtractReceipt(
  bytes: Uint8Array,
  mime: string,
  locale?: string,
): Promise<ExpenseDraft> {
  return callCommand("ai_extract_receipt", {
    bytes: Array.from(bytes),
    mime,
    locale,
  });
}

/**
 * Result of `ai_import_statement_file`. Each row carries a `source` tag so
 * the UI can highlight which rows were resolved cheaply (heuristics) vs
 * which required an LLM round-trip.
 */
export interface StatementImportResult {
  format: "csv" | "xlsx" | "json" | "pdf" | "text";
  rows: StatementDraftRow[];
  stats: StatementImportStats;
  usage?: AiUsage;
}

export interface StatementDraftRow {
  source: "heuristic" | "llm";
  draft: ExpenseDraft;
}

export interface StatementImportStats {
  matchedByHeuristic: number;
  matchedByLlm: number;
  failed: number;
  total: number;
}

/** Streamed progress event emitted while parsing a statement file. */
export type StatementImportProgress =
  | { kind: "detected"; format: string }
  | { kind: "heuristic"; resolved: number; unresolved: number }
  | { kind: "llmStart"; chunks: number }
  | { kind: "llmChunk"; index: number; total: number }
  | { kind: "done"; total: number };

/**
 * Stream-aware statement import. We bypass `callCommand` here because the
 * backend signature uses `tauri::ipc::Channel`, which the generic helper
 * doesn't model.
 */
export async function aiImportStatementFile(
  bytes: Uint8Array,
  mime: string,
  locale: string | undefined,
  onProgress?: (ev: StatementImportProgress) => void,
): Promise<StatementImportResult> {
  const channel = new Channel<StatementImportProgress>();
  if (onProgress) {
    channel.onmessage = (ev) => {
      try {
        onProgress(ev);
      } catch (e) {
        console.warn("[aiImportStatementFile] progress handler threw", e);
      }
    };
  }
  return invoke("ai_import_statement_file", {
    bytes: Array.from(bytes),
    mime,
    locale,
    onProgress: channel,
  });
}

/** Convenience: keyring account name for a provider's API key (mirrors backend). */
export function aiApiKeyName(providerType: AiProviderType): string {
  return `aiApiKey.${providerType}`;
}
