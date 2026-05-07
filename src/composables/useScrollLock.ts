import { watch, onBeforeUnmount, type Ref } from "vue";

let lockCount = 0;

function applyBodyLock() {
  if (typeof document === "undefined") return;
  document.body.style.overflow = lockCount > 0 ? "hidden" : "";
}

export function useScrollLock(isLocked: Ref<boolean>) {
  if (typeof document === "undefined") return;

  let wasLocked = false;
  watch(
    isLocked,
    (locked) => {
      if (locked && !wasLocked) {
        lockCount += 1;
        wasLocked = true;
      } else if (!locked && wasLocked) {
        lockCount = Math.max(0, lockCount - 1);
        wasLocked = false;
      }
      applyBodyLock();
    },
    { immediate: true },
  );

  onBeforeUnmount(() => {
    if (wasLocked) {
      lockCount = Math.max(0, lockCount - 1);
      wasLocked = false;
    }
    applyBodyLock();
  });
}
