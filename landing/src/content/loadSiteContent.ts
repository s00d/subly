import type { SiteContent } from "./types";
import type { LocaleId } from "./locales";

type ContentLoader = () => Promise<SiteContent>;

const loaders: Record<LocaleId, ContentLoader> = {
  en: () => import("./site.en").then((m) => m.enContent),
  ru: () => import("./site.ru").then((m) => m.ruContent),
  zh: () => import("./site.zh").then((m) => m.zhContent),
  es: () => import("./site.es").then((m) => m.esContent),
  fr: () => import("./site.fr").then((m) => m.frContent),
  de: () => import("./site.de").then((m) => m.deContent),
  pt: () => import("./site.pt").then((m) => m.ptContent),
  ja: () => import("./site.ja").then((m) => m.jaContent),
  ko: () => import("./site.ko").then((m) => m.koContent),
  ar: () => import("./site.ar").then((m) => m.arContent),
  hi: () => import("./site.hi").then((m) => m.hiContent),
  tr: () => import("./site.tr").then((m) => m.trContent),
};

const cache = new Map<LocaleId, SiteContent>();
const inflight = new Map<LocaleId, Promise<SiteContent>>();

/** Load locale bundle on demand (Vite splits each `site.*` into its own chunk). */
export function loadSiteContent(locale: LocaleId): Promise<SiteContent> {
  const cached = cache.get(locale);
  if (cached) return Promise.resolve(cached);

  const pending = inflight.get(locale);
  if (pending) return pending;

  const promise = loaders[locale]().then((content) => {
    cache.set(locale, content);
    inflight.delete(locale);
    return content;
  });

  inflight.set(locale, promise);
  return promise;
}

/** Warm a locale chunk before navigation (e.g. hover on language menu). */
export function preloadSiteContent(locale: LocaleId): void {
  if (cache.has(locale) || inflight.has(locale)) return;
  void loadSiteContent(locale);
}
