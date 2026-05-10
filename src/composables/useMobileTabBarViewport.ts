import { ref, onMounted, onUnmounted } from "vue";

function isTextInputFocused(): boolean {
  const el = document.activeElement as HTMLElement | null;
  if (!el) return false;
  const tag = el.tagName;
  if (tag === "TEXTAREA" || tag === "SELECT") return true;
  if (tag === "INPUT") {
    const type = (el as HTMLInputElement).type?.toLowerCase() ?? "text";
    if (
      type === "button" ||
      type === "submit" ||
      type === "checkbox" ||
      type === "radio" ||
      type === "file" ||
      type === "hidden" ||
      type === "range" ||
      type === "color"
    ) {
      return false;
    }
    return true;
  }
  if (el.isContentEditable) return true;
  return false;
}

/**
 * iOS WKWebView often leaves a dead gap under `position:fixed` bottom bars after the
 * virtual keyboard closes. While a text field is focused we align the bar to the visual
 * viewport; once focus leaves an editable, we pin `bottom` to 0 and nudge window scroll
 * so WebKit drops stale viewport offsets without requiring an app restart.
 */
export function useMobileTabBarViewport() {
  const bottomPx = ref(0);

  function sync() {
    const vv = window.visualViewport;
    if (!vv) {
      bottomPx.value = 0;
      return;
    }
    if (!isTextInputFocused()) {
      bottomPx.value = 0;
      return;
    }
    const gap = Math.max(0, window.innerHeight - vv.offsetTop - vv.height);
    bottomPx.value = gap;
  }

  function nudgeLayout() {
    window.scrollTo(0, 0);
    sync();
  }

  let focusOutTimers: ReturnType<typeof setTimeout>[] = [];

  function onFocusOut() {
    for (const id of focusOutTimers) clearTimeout(id);
    focusOutTimers = [];
    bottomPx.value = 0;
    requestAnimationFrame(nudgeLayout);
    focusOutTimers.push(setTimeout(nudgeLayout, 50));
    focusOutTimers.push(setTimeout(nudgeLayout, 200));
    focusOutTimers.push(setTimeout(nudgeLayout, 450));
  }

  function onFocusIn() {
    requestAnimationFrame(sync);
    setTimeout(sync, 120);
    setTimeout(sync, 400);
  }

  onMounted(() => {
    sync();
    window.addEventListener("resize", sync, { passive: true });
    window.visualViewport?.addEventListener("resize", sync, { passive: true });
    window.visualViewport?.addEventListener("scroll", sync, { passive: true });
    window.addEventListener("focusout", onFocusOut, true);
    window.addEventListener("focusin", onFocusIn, true);
  });

  onUnmounted(() => {
    for (const id of focusOutTimers) clearTimeout(id);
    window.removeEventListener("resize", sync);
    window.visualViewport?.removeEventListener("resize", sync);
    window.visualViewport?.removeEventListener("scroll", sync);
    window.removeEventListener("focusout", onFocusOut, true);
    window.removeEventListener("focusin", onFocusIn, true);
  });

  return { bottomPx };
}
