import { defineStore } from "pinia";
import { ref } from "vue";
import { getCatalogsUsageSummary, type CatalogsUsageSummary } from "@/services/catalogClient";

export const useCatalogsUsageStore = defineStore("catalogsUsage", () => {
  const usage = ref<CatalogsUsageSummary | null>(null);
  const loading = ref(false);
  const initialized = ref(false);

  async function refresh() {
    if (loading.value) return;
    loading.value = true;
    try {
      usage.value = await getCatalogsUsageSummary();
      initialized.value = true;
    } finally {
      loading.value = false;
    }
  }

  async function ensureLoaded() {
    if (initialized.value) return;
    await refresh();
  }

  async function refreshIfLoaded() {
    if (!initialized.value) return;
    await refresh();
  }

  return {
    usage,
    loading,
    initialized,
    refresh,
    ensureLoaded,
    refreshIfLoaded,
  };
});
