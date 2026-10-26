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
    card: "rounded-xl border border-border bg-surface p-4 sm:p-5 shadow-sm",
    panel: "rounded-xl border border-border bg-surface overflow-hidden",
    listContainer:
      "rounded-xl border border-border bg-surface overflow-hidden divide-y divide-border",
    listRow: "px-3 py-2.5 sm:px-4 transition-colors hover:bg-surface-hover",
    sectionTitle: "text-[13px] sm:text-sm font-semibold text-text-primary tracking-tight",
    metaText: "text-[11px] sm:text-xs text-text-muted",
    field:
      "w-full min-h-9 rounded-lg border border-border-strong bg-surface px-3 py-1.5 text-[13px] text-text-primary placeholder-text-muted transition-all hover:border-text-muted focus:outline-none focus:ring-[3px] focus:ring-primary/15 focus:border-primary disabled:opacity-50 disabled:cursor-not-allowed",
    fieldLabel: "block text-xs font-medium text-text-secondary mb-1.5",
    fieldError: "mt-1 text-xs text-danger",
  },
});

export const ui = uiRecipe();

/** Typography tokens (headings, captions, secondary KPIs) — slots recipe. */
const typoRecipe = tv({
  slots: {
    pageTitle:
      "text-lg sm:text-[22px] font-bold text-text-primary tracking-[-0.02em] truncate",
    screenTitle:
      "text-base sm:text-lg font-semibold text-text-primary tracking-tight",
    subsection: "text-xs font-medium text-text-secondary",
    statLabel: "text-[11px] sm:text-xs font-medium text-text-muted",
    statValueMd:
      "text-base sm:text-lg font-bold tabular-nums text-text-primary",
  },
});

export const typo = typoRecipe();

/**
 * Primary KPI numbers — single recipe + **`tone`** variant (works well with tw-merge).
 */
export const statValue = tv({
  base: "text-lg sm:text-[22px] font-semibold tabular-nums tracking-[-0.02em]",
  variants: {
    tone: {
      default: "text-text-primary",
      primary: "text-primary",
      green: "text-success",
    },
  },
  defaultVariants: {
    tone: "default",
  },
});

/**
 * Buttons — one recipe for every action so pages stop hand-rolling `px-4 py-2 rounded-lg …`.
 * `intent`: primary (filled accent), secondary (bordered), ghost (text only), danger.
 */
export const button = tv({
  base: [
    "inline-flex items-center justify-center gap-1.5 font-medium whitespace-nowrap select-none",
    "transition-[background-color,border-color,color,box-shadow,opacity] duration-150",
    "disabled:opacity-50 disabled:pointer-events-none",
    "focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-primary/25",
  ],
  variants: {
    intent: {
      primary: "bg-primary text-white shadow-sm hover:bg-primary-hover",
      secondary:
        "bg-surface text-text-primary border border-border-strong shadow-sm hover:bg-surface-hover",
      ghost: "text-text-secondary hover:bg-surface-hover hover:text-text-primary",
      danger: "bg-danger text-white shadow-sm hover:opacity-90",
    },
    size: {
      sm: "h-7 px-2.5 text-xs rounded-md",
      md: "h-9 px-3.5 text-[13px] rounded-lg",
      lg: "h-10 px-4 text-sm rounded-lg",
      icon: "h-9 w-9 rounded-lg",
      iconSm: "h-7 w-7 rounded-md",
    },
    /** Toggle buttons that are ON (filter panel open, selection mode). */
    pressed: {
      true: "bg-primary-light text-primary border-primary/40 hover:bg-primary-light",
    },
  },
  defaultVariants: { intent: "secondary", size: "md", pressed: false },
});

/** Stable vivid colour for letter avatars (logo fallback), picked from the item name. */
const AVATAR_TONES = [
  "bg-gradient-to-br from-blue-500 to-indigo-600 text-white",
  "bg-gradient-to-br from-violet-500 to-purple-600 text-white",
  "bg-gradient-to-br from-emerald-500 to-teal-600 text-white",
  "bg-gradient-to-br from-amber-400 to-orange-500 text-white",
  "bg-gradient-to-br from-rose-500 to-pink-600 text-white",
  "bg-gradient-to-br from-cyan-500 to-sky-600 text-white",
  "bg-gradient-to-br from-fuchsia-500 to-violet-600 text-white",
  "bg-gradient-to-br from-orange-500 to-red-500 text-white",
] as const;

export function avatarTone(name: string): string {
  let h = 0;
  for (const ch of name) h = (h * 31 + ch.charCodeAt(0)) >>> 0;
  return AVATAR_TONES[h % AVATAR_TONES.length];
}

/** Default Lucide icon sizes */
export const iconSize = {
  xs: 14,
  sm: 16,
  md: 18,
  nav: 20,
  lg: 22,
} as const;
