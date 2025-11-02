import { defineStore } from "pinia";
import { ref } from "vue";

/**
 * UI State Store
 * Manages the state of current active views.
 */
export const useViewStateStore = defineStore("viewState", () => {
  const isTopBarActive = ref(false);

  const activeView = ref<"dashboard" | "workspace" | "sftp">("workspace");

  function setActiveView(view: "dashboard" | "workspace" | "sftp") {
    activeView.value = view;
  }

  function toggleTopBar(status?: boolean) {
    isTopBarActive.value =
      status !== undefined ? status : !isTopBarActive.value;
  }

  return {
    isTopBarActive,
    activeView,
    setActiveView,
    toggleTopBar,
  };
});
