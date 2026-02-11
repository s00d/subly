import { ref, type Component, markRaw } from "vue";

export interface HeaderAction {
  id: string;
  icon: Component;
  title: string;
  onClick: () => void;
}

const actions = ref<HeaderAction[]>([]);

export function useHeaderActions() {
  function setActions(newActions: HeaderAction[]) {
    actions.value = newActions.map((a) => ({
      ...a,
      icon: markRaw(a.icon as any),
    }));
  }

  function clearActions() {
    actions.value = [];
  }

  return { actions, setActions, clearActions };
}
