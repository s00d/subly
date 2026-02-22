import { defineStore } from "pinia";
import { ref } from "vue";

export const useConverterStore = defineStore("converter", () => {
  const baseAmount = ref(1);
  const order = ref<string[]>([]);

  function setOrder(ids: string[]) {
    order.value = [...ids];
  }

  function moveUp(id: string) {
    const idx = order.value.indexOf(id);
    if (idx > 0) {
      const arr = [...order.value];
      [arr[idx - 1], arr[idx]] = [arr[idx], arr[idx - 1]];
      order.value = arr;
    }
  }

  function moveDown(id: string) {
    const idx = order.value.indexOf(id);
    if (idx >= 0 && idx < order.value.length - 1) {
      const arr = [...order.value];
      [arr[idx], arr[idx + 1]] = [arr[idx + 1], arr[idx]];
      order.value = arr;
    }
  }

  function reset() {
    baseAmount.value = 1;
  }

  return { baseAmount, order, setOrder, moveUp, moveDown, reset };
});
