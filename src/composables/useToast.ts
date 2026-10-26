import { ref } from "vue";

const toastMsg = ref("");
export type ToastType = "success" | "error" | "info" | "warning";

const toastType = ref<ToastType>("success");
const showToast = ref(false);

export function useToast() {
  function toast(msg: string, type: ToastType = "success") {
    toastMsg.value = msg;
    toastType.value = type;
    showToast.value = true;
  }

  function closeToast() {
    showToast.value = false;
  }

  return { toastMsg, toastType, showToast, toast, closeToast };
}
