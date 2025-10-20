<template>
  <div
    class="grid grid-cols-3 items-center h-[30px] text-white font-sans select-none bg-[#0D0D0D] border-b border-gray-800 flex-shrink-0 relative z-50 topbar-container"
  >
    <!-- Overlay when top bar is not active -->
    <div
      v-if="!viewState.isTopBarActive"
      class="absolute inset-0 bg-black opacity-50 z-50 cursor-not-allowed col-span-3"
    ></div>

    <!-- Left side buttons -->
    <div class="flex items-center justify-start">
      <!-- Dashboard button -->
      <div
        class="flex items-center px-3 h-[30px] transition-colors duration-200 flex-shrink-0 hover:bg-gray-800 cursor-pointer"
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
        class="flex items-center px-3 h-[30px] transition-colors duration-200 flex-shrink-0 hover:bg-gray-800 cursor-pointer"
        :class="{
          'bg-gray-800': viewState.activeView === 'workspace',
        }"
        @click="setActiveView('workspace')"
      >
        <LayoutGrid
          :size="16"
          class="transition-opacity duration-200"
          :class="
            viewState.activeView === 'workspace'
              ? 'opacity-100'
              : 'opacity-60 hover:opacity-100'
          "
        />
      </div>

      <!-- SSH Profiles button -->
      <div
        class="flex items-center px-3 h-[30px] transition-colors duration-200 flex-shrink-0 hover:bg-gray-800 cursor-pointer"
        :class="{
          'bg-gray-800': isOverlayVisible('ssh-profile-drawer'),
        }"
        @click="toggleOverlay('ssh-profile-drawer')"
      >
        <Server
          :size="16"
          class="transition-opacity duration-200"
          :class="
            isOverlayVisible('ssh-profile-drawer')
              ? 'opacity-100'
              : 'opacity-60 hover:opacity-100'
          "
        />
      </div>
    </div>

    <!-- Center content -->
    <div class="flex justify-center items-center h-full" />

    <!-- Right side buttons -->
    <div class="flex items-center justify-end">
      <!-- Saved Commands -->
      <Button
        title="Saved Commands"
        variant="ghost"
        size="sm"
        :icon="Terminal"
        :class="
          isOverlayVisible('saved-command-drawer')
            ? 'bg-gray-800 text-gray-400 hover:text-white'
            : ''
        "
        @click="toggleOverlay('saved-command-drawer')"
      />

      <!-- Tunnel Manager -->
      <Button
        title="SSH Tunnel Manager"
        variant="ghost"
        size="sm"
        :icon="Route"
        :class="
          isOverlayVisible('tunnel-manager-modal')
            ? 'bg-gray-800 text-gray-400 hover:text-white'
            : ''
        "
        @click="toggleOverlay('tunnel-manager-modal')"
      />

      <!-- SSH Key Manager -->
      <Button
        title="SSH Key Manager"
        variant="ghost"
        size="sm"
        :icon="Key"
        :class="
          isOverlayVisible('ssh-key-manager-modal')
            ? 'bg-gray-800 text-gray-400 hover:text-white'
            : ''
        "
        @click="toggleOverlay('ssh-key-manager-modal')"
      />

      <!-- Master Password  -->
      <Button
        title="Master Password Settings"
        variant="ghost"
        size="sm"
        :icon="Shield"
        :class="
          isOverlayVisible('master-password-settinTopgs')
            ? 'bg-gray-800 text-gray-400 hover:text-white'
            : ''
        "
        @click="toggleOverlay('master-password-settings')"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  LayoutGrid,
  Server,
  Shield,
  Key,
  Route,
  Terminal,
} from "lucide-vue-next";
import Button from "./ui/Button.vue";

import { useViewStateStore } from "../stores/viewState";
import { useOverlay } from "../composables/useOverlay";

const viewState = useViewStateStore();
const { openOverlay, closeOverlay, closeAllOverlays, isOverlayVisible } =
  useOverlay();

const setActiveView = (view: "dashboard" | "workspace" | "fileManager") => {
  if (!viewState.isTopBarActive || viewState.activeView === view) return;

  closeAllOverlays();

  viewState.setActiveView(view);
};

const toggleOverlay = (overlayName: string) => {
  if (isOverlayVisible(overlayName)) {
    closeAllOverlays();
    closeOverlay(overlayName);
  } else {
    closeAllOverlays();
    openOverlay(overlayName);
  }
};
</script>
