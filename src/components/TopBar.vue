<template>
  <div
    class="flex items-center h-[30px] text-white font-sans select-none bg-[#0D0D0D] border-b border-gray-800 flex-shrink-0 relative z-50 topbar-container"
  >
    <!-- Dashboard button -->
    <div
      class="flex items-center px-3 h-full max-h-[30px] transition-colors duration-200 flex-shrink-0 hover:bg-gray-800 cursor-pointer"
      :class="{
        'bg-gray-800': viewState.activeView === 'dashboard',
      }"
      @click="setActiveView('dashboard')"
    >
      <img
        src="../assets/images/logo_500.png"
        alt="Dashboard"
        class="w-4 h-4 transition-opacity duration-200"
        :class="
          viewState.activeView === 'dashboard'
            ? 'opacity-100'
            : 'opacity-60 hover:opacity-100'
        "
      />
    </div>

    <!-- Workspace button -->
    <div
      class="flex items-center px-3 h-full max-h-[30px] transition-colors duration-200 flex-shrink-0 hover:bg-gray-800 cursor-pointer"
      :class="{
        'bg-gray-800': viewState.activeView === 'workspace',
      }"
      @click="setActiveView('workspace')"
    >
      <LayoutGrid
        class="w-4 h-4 transition-opacity duration-200"
        :class="
          viewState.activeView === 'workspace'
            ? 'opacity-100'
            : 'opacity-60 hover:opacity-100'
        "
      />
    </div>
    <div class="flex-1 h-full"></div>
  </div>
</template>

<script setup lang="ts">
import { LayoutGrid } from "lucide-vue-next";
import { useViewStateStore } from "../stores/viewState";
const viewState = useViewStateStore();

const setActiveView = (view: "dashboard" | "workspace" | "fileManager") => {
  if (!viewState.isTopBarActive || viewState.activeView === view) return;

  viewState.setActiveView(view);
};
</script>
