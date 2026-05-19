import { nextTick, onMounted, onUnmounted, ref, watch, type Ref } from "vue";
import { useRoute } from "vue-router";

/** Sticky header + scroll-margin — section counts as reached when its top crosses this line */
const SCROLL_TRIGGER_PX = 96;

export function useActiveSection(sectionIds: readonly string[]): Ref<string> {
  const active = ref(sectionIds[0] ?? "");
  const route = useRoute();
  let listening = false;

  function updateActive() {
    let current = sectionIds[0] ?? "";
    for (const id of sectionIds) {
      const el = document.getElementById(id);
      if (!el) continue;
      if (el.getBoundingClientRect().top <= SCROLL_TRIGGER_PX) {
        current = id;
      }
    }
    active.value = current;
  }

  function bind() {
    const hasSections = sectionIds.some((id) => document.getElementById(id) != null);
    if (!hasSections || listening) return false;

    listening = true;
    updateActive();
    window.addEventListener("scroll", updateActive, { passive: true });
    window.addEventListener("resize", updateActive, { passive: true });
    return true;
  }

  function unbind() {
    if (!listening) return;
    listening = false;
    window.removeEventListener("scroll", updateActive);
    window.removeEventListener("resize", updateActive);
  }

  async function setup() {
    await nextTick();
    unbind();
    if (bind()) return;
    requestAnimationFrame(() => {
      unbind();
      bind();
    });
  }

  onMounted(setup);
  watch(() => route.path, setup);
  onUnmounted(unbind);

  return active;
}
