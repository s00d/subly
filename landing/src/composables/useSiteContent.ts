import { computed, ref, shallowRef, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  docPaths,
  homePathFor,
  isHomePath,
  loadSiteContent,
  localeFromPath,
  type LocaleId,
} from "@/content/index";
import type { SiteContent } from "@/content/types";

export function useSiteContent() {
  const route = useRoute();
  const router = useRouter();

  const locale = computed<LocaleId>(() => localeFromPath(route.path));
  const content = shallowRef<SiteContent | null>(null);
  const loading = ref(true);
  const loadError = ref<string | null>(null);

  let loadGeneration = 0;

  watch(
    locale,
    async (loc) => {
      const generation = ++loadGeneration;
      loading.value = true;
      loadError.value = null;

      try {
        const loaded = await loadSiteContent(loc);
        if (generation !== loadGeneration) return;
        content.value = loaded;
      } catch (err) {
        if (generation !== loadGeneration) return;
        content.value = null;
        loadError.value = err instanceof Error ? err.message : String(err);
      } finally {
        if (generation === loadGeneration) {
          loading.value = false;
        }
      }
    },
    { immediate: true },
  );

  const ready = computed(() => content.value !== null && !loading.value);

  const homePath = computed(() => homePathFor(locale.value));
  const isHome = computed(() => isHomePath(route.path));

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
    loading,
    ready,
    loadError,
    homePath,
    isHome,
    docPaths,
    switchLocale,
    sectionLink,
  };
}
