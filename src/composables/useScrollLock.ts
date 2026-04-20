import { watch, onBeforeUnmount, type Ref } from "vue";

export function useScrollLock(isLocked: Ref<boolean>) {
  if (typeof document === "undefined") return;

  watch(
    isLocked,
    (locked) => {
      document.body.style.overflow = locked ? "hidden" : "";
    },
    { immediate: true },
  );

  onBeforeUnmount(() => {
    document.body.style.overflow = "";
  });
}
