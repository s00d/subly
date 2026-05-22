/**
 * Keep in sync with `src/i18n/index.ts` (SUPPORT_LOCALES + LANGUAGE_NAMES).
 */
export const SUPPORT_LOCALES = [
  "en",
  "ru",
  "zh",
  "es",
  "fr",
  "de",
  "pt",
  "ja",
  "ko",
  "ar",
  "hi",
  "tr",
] as const;

export type LocaleId = (typeof SUPPORT_LOCALES)[number];

export const LANGUAGE_NAMES: Record<LocaleId, string> = {
  en: "English",
  ru: "Русский",
  zh: "中文",
  es: "Español",
  fr: "Français",
  de: "Deutsch",
  pt: "Português",
  ja: "日本語",
  ko: "한국어",
  ar: "العربية",
  hi: "हिन्दी",
  tr: "Türkçe",
};

export interface LocaleMeta {
  ogLocale: string;
  htmlLang: string;
  canonicalPath: string;
  siteUrl: string;
  canonicalUrl: string;
}

const SITE_ORIGIN = "https://s00d.github.io/subly";

export function homePathFor(locale: LocaleId): string {
  return locale === "en" ? "/" : `/${locale}`;
}

export function localeFromPath(path: string): LocaleId {
  const normalized = path.replace(/\/+$/, "") || "/";
  const segment = normalized.split("/").filter(Boolean)[0];
  if (segment && SUPPORT_LOCALES.includes(segment as LocaleId) && segment !== "en") {
    return segment as LocaleId;
  }
  return "en";
}

export function isHomePath(path: string): boolean {
  const normalized = path.replace(/\/+$/, "") || "/";
  if (normalized === "/") return true;
  const segment = normalized.split("/").filter(Boolean)[0];
  return Boolean(segment && SUPPORT_LOCALES.includes(segment as LocaleId) && segment !== "en");
}

const LOCALE_META_RAW: Record<
  LocaleId,
  { ogLocale: string; htmlLang: string }
> = {
  en: { ogLocale: "en_US", htmlLang: "en" },
  ru: { ogLocale: "ru_RU", htmlLang: "ru" },
  zh: { ogLocale: "zh_CN", htmlLang: "zh-Hans" },
  es: { ogLocale: "es_ES", htmlLang: "es" },
  fr: { ogLocale: "fr_FR", htmlLang: "fr" },
  de: { ogLocale: "de_DE", htmlLang: "de" },
  pt: { ogLocale: "pt_BR", htmlLang: "pt" },
  ja: { ogLocale: "ja_JP", htmlLang: "ja" },
  ko: { ogLocale: "ko_KR", htmlLang: "ko" },
  ar: { ogLocale: "ar_SA", htmlLang: "ar" },
  hi: { ogLocale: "hi_IN", htmlLang: "hi" },
  tr: { ogLocale: "tr_TR", htmlLang: "tr" },
};

export const LOCALE_META: Record<LocaleId, LocaleMeta> = Object.fromEntries(
  SUPPORT_LOCALES.map((id) => {
    const path = homePathFor(id);
    const base = path === "/" ? `${SITE_ORIGIN}/` : `${SITE_ORIGIN}${path}/`;
    const raw = LOCALE_META_RAW[id];
    return [
      id,
      {
        ...raw,
        canonicalPath: path,
        siteUrl: base,
        canonicalUrl: base,
      },
    ];
  }),
) as Record<LocaleId, LocaleMeta>;

export function buildLocaleSwitcherOptions(ariaLabel: string) {
  return {
    ariaLabel,
    options: SUPPORT_LOCALES.map((id) => ({
      id,
      label: LANGUAGE_NAMES[id],
    })),
  } as const;
}

export function jsonLdLanguage(ogLocale: string): string {
  return ogLocale.split("_")[0] ?? "en";
}
