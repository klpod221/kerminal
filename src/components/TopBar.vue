<template>
  <div
    class="grid items-center h-[30px] sm:h-9 text-white font-sans select-none bg-bg-primary border-b border-gray-800 shrink-0 relative z-50 topbar-container"
    :class="isMobile ? 'grid-cols-[auto_1fr_auto]' : 'grid-cols-3'"
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
        data-tour="dashboard-btn"
        class="flex items-center h-[30px] sm:h-9 transition-colors duration-200 shrink-0 cursor-pointer touch-manipulation"
        :class="[
          viewState.activeView === 'dashboard' ? 'bg-gray-800' : '',
          isMobile ? 'px-2' : 'px-3',
        ]"
        @click="setActiveView('dashboard')"
      >
        <img
          src="../assets/images/logo_500.png"
          alt="Dashboard"
          class="transition-opacity duration-200"
          :class="[
            viewState.activeView === 'dashboard'
              ? 'opacity-100'
              : 'opacity-60 hover:opacity-100',
            isMobile ? 'w-5 h-5' : 'w-4 h-4',
          ]"
        />
      </div>

      <!-- Workspace button -->
      <div
        data-tour="workspace-btn"
        class="flex items-center h-[30px] sm:h-9 transition-colors duration-200 shrink-0 hover:bg-gray-800 cursor-pointer touch-manipulation"
        :class="[
          viewState.activeView === 'workspace' ? 'bg-gray-800' : '',
          isMobile ? 'px-2' : 'px-3',
        ]"
        @click="setActiveView('workspace')"
      >
        <LayoutGrid
          :size="isMobile ? 18 : 16"
          class="transition-opacity duration-200"
          :class="
            viewState.activeView === 'workspace'
              ? 'opacity-100'
              : 'opacity-60 hover:opacity-100'
          "
        />
      </div>

      <!-- SFTP button -->
      <div
        data-tour="sftp-btn"
        class="flex items-center h-[30px] sm:h-9 transition-colors duration-200 shrink-0 hover:bg-gray-800 cursor-pointer touch-manipulation"
        :class="[
          viewState.activeView === 'sftp' ? 'bg-gray-800' : '',
          isMobile ? 'px-2' : 'px-3',
        ]"
        @click="setActiveView('sftp')"
      >
        <FolderOpen
          :size="isMobile ? 18 : 16"
          class="transition-opacity duration-200"
          :class="
            viewState.activeView === 'sftp'
              ? 'opacity-100'
              : 'opacity-60 hover:opacity-100'
          "
        />
      </div>

      <!-- line -->
      <div class="w-px h-[20px] bg-gray-700" />

      <!-- SSH Profiles button -->
      <div
        data-tour="ssh-profiles-btn"
        class="flex items-center h-[30px] sm:h-9 transition-colors duration-200 shrink-0 hover:bg-gray-800 cursor-pointer touch-manipulation"
        :class="[
          isOverlayVisible('ssh-profile-drawer') ? 'bg-gray-800' : '',
          isMobile ? 'px-2' : 'px-3',
        ]"
        @click="toggleOverlay('ssh-profile-drawer')"
      >
        <Server
          :size="isMobile ? 18 : 16"
          class="transition-opacity duration-200"
          :class="
            isOverlayVisible('ssh-profile-drawer')
              ? 'opacity-100'
              : 'opacity-60 hover:opacity-100'
          "
        />
      </div>

      <!-- Terminal Profiles button -->
      <div
        data-tour="terminal-profiles-btn"
        class="flex items-center h-[30px] sm:h-9 transition-colors duration-200 shrink-0 hover:bg-gray-800 cursor-pointer touch-manipulation"
        :class="[
          isOverlayVisible('terminal-profile-drawer') ? 'bg-gray-800' : '',
          isMobile ? 'px-2' : 'px-3',
        ]"
        @click="toggleOverlay('terminal-profile-drawer')"
      >
        <Terminal
          :size="isMobile ? 18 : 16"
          class="transition-opacity duration-200"
          :class="
            isOverlayVisible('terminal-profile-drawer')
              ? 'opacity-100'
              : 'opacity-60 hover:opacity-100'
          "
        />
      </div>
    </div>

    <!-- Center content -->
    <div class="flex justify-center items-center h-full">
      <SyncStatusIndicator />
    </div>

    <!-- Right side buttons -->
    <div class="flex items-center justify-end">
      <!-- More menu for mobile -->
      <Button
        v-if="isMobile"
        title="More options"
        variant="ghost"
        size="sm"
        :icon="Menu"
        @click="toggleMobileMenu"
      />

      <!-- Desktop buttons -->
      <template v-else>
        <!-- Saved Commands -->
        <Button
          data-tour="saved-commands-btn"
          title="Saved Commands"
          variant="ghost"
          size="sm"
          :icon="FileCode"
          :class="
            isOverlayVisible('saved-command-drawer')
              ? 'bg-gray-800 text-gray-400 hover:text-white'
              : ''
          "
          @click="toggleOverlay('saved-command-drawer')"
        />

        <!-- Recordings -->
        <Button
          data-tour="recordings-btn"
          title="Session Recordings"
          variant="ghost"
          size="sm"
          :icon="Video"
          :class="
            isOverlayVisible('recordings-modal')
              ? 'bg-gray-800 text-gray-400 hover:text-white'
              : ''
          "
          @click="toggleOverlay('recordings-modal')"
        />

        <!-- Tunnel Manager -->
        <Button
          data-tour="tunnels-btn"
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
          data-tour="ssh-keys-btn"
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

        <!-- Sync Manager -->
        <Button
          data-tour="sync-btn"
          title="Sync Manager"
          variant="ghost"
          size="sm"
          :icon="Cloud"
          :class="
            isOverlayVisible('sync-manager-modal')
              ? 'bg-gray-800 text-gray-400 hover:text-white'
              : ''
          "
          @click="toggleOverlay('sync-manager-modal')"
        />

        <!-- Theme Selector -->
        <Button
          data-tour="theme-btn"
          title="Terminal Theme"
          variant="ghost"
          size="sm"
          :icon="Palette"
          :class="
            isOverlayVisible('theme-selector-modal')
              ? 'bg-gray-800 text-gray-400 hover:text-white'
              : ''
          "
          @click="toggleOverlay('theme-selector-modal')"
        />

        <!-- Keyboard Shortcuts -->
        <Button
          data-tour="shortcuts-btn"
          title="Keyboard Shortcuts"
          variant="ghost"
          size="sm"
          :icon="Keyboard"
          :class="
            isOverlayVisible('keyboard-shortcuts-modal')
              ? 'bg-gray-800 text-gray-400 hover:text-white'
              : ''
          "
          @click="toggleOverlay('keyboard-shortcuts-modal')"
        />
        <!-- Main Settings / Backup -->
        <Button
          data-tour="backup-btn"
          title="Backup & Restore"
          variant="ghost"
          size="sm"
          :icon="Archive"
          :class="
            isOverlayVisible('backup-restore-modal')
              ? 'bg-gray-800 text-gray-400 hover:text-white'
              : ''
          "
          @click="toggleOverlay('backup-restore-modal')"
        />

        <!-- Master Password  -->
        <Button
          data-tour="master-password-btn"
          title="Master Password Settings"
          variant="ghost"
          size="sm"
          :icon="Shield"
          :class="
            isOverlayVisible('master-password-settings')
              ? 'bg-gray-800 text-gray-400 hover:text-white'
              : ''
          "
          @click="toggleOverlay('master-password-settings')"
        />

        <!-- Update Checker -->
        <div class="relative">
          <Button
            data-tour="updates-btn"
            title="Check for Updates"
            variant="ghost"
            size="sm"
            :icon="Download"
            :class="
              isOverlayVisible('updater-modal')
                ? 'bg-gray-800 text-gray-400 hover:text-white'
                : ''
            "
            @click="toggleOverlay('updater-modal')"
          />
          <span
            v-if="updaterStore.hasUpdate"
            class="absolute -top-0.5 -right-0.5 w-2.5 h-2.5 bg-red-500 rounded-full border border-bg-primary animate-pulse"
          />
        </div>
      </template>
    </div>

    <!-- Mobile menu overlay -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition-opacity duration-200"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition-opacity duration-200"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div
          v-if="showMobileMenu"
          class="fixed inset-0 bg-black/50 z-60 top-[30px]"
          @click="showMobileMenu = false"
        >
          <div
            class="absolute right-0 top-0 bg-bg-tertiary border-l border-gray-700 w-64 shadow-xl"
            @click.stop
          >
            <div class="p-2 space-y-1">
              <button
                class="w-full flex items-center gap-3 px-3 py-3 text-left text-white hover:bg-gray-800 rounded transition-colors touch-manipulation"
                :class="
                  isOverlayVisible('saved-command-drawer') ? 'bg-gray-800' : ''
                "
                @click="handleMobileMenuClick('saved-command-drawer')"
              >
                <FileCode :size="18" />
                <span>Saved Commands</span>
              </button>

              <button
                class="w-full flex items-center gap-3 px-3 py-3 text-left text-white hover:bg-gray-800 rounded transition-colors touch-manipulation"
                :class="
                  isOverlayVisible('recordings-modal') ? 'bg-gray-800' : ''
                "
                @click="handleMobileMenuClick('recordings-modal')"
              >
                <Video :size="18" />
                <span>Session Recordings</span>
              </button>

              <button
                class="w-full flex items-center gap-3 px-3 py-3 text-left text-white hover:bg-gray-800 rounded transition-colors touch-manipulation"
                :class="
                  isOverlayVisible('tunnel-manager-modal') ? 'bg-gray-800' : ''
                "
                @click="handleMobileMenuClick('tunnel-manager-modal')"
              >
                <Route :size="18" />
                <span>SSH Tunnel Manager</span>
              </button>

              <button
                class="w-full flex items-center gap-3 px-3 py-3 text-left text-white hover:bg-gray-800 rounded transition-colors touch-manipulation"
                :class="
                  isOverlayVisible('ssh-key-manager-modal') ? 'bg-gray-800' : ''
                "
                @click="handleMobileMenuClick('ssh-key-manager-modal')"
              >
                <Key :size="18" />
                <span>SSH Key Manager</span>
              </button>

              <button
                class="w-full flex items-center gap-3 px-3 py-3 text-left text-white hover:bg-gray-800 rounded transition-colors touch-manipulation"
                :class="
                  isOverlayVisible('sync-manager-modal') ? 'bg-gray-800' : ''
                "
                @click="handleMobileMenuClick('sync-manager-modal')"
              >
                <Cloud :size="18" />
                <span>Sync Manager</span>
              </button>

              <button
                class="w-full flex items-center gap-3 px-3 py-3 text-left text-white hover:bg-gray-800 rounded transition-colors touch-manipulation"
                :class="
                  isOverlayVisible('master-password-settings')
                    ? 'bg-gray-800'
                    : ''
                "
                @click="handleMobileMenuClick('master-password-settings')"
              >
                <Shield :size="18" />
                <span>Master Password</span>
              </button>

              <button
                class="w-full flex items-center gap-3 px-3 py-3 text-left text-white hover:bg-gray-800 rounded transition-colors touch-manipulation"
                :class="
                  isOverlayVisible('theme-selector-modal') ? 'bg-gray-800' : ''
                "
                @click="handleMobileMenuClick('theme-selector-modal')"
              >
                <Palette :size="18" />
                <span>Terminal Theme</span>
              </button>

              <button
                class="w-full flex items-center gap-3 px-3 py-3 text-left text-white hover:bg-gray-800 rounded transition-colors touch-manipulation relative"
                :class="isOverlayVisible('updater-modal') ? 'bg-gray-800' : ''"
                @click="handleMobileMenuClick('updater-modal')"
              >
                <Download :size="18" />
                <span>Check for Updates</span>
                <span
                  v-if="updaterStore.hasUpdate"
                  class="ml-auto w-2.5 h-2.5 bg-red-500 rounded-full animate-pulse"
                />
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import {
  LayoutGrid,
  Server,
  Shield,
  Keyboard,
  Key,
  Route,
  Terminal,
  Cloud,
  Menu,
  Palette,
  Video,
  FolderOpen,
  FileCode,
  Archive,
  Download,
} from "lucide-vue-next";
import Button from "./ui/Button.vue";
import SyncStatusIndicator from "./sync/SyncStatusIndicator.vue";

import { useViewStateStore } from "../stores/viewState";
import { useUpdaterStore } from "../stores/updater";
import { useOverlay } from "../composables/useOverlay";
import { useWindowSize } from "../composables/useWindowSize";

const viewState = useViewStateStore();
const updaterStore = useUpdaterStore();
const { openOverlay, closeOverlay, closeAllOverlays, isOverlayVisible } =
  useOverlay();
const { isMobile } = useWindowSize();

const showMobileMenu = ref(false);

const setActiveView = (view: "dashboard" | "workspace" | "sftp") => {
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

const toggleMobileMenu = () => {
  showMobileMenu.value = !showMobileMenu.value;
};

const handleMobileMenuClick = (overlayName: string) => {
  showMobileMenu.value = false;
  toggleOverlay(overlayName);
};
</script>
