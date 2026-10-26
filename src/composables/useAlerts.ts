import { ref } from "vue";
import type { InAppAlert } from "@/services/notificationsClient";

const alerts = ref<InAppAlert[]>([]);
const dismissedIds = new Set<string>();

export function useAlerts() {
  function setAlerts(newAlerts: InAppAlert[]) {
    alerts.value = newAlerts.filter((a) => !dismissedIds.has(a.id));
  }

  function dismiss(id: string) {
    dismissedIds.add(id);
    alerts.value = alerts.value.filter((a) => a.id !== id);
  }

  function dismissAll() {
    for (const a of alerts.value) {
      dismissedIds.add(a.id);
    }
    alerts.value = [];
  }

  return { alerts, setAlerts, dismiss, dismissAll };
}
