import { ref } from "vue";

export function useConfirmDialog() {
  const confirmId = ref<string | null>(null);

  function requestConfirm(id: string) {
    confirmId.value = id;
  }

  function confirm(callback: (id: string) => void) {
    if (confirmId.value) {
      callback(confirmId.value);
    }
    confirmId.value = null;
  }

  function cancel() {
    confirmId.value = null;
  }

  return { confirmId, requestConfirm, confirm, cancel };
}
