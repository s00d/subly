import { onMounted, onUnmounted, ref, type Ref } from "vue";

export function useActiveSection(sectionIds: readonly string[]): Ref<string> {
  const active = ref(sectionIds[0] ?? "");
  let observer: IntersectionObserver | null = null;

  onMounted(() => {
    const elements = sectionIds
      .map((id) => document.getElementById(id))
      .filter((el): el is HTMLElement => el != null);

    if (elements.length === 0) return;

    observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((e) => e.isIntersecting)
          .sort((a, b) => b.intersectionRatio - a.intersectionRatio);
        const top = visible[0]?.target;
        if (top?.id) active.value = top.id;
      },
      { rootMargin: "-40% 0px -45% 0px", threshold: [0, 0.15, 0.4] },
    );

    for (const el of elements) observer.observe(el);
  });

  onUnmounted(() => observer?.disconnect());

  return active;
}
