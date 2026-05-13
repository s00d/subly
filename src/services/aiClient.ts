import { Channel } from "@tauri-apps/api/core";
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

/**
 * What the LLM classified the expense-side input as. Drives the badge in
 * the preview dialog and the receipt-only `metadata.lineItems` block.
 */
export type AiImportKind = "receipt" | "statement";

/**
 * Optional receipt-specific extras returned alongside the expense row list.
 */
export interface AiImportMetadata {
  merchantName?: string;
  totalAmount?: number;
  lineItems: ExpenseDraftLineItem[];
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

/**
 * Subscription-side classification. The model decides between a single
 * confirmation (`single`, e.g. one App Store receipt) or a list of
 * active services (`list`, e.g. the Google Play subscriptions screen).
 */
export type AiSubscriptionImportKind = "single" | "list";

export interface AiSubscriptionDraftRow {
  source: "llm";
  draft: SubscriptionDraft;
}

/**
 * Surface the smart command is operating against. Drives the schema /
 * row shape returned by the backend and the preview list the frontend
 * renders.
 */
export type AiSmartSurface = "expense" | "subscription";

/**
 * Tagged result of `ai_smart_input`. Frontend uses TS-discriminated-union
 * narrowing on `surface` to render the right preview list and bulk-save
 * handler. Mirrors `AiSmartResultDto` in `commands/ai/dto.rs`.
 */
export type AiSmartResult =
  | {
      surface: "expense";
      kind: AiImportKind;
      format: "text" | "image" | "csv" | "xlsx" | "json" | "pdf";
      rows: StatementDraftRow[];
      metadata: AiImportMetadata;
      stats: StatementImportStats;
      usage?: AiUsage;
    }
  | {
      surface: "subscription";
      kind: AiSubscriptionImportKind;
      format: "text" | "image" | "csv" | "xlsx" | "json" | "pdf";
      rows: AiSubscriptionDraftRow[];
      stats: StatementImportStats;
      usage?: AiUsage;
    };

/** Streamed progress event emitted while the backend extracts drafts. */
export type AiImportProgress =
  | { kind: "detected"; format: string }
  | { kind: "heuristic"; resolved: number; unresolved: number }
  | { kind: "llmStart"; chunks: number }
  | { kind: "llmChunk"; index: number; total: number }
  | { kind: "done"; total: number };

export interface AiSmartInputArgs {
  surface: AiSmartSurface;
  /** Free-form description. Mutually optional with `bytes`. */
  text?: string;
  /** File / image bytes. Mutually optional with `text`. */
  bytes?: Uint8Array;
  /** Required when `bytes` is set. Ignored otherwise. */
  mime?: string;
  /** BCP-47 short tag; the backend instructs the LLM to reply in this language. */
  locale?: string;
}

/**
 * Single entry point for every AI extraction in the app. Accepts text *or*
 * file bytes (one of the two), routes to the right backend pipeline based
 * on `surface` + MIME, and returns the surface-tagged result envelope.
 *
 * Progress uses `tauri::ipc::Channel`; payload is passed through `callCommand`
 * like other IPC so errors normalize to `CommandError`.
 */
export async function aiSmartInput(
  args: AiSmartInputArgs,
  onProgress?: (ev: AiImportProgress) => void,
): Promise<AiSmartResult> {
  const channel = new Channel<AiImportProgress>();
  if (onProgress) {
    channel.onmessage = (ev) => {
      try {
        onProgress(ev);
      } catch (e) {
        console.warn("[aiSmartInput] progress handler threw", e);
      }
    };
  }
  return callCommand<AiSmartResult>("ai_smart_input", {
    surface: args.surface,
    text: args.text ?? "",
    bytes: args.bytes ? Array.from(args.bytes) : [],
    mime: args.mime ?? "",
    locale: args.locale,
    onProgress: channel,
  });
}

/** Convenience: keyring account name for a provider's API key (mirrors backend). */
export function aiApiKeyName(providerType: AiProviderType): string {
  return `aiApiKey.${providerType}`;
}
