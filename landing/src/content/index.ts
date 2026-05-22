export type { LocaleId, SiteContent } from "./types";
export {
  SUPPORT_LOCALES,
  LANGUAGE_NAMES,
  LOCALE_META,
  homePathFor,
  localeFromPath,
  isHomePath,
  buildLocaleSwitcherOptions,
} from "./locales";
export { loadSiteContent, preloadSiteContent } from "./loadSiteContent";
export { docPageTitles } from "./docMeta";

/** Privacy and changelog stay on English routes for every locale. */
export const docPaths = {
  privacy: "/privacy",
  changelog: "/changelog",
} as const;
