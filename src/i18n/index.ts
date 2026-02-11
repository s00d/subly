import { ref, computed } from "vue";
import en from "./en";
import ru from "./ru";
import zh from "./zh";
import es from "./es";
import fr from "./fr";
import de from "./de";
import pt from "./pt";
import ja from "./ja";
import ko from "./ko";
import ar from "./ar";
import hi from "./hi";
import tr from "./tr";

type TranslationKeys = keyof typeof en;
type Translations = Record<TranslationKeys, string>;

const languages: Record<string, { name: string; translations: Translations }> = {
  en: { name: "English", translations: en as Translations },
  ru: { name: "Русский", translations: ru as Translations },
  zh: { name: "中文", translations: zh as Translations },
  es: { name: "Español", translations: es as Translations },
  fr: { name: "Français", translations: fr as Translations },
  de: { name: "Deutsch", translations: de as Translations },
  pt: { name: "Português", translations: pt as Translations },
  ja: { name: "日本語", translations: ja as Translations },
  ko: { name: "한국어", translations: ko as Translations },
  ar: { name: "العربية", translations: ar as Translations },
  hi: { name: "हिन्दी", translations: hi as Translations },
  tr: { name: "Türkçe", translations: tr as Translations },
};

const currentLang = ref("en");

// Computed translations map — ensures Vue tracks dependency
const currentTranslations = computed<Record<string, string>>(() => {
  const lang = languages[currentLang.value];
  return lang ? (lang.translations as unknown as Record<string, string>) : (en as unknown as Record<string, string>);
});

export function setLanguage(lang: string) {
  if (languages[lang]) {
    currentLang.value = lang;
  }
}

export function getLanguage(): string {
  return currentLang.value;
}

export function getAvailableLanguages() {
  return Object.entries(languages).map(([code, { name }]) => ({ code, name }));
}

/** Non-reactive translate – for use in non-Vue contexts (e.g. store) */
export function translate(key: string): string {
  const lang = languages[currentLang.value];
  const translations = lang ? (lang.translations as unknown as Record<string, string>) : (en as unknown as Record<string, string>);
  return translations[key] || key;
}

export function useI18n() {
  const t = (key: string): string => {
    return currentTranslations.value[key] || key;
  };

  const locale = computed(() => currentLang.value);

  return { t, locale, setLanguage, getAvailableLanguages };
}
