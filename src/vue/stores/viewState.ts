// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

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
    isTopBarActive.value = status ?? !isTopBarActive.value;
  }

  return {
    isTopBarActive,
    activeView,
    setActiveView,
    toggleTopBar,
  };
});
