import { defineStore } from "pinia";
import { ref, computed } from "vue";

let timer: ReturnType<typeof setInterval> | null = null;
let midnightTimer: ReturnType<typeof setTimeout> | null = null;

function formatTodayId(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

function msUntilNextLocalMidnight(): number {
  const n = new Date();
  const next = new Date(n.getFullYear(), n.getMonth(), n.getDate() + 1);
  return Math.max(0, next.getTime() - n.getTime());
}

export const useNowStore = defineStore("now", () => {
  const now = ref(new Date());

  const todayId = computed(() => formatTodayId(now.value));

  function tick() {
    now.value = new Date();
  }

  function scheduleMidnightTick() {
    if (midnightTimer) clearTimeout(midnightTimer);
    midnightTimer = setTimeout(() => {
      tick();
      scheduleMidnightTick();
    }, msUntilNextLocalMidnight());
  }

  function ensureStarted() {
    if (timer) return;
    timer = setInterval(tick, 60_000);
    scheduleMidnightTick();
  }

  function refresh() {
    tick();
  }

  return { now, todayId, ensureStarted, refresh };
});
