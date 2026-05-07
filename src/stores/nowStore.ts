import { defineStore } from "pinia";
import { ref } from "vue";

let timer: ReturnType<typeof setInterval> | null = null;

export const useNowStore = defineStore("now", () => {
  const now = ref(new Date());

  function tick() {
    now.value = new Date();
  }

  function ensureStarted() {
    if (timer) return;
    timer = setInterval(tick, 60_000);
  }

  function refresh() {
    tick();
  }

  return { now, ensureStarted, refresh };
});

