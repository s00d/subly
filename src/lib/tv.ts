import type { TV } from "tailwind-variants";
import { tv as tvBase } from "tailwind-variants";

/**
 * Project `tv` wrapper: enables **tailwind-merge** (`twMerge: true`) so composed
 * classes and `class` overrides on slot functions resolve conflicts correctly.
 *
 * @see https://www.tailwind-variants.org/docs/api-reference
 */
export const tv: TV = (options, config) =>
  tvBase(options, {
    ...config,
    twMerge: config?.twMerge ?? true,
    twMergeConfig: {
      ...config?.twMergeConfig,
    },
  });

/**
 * Shared UI primitives — Tailwind Variants **slots** recipe.
 * In templates use slot functions: `ui.sectionTitle()`, `ui.field()`, …
 * Inside other `tv({ slots })` configs pass resolved strings: `title: ui.sectionTitle()`.
 */
const uiRecipe = tv({
  slots: {
    card: [
      "rounded-2xl border border-border bg-surface p-3 sm:p-5",
      "shadow-sm transition-all duration-200 hover:shadow-md",
    ],
    panel: "rounded-2xl border border-border bg-surface-secondary overflow-hidden",
    listContainer:
      "rounded-xl border border-border bg-surface-secondary overflow-hidden divide-y divide-border",
    listRow:
      "px-3 py-2.5 sm:py-3 transition-colors hover:bg-surface dark:hover:bg-white/[0.06]",
    sectionTitle:
      "text-xs sm:text-sm font-semibold text-text-primary tracking-tight",
    metaText: "text-[11px] sm:text-xs text-text-muted",
    field:
      "w-full rounded-xl border border-border bg-surface px-4 py-2.5 text-sm text-text-primary placeholder-text-muted transition-all hover:border-text-muted focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary disabled:opacity-50 disabled:cursor-not-allowed",
    fieldLabel: "block text-xs font-medium text-text-secondary mb-1.5",
    fieldError: "mt-1 text-xs text-red-500",
  },
});

export const ui = uiRecipe();

/** Typography tokens (headings, captions, secondary KPIs) — slots recipe. */
const typoRecipe = tv({
  slots: {
    pageTitle:
      "text-base sm:text-lg font-semibold text-text-primary tracking-tight truncate",
    screenTitle:
      "text-base sm:text-lg font-semibold text-text-primary tracking-tight",
    subsection: "text-xs font-medium text-text-secondary",
    statLabel: "text-[11px] sm:text-xs text-text-muted",
    statValueMd:
      "text-base sm:text-lg font-bold tabular-nums text-text-primary",
  },
});

export const typo = typoRecipe();

/**
 * Primary KPI numbers — single recipe + **`tone`** variant (works well with tw-merge).
 */
export const statValue = tv({
  base: "text-lg sm:text-xl font-bold tabular-nums",
  variants: {
    tone: {
      default: "text-text-primary",
      primary: "text-primary",
      green: "text-green-600",
    },
  },
  defaultVariants: {
    tone: "default",
  },
});

/** Default Lucide icon sizes */
export const iconSize = {
  xs: 14,
  sm: 16,
  md: 18,
  nav: 20,
  lg: 22,
} as const;
