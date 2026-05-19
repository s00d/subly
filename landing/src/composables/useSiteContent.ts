import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  docPaths,
  getSiteContent,
  homePathFor,
  localeFromPath,
  type LocaleId,
} from "@/content/index";

export function useSiteContent() {
  const route = useRoute();
  const router = useRouter();

  const locale = computed<LocaleId>(() => localeFromPath(route.path));
  const content = computed(() => getSiteContent(locale.value));
  const homePath = computed(() => homePathFor(locale.value));
  const isHome = computed(() => route.path === "/" || route.path === "/ru");

  function switchLocale(target: LocaleId) {
    if (target === locale.value) return;

    if (route.path === docPaths.privacy || route.path === docPaths.changelog) {
      void router.push(homePathFor(target));
      return;
    }

    const hash = route.hash;
    void router.push({ path: homePathFor(target), hash });
  }

  function sectionLink(sectionId: string) {
    return { path: homePath.value, hash: `#${sectionId}` };
  }

  return {
    locale,
    content,
    homePath,
    isHome,
    docPaths,
    switchLocale,
    sectionLink,
  };
}
