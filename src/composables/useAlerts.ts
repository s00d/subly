import { ref } from "vue";
import type { InAppAlert } from "@/services/notificationsClient";

const alerts = ref<InAppAlert[]>([]);

export function useAlerts() {
  function setAlerts(newAlerts: InAppAlert[]) {
    alerts.value = newAlerts;
  }

  function dismiss(id: string) {
    alerts.value = alerts.value.filter((a) => a.id !== id);
  }

  function dismissAll() {
    alerts.value = [];
  }

  return { alerts, setAlerts, dismiss, dismissAll };
}
