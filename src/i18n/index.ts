import { nextTick } from "vue";
import { createI18n } from "vue-i18n";
import type { I18n, Composer } from "vue-i18n";

export const SUPPORT_LOCALES = [
  "en", "ru", "zh", "es", "fr", "de", "pt", "ja", "ko", "ar", "hi", "tr",
] as const;

export type SupportedLocale = (typeof SUPPORT_LOCALES)[number];

const LANGUAGE_NAMES: Record<string, string> = {
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

let i18nInstance: I18n | null = null;

function getComposer(): Composer {
  if (!i18nInstance) throw new Error("i18n not initialized");
  return i18nInstance.global as unknown as Composer;
}

export function getI18nInstance(): I18n {
  if (!i18nInstance) {
    throw new Error("i18n not initialized. Call setupI18n() first.");
  }
  return i18nInstance;
}

export async function setupI18n(locale: string = "en"): Promise<I18n> {
  const i18n = createI18n({
    legacy: false,
    locale,
    fallbackLocale: "en",
    messages: {},
  });

  i18nInstance = i18n;

  await loadLocaleMessages(locale);
  if (locale !== "en") {
    await loadLocaleMessages("en");
  }

  return i18n;
}

export async function loadLocaleMessages(locale: string): Promise<void> {
  if (!i18nInstance) return;

  const composer = getComposer();

  if (composer.availableLocales.includes(locale)) return;

  const messages = await import(`../locales/${locale}.json`);

  composer.setLocaleMessage(locale, messages.default || messages);

  return nextTick();
}

export async function setLanguage(locale: string): Promise<void> {
  if (!i18nInstance) return;
  if (!SUPPORT_LOCALES.includes(locale as SupportedLocale)) return;

  await loadLocaleMessages(locale);

  const composer = getComposer();
  composer.locale.value = locale;

  document.querySelector("html")?.setAttribute("lang", locale);
}

export function getLanguage(): string {
  if (!i18nInstance) return "en";
  return getComposer().locale.value;
}

export function getAvailableLanguages(): { code: string; name: string }[] {
  return SUPPORT_LOCALES.map((code) => ({
    code,
    name: LANGUAGE_NAMES[code] || code,
  }));
}

export function translate(key: string): string {
  if (!i18nInstance) return key;
  const composer = getComposer();
  if (!composer.te(key)) return key;
  return composer.t(key);
}

export { useI18n } from "vue-i18n";
