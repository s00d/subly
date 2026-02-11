import { ref } from "vue";

const toastMsg = ref("");
const toastType = ref<"success" | "error">("success");
const showToast = ref(false);

export function useToast() {
  function toast(msg: string, type: "success" | "error" = "success") {
    toastMsg.value = msg;
    toastType.value = type;
    showToast.value = true;
  }

  function closeToast() {
    showToast.value = false;
  }

  return { toastMsg, toastType, showToast, toast, closeToast };
}
