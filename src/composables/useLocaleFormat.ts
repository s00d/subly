import { computed } from "vue";
import { useI18n } from "@/i18n";

/**
 * Map app language codes to BCP-47 locale tags for Intl APIs.
 */
const localeMap: Record<string, string> = {
  en: "en-US",
  ru: "ru-RU",
  zh: "zh-CN",
  es: "es-ES",
  fr: "fr-FR",
  de: "de-DE",
  pt: "pt-BR",
  ja: "ja-JP",
  ko: "ko-KR",
  ar: "ar-SA",
  hi: "hi-IN",
  tr: "tr-TR",
};

/**
 * Get BCP-47 locale string for a given app language code.
 */
export function getLocaleTag(lang: string): string {
  return localeMap[lang] || "en-US";
}

/**
 * Reactive composable that provides locale-aware date & number formatting.
 */
export function useLocaleFormat() {
  const { locale } = useI18n();

  const localeTag = computed(() => getLocaleTag(locale.value));

  // ─── Date formatting ───────────────────────────────────────────

  /** Full date: "Monday, January 5, 2026" / "понедельник, 5 января 2026 г." */
  function fmtDateFull(dateStr: string): string {
    try {
      return new Intl.DateTimeFormat(localeTag.value, {
        weekday: "short",
        month: "long",
        day: "numeric",
        year: "numeric",
      }).format(new Date(dateStr));
    } catch {
      return dateStr;
    }
  }

  /** Medium date: "Jan 5, 2026" / "5 янв. 2026 г." */
  function fmtDateMedium(dateStr: string): string {
    try {
      return new Intl.DateTimeFormat(localeTag.value, {
        month: "short",
        day: "numeric",
        year: "numeric",
      }).format(new Date(dateStr));
    } catch {
      return dateStr;
    }
  }

  /** Short date: "Jan 5" / "5 янв." */
  function fmtDateShort(dateStr: string): string {
    try {
      return new Intl.DateTimeFormat(localeTag.value, {
        month: "short",
        day: "numeric",
      }).format(new Date(dateStr));
    } catch {
      return dateStr;
    }
  }

  /** Month + year: "January 2026" / "январь 2026 г." */
  function fmtMonthYear(date: Date): string {
    try {
      return new Intl.DateTimeFormat(localeTag.value, {
        month: "long",
        year: "numeric",
      }).format(date);
    } catch {
      return `${date.getMonth() + 1}/${date.getFullYear()}`;
    }
  }

  // ─── Number formatting ─────────────────────────────────────────

  /** Decimal number: "1,234.56" / "1 234,56" */
  function fmtNumber(value: number, fractionDigits: number = 2): string {
    try {
      return new Intl.NumberFormat(localeTag.value, {
        minimumFractionDigits: fractionDigits,
        maximumFractionDigits: fractionDigits,
      }).format(value);
    } catch {
      return value.toFixed(fractionDigits);
    }
  }

  /** Percentage: "45.2%" / "45,2 %" */
  function fmtPercent(value: number, fractionDigits: number = 1): string {
    try {
      return new Intl.NumberFormat(localeTag.value, {
        style: "percent",
        minimumFractionDigits: fractionDigits,
        maximumFractionDigits: fractionDigits,
      }).format(value / 100);
    } catch {
      return `${value.toFixed(fractionDigits)}%`;
    }
  }

  /** Currency: "$1,234.56" / "1 234,56 $" */
  function fmtCurrency(amount: number, currencyCode: string): string {
    try {
      return new Intl.NumberFormat(localeTag.value, {
        style: "currency",
        currency: currencyCode,
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      }).format(amount);
    } catch {
      return `${currencyCode} ${fmtNumber(amount)}`;
    }
  }

  return {
    localeTag,
    fmtDateFull,
    fmtDateMedium,
    fmtDateShort,
    fmtMonthYear,
    fmtNumber,
    fmtPercent,
    fmtCurrency,
  };
}

// ─── Non-reactive helpers (for services/store) ─────────────────

import { getLanguage } from "@/i18n";

export function getLocaleTagNonReactive(): string {
  return getLocaleTag(getLanguage());
}

export function formatCurrencyLocale(amount: number, currencyCode: string, _symbol?: string): string {
  const tag = getLocaleTagNonReactive();
  try {
    return new Intl.NumberFormat(tag, {
      style: "currency",
      currency: currencyCode,
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(amount);
  } catch {
    const s = _symbol || currencyCode;
    return `${s}${new Intl.NumberFormat(tag, { minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(amount)}`;
  }
}

export function formatDateShortNonReactive(dateStr: string): string {
  const tag = getLocaleTagNonReactive();
  try {
    return new Intl.DateTimeFormat(tag, { month: "short", day: "numeric" }).format(new Date(dateStr));
  } catch {
    return dateStr;
  }
}
