<template>
  <div
    class="flex items-center h-[30px] text-white font-sans select-none bg-[#0D0D0D] border-b border-gray-800 flex-shrink-0 relative z-50 topbar-container"
  >
    <!-- Overlay when top bar is not active -->
    <div
      v-if="!viewState.isTopBarActive"
      class="absolute inset-0 bg-black opacity-50 z-50 cursor-not-allowed"
    ></div>

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

    <!-- SSH Profiles Icon -->
     <Button
      title="SSH Profiles"
      variant="ghost"
      size="sm"
      :icon="Server"
      :class="isOverlayVisible('ssh-profile-drawer') ? 'text-gray-400 hover:text-white' : ''"
      @click="openOverlay('ssh-profile-drawer')"
    />

    <div class="flex-1 h-full"></div>
  </div>
</template>

<script setup lang="ts">
import { LayoutGrid, Server } from "lucide-vue-next";
import Button from "./ui/Button.vue";

import { useViewStateStore } from "../stores/viewState";
import { useOverlay } from '../composables/useOverlay'

const viewState = useViewStateStore();
const { openOverlay, isOverlayVisible } = useOverlay();

const setActiveView = (view: "dashboard" | "workspace" | "fileManager") => {
  if (!viewState.isTopBarActive || viewState.activeView === view) return;

  viewState.setActiveView(view);
};
</script>
