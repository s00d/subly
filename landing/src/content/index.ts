import { enContent } from "./site.en";
import { ruContent } from "./site.ru";
import type { LocaleId, SiteContent } from "./types";

export type { LocaleId, SiteContent } from "./types";

const contentByLocale: Record<LocaleId, SiteContent> = {
  en: enContent,
  ru: ruContent,
};

export function getSiteContent(locale: LocaleId): SiteContent {
  return contentByLocale[locale];
}

export function localeFromPath(path: string): LocaleId {
  return path === "/ru" || path.startsWith("/ru/") ? "ru" : "en";
}

export function homePathFor(locale: LocaleId): string {
  return locale === "ru" ? "/ru" : "/";
}

/** Privacy and changelog stay on English routes for every locale. */
export const docPaths = {
  privacy: "/privacy",
  changelog: "/changelog",
} as const;
