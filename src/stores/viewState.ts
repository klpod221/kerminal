import { defineStore } from "pinia";
import { ref } from "vue";

/**
 * UI State Store
 * Manages the state of current active views.
 */
export const useViewStateStore = defineStore("viewState", () => {
  const isTopBarActive = ref(true);

  const activeView = ref<"dashboard" | "workspace" | "fileManager">("workspace");

  function setActiveView(view: "dashboard" | "workspace" | "fileManager") {
    activeView.value = view;
  }

  function toggleTopBar() {
    isTopBarActive.value = !isTopBarActive.value;
  }

  return {
    isTopBarActive,
    activeView,
    setActiveView,
    toggleTopBar,
  };
});
