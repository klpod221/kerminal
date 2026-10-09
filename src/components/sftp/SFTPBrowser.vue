<!--
  Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <div class="relative flex h-full flex-col bg-bg-primary font-mono text-xs text-gray-200">
    <!-- Top Header Bar -->
    <header class="flex items-center justify-between border-b border-gray-800 bg-bg-secondary px-4 py-2">
      <!-- Connection controls -->
      <div class="flex items-center gap-3">
        <div class="w-64">
          <Select
            id="sftp-profile-select"
            v-model="selectedProfileId"
            :options="profileOptions"
            placeholder="Select SSH Profile..."
            :space="false"
            :disabled="session.connecting.value"
          />
        </div>

        <Button
          v-if="!session.sessionId.value"
          variant="primary"
          size="sm"
          :disabled="!selectedProfileId || session.connecting.value"
          :loading="session.connecting.value"
          @click="handleConnect"
        >
          <span>{{ session.connecting.value ? 'Connecting...' : 'Connect' }}</span>
        </Button>
        <Button v-else variant="danger" size="sm" @click="handleDisconnect">
          <span>Disconnect</span>
        </Button>
      </div>

      <!-- Center Quick Transfer between panes -->
      <div v-if="session.sessionId.value" class="hidden md:flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          :icon="Upload"
          :disabled="localExplorer.selectedPaths.value.size === 0"
          title="Upload selected files from Local to Server"
          @click="queueUploads(Array.from(localExplorer.selectedPaths.value))"
        >
          <span>Upload</span>
          <span v-if="localExplorer.selectedPaths.value.size > 0" class="ml-1 text-cyan-400">
            ({{ localExplorer.selectedPaths.value.size }})
          </span>
        </Button>

        <Button
          variant="outline"
          size="sm"
          :icon="Download"
          :disabled="remoteExplorer.selectedPaths.value.size === 0"
          title="Download selected files from Server to Local"
          @click="queueDownloads(Array.from(remoteExplorer.selectedPaths.value))"
        >
          <span>Download</span>
          <span v-if="remoteExplorer.selectedPaths.value.size > 0" class="ml-1 text-purple-400">
            ({{ remoteExplorer.selectedPaths.value.size }})
          </span>
        </Button>
      </div>

      <!-- Right Utility Actions -->
      <div class="flex items-center gap-2">
        <Button
          v-if="session.sessionId.value"
          variant="ghost"
          size="sm"
          :icon="GitCompare"
          title="Compare & Sync directories"
          @click="openOverlay('sftp-sync-compare-modal', { sessionId: session.sessionId.value, remotePath: remoteExplorer.currentPath.value, localPath: localExplorer.currentPath.value })"
        >
          <span class="hidden sm:inline">Sync</span>
        </Button>

        <Button
          v-if="session.sessionId.value"
          variant="ghost"
          size="sm"
          :icon="Search"
          title="Search files on server"
          @click="openOverlay('sftp-file-search-modal', { sessionId: session.sessionId.value, path: remoteExplorer.currentPath.value })"
        >
          <span class="hidden sm:inline">Search</span>
        </Button>

        <Button
          variant="ghost"
          size="sm"
          :icon="ArrowUpDown"
          title="Transfer Queue"
          @click="transfers.isDrawerOpen.value = !transfers.isDrawerOpen.value"
        >
          <span class="hidden sm:inline">Transfers</span>
          <span
            v-if="transfers.activeCount.value > 0"
            class="ml-1.5 rounded-full bg-blue-500/20 px-1.5 py-0.2 text-2xs text-blue-400 border border-blue-500/30"
          >
            {{ transfers.activeCount.value }}
          </span>
        </Button>
      </div>
    </header>

    <!-- Error Banner -->
    <div
      v-if="session.error.value"
      class="flex items-center justify-between border-b border-red-500/20 bg-red-500/10 px-4 py-2 text-xs text-red-400"
    >
      <div class="flex items-center gap-2">
        <AlertTriangle class="h-4 w-4 shrink-0 text-red-400" />
        <span>{{ session.error.value }}</span>
      </div>
      <button class="text-red-400 hover:text-white" @click="session.error.value = null">×</button>
    </div>

    <!-- Main Body Area -->
    <main class="relative flex-1 overflow-hidden">
      <!-- Disconnected State -->
      <div v-if="!session.sessionId.value" class="flex h-full items-center justify-center p-6 bg-bg-primary">
        <EmptyState
          :icon="Server"
          title="SFTP Disconnected"
          description="Select an SSH profile above and click Connect to start managing local and remote files."
        />
      </div>

      <!-- Connected State: Dual-Pane Layout -->
      <Splitpanes v-else class="default-theme h-full w-full">
        <!-- Left Pane: Local File Browser -->
        <Pane :size="50" :min-size="20" class="flex flex-col h-full bg-bg-primary overflow-hidden">
          <PaneToolbar
            :current-path="localExplorer.currentPath.value"
            :loading="localExplorer.loading.value"
            :can-go-back="localExplorer.canGoBack.value"
            :can-go-forward="localExplorer.canGoForward.value"
            :show-hidden="localExplorer.showHidden.value"
            :search-query="localExplorer.searchQuery.value"
            :is-remote="false"
            :selected-count="localExplorer.selectedPaths.value.size"
            @back="localExplorer.goBack()"
            @forward="localExplorer.goForward()"
            @up="localExplorer.goUp()"
            @refresh="localExplorer.loadDirectory(localExplorer.currentPath.value)"
            @navigate="localExplorer.navigateTo($event)"
            @new-folder="openCreateModal(true, 'dir')"
            @new-file="openCreateModal(true, 'file')"
            @edit="handleToolbarAction('edit', true)"
            @rename="handleToolbarAction('rename', true)"
            @delete="handleToolbarAction('delete', true)"
            @transfer-action="queueUploads(Array.from(localExplorer.selectedPaths.value))"
            @toggle-hidden="localExplorer.showHidden.value = !localExplorer.showHidden.value"
            @update:search-query="localExplorer.searchQuery.value = $event"
          />

          <div class="flex-1 overflow-hidden">
            <FileListTable
              :entries="localExplorer.filteredEntries.value"
              :loading="localExplorer.loading.value"
              :error="localExplorer.error.value"
              :selected-paths="localExplorer.selectedPaths.value"
              :sort-field="localExplorer.sortField.value"
              :sort-asc="localExplorer.sortAsc.value"
              :is-remote="false"
              @open="handleLocalOpen"
              @refresh="localExplorer.loadDirectory(localExplorer.currentPath.value)"
              @sort="localExplorer.setSort"
              @toggle-select="localExplorer.toggleSelect"
              @toggle-select-all="localExplorer.selectAll"
              @clear-selection="localExplorer.clearSelection()"
              @action="(act, item) => handleAction(act, item, true)"
              @drop-transfer="(d) => { if (d.isRemoteSource) queueDownloads(d.paths); }"
            />
          </div>
        </Pane>

        <!-- Right Pane: Remote SFTP Browser -->
        <Pane :size="50" :min-size="20" class="flex flex-col h-full bg-bg-primary overflow-hidden">
          <PaneToolbar
            :current-path="remoteExplorer.currentPath.value"
            :loading="remoteExplorer.loading.value"
            :can-go-back="remoteExplorer.canGoBack.value"
            :can-go-forward="remoteExplorer.canGoForward.value"
            :show-hidden="remoteExplorer.showHidden.value"
            :search-query="remoteExplorer.searchQuery.value"
            :is-remote="true"
            :selected-count="remoteExplorer.selectedPaths.value.size"
            @back="remoteExplorer.goBack()"
            @forward="remoteExplorer.goForward()"
            @up="remoteExplorer.goUp()"
            @refresh="remoteExplorer.loadDirectory(remoteExplorer.currentPath.value)"
            @navigate="remoteExplorer.navigateTo($event)"
            @new-folder="openCreateModal(false, 'dir')"
            @new-file="openCreateModal(false, 'file')"
            @edit="handleToolbarAction('edit', false)"
            @rename="handleToolbarAction('rename', false)"
            @delete="handleToolbarAction('delete', false)"
            @transfer-action="queueDownloads(Array.from(remoteExplorer.selectedPaths.value))"
            @toggle-hidden="remoteExplorer.showHidden.value = !remoteExplorer.showHidden.value"
            @update:search-query="remoteExplorer.searchQuery.value = $event"
          />

          <div class="flex-1 overflow-hidden">
            <FileListTable
              :entries="remoteExplorer.filteredEntries.value"
              :loading="remoteExplorer.loading.value"
              :error="remoteExplorer.error.value"
              :selected-paths="remoteExplorer.selectedPaths.value"
              :sort-field="remoteExplorer.sortField.value"
              :sort-asc="remoteExplorer.sortAsc.value"
              :is-remote="true"
              @open="handleRemoteOpen"
              @refresh="remoteExplorer.loadDirectory(remoteExplorer.currentPath.value)"
              @sort="remoteExplorer.setSort"
              @toggle-select="remoteExplorer.toggleSelect"
              @toggle-select-all="remoteExplorer.selectAll"
              @clear-selection="remoteExplorer.clearSelection()"
              @action="(act, item) => handleAction(act, item, false)"
              @drop-transfer="(d) => { if (!d.isRemoteSource) queueUploads(d.paths); }"
            />
          </div>
        </Pane>
      </Splitpanes>
    </main>

    <!-- Bottom Transfer Drawer -->
    <TransferDrawer
      :is-open="transfers.isDrawerOpen.value"
      :transfers="transfers.transferList.value"
      @close="transfers.isDrawerOpen.value = false"
      @refresh="transfers.refreshQueue"
      @pause-all="transfers.pauseAll"
      @resume-all="transfers.resumeAll"
      @clear-completed="transfers.clearCompleted"
      @pause="transfers.pauseTask"
      @resume="transfers.resumeTask"
      @cancel="transfers.cancelTask"
      @retry="transfers.retryTask"
    />

    <!-- Dialogs & Modals -->
    <CreateDirectoryModal />
    <CreateFileModal />
    <FileRenameModal />
    <FileDeleteModal />
    <FilePermissionsModal />
    <FileEditorModal />
    <FilePreviewModal />
    <FileSearchModal />
    <SyncCompareModal />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { Splitpanes, Pane } from "splitpanes";
import { useSSHStore } from "../../stores/ssh";
import { useOverlay } from "../../composables/useOverlay";
import {
  useSftpSession,
  useFileExplorer,
  useFileOperations,
  useTransferQueue,
} from "../../composables/sftp";
import { listSFTPDirectory } from "../../services/sftp";
import { listLocalDirectory, getDefaultLocalPath } from "../../services/localFs";
import type { FileEntry } from "../../types/sftp";

import Button from "../ui/Button.vue";
import Select from "../ui/Select.vue";
import EmptyState from "../ui/EmptyState.vue";
import PaneToolbar from "./PaneToolbar.vue";
import FileListTable from "./FileListTable.vue";
import TransferDrawer from "./TransferDrawer.vue";
import CreateDirectoryModal from "./CreateDirectoryModal.vue";
import CreateFileModal from "./CreateFileModal.vue";
import FileRenameModal from "./FileRenameModal.vue";
import FileDeleteModal from "./FileDeleteModal.vue";
import FilePermissionsModal from "./FilePermissionsModal.vue";
import FileEditorModal from "./FileEditorModal.vue";
import FilePreviewModal from "./FilePreviewModal.vue";
import FileSearchModal from "./FileSearchModal.vue";
import SyncCompareModal from "./SyncCompareModal.vue";
import { message } from "../../utils/message";
import { Server, Upload, Download, GitCompare, Search, ArrowUpDown, AlertTriangle } from "lucide-vue-next";

const sshStore = useSSHStore();
const { openOverlay } = useOverlay();
const session = useSftpSession();
const ops = useFileOperations();
const transfers = useTransferQueue();

const selectedProfileId = ref<string>("");
const profileOptions = computed(() =>
  sshStore.profiles.map((p) => ({
    value: p.id,
    label: `${p.name} (${p.username}@${p.host})`,
  })),
);

// File Explorers: Local and Remote
const localExplorer = useFileExplorer({
  fetcher: listLocalDirectory,
  initialPath: "/",
});

const remoteExplorer = useFileExplorer({
  fetcher: (path: string) => listSFTPDirectory(session.sessionId.value!, path),
  initialPath: "/",
});

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

const modalEvents = ["sftp-create-directory", "sftp-create-file", "sftp-rename", "sftp-delete"];
let unlistenTransferComplete: UnlistenFn | null = null;

onMounted(async () => {
  if (sshStore.profiles.length === 0) await sshStore.loadProfiles();
  const defaultLocal = await getDefaultLocalPath();
  await localExplorer.loadDirectory(defaultLocal);
  modalEvents.forEach((evt) => globalThis.addEventListener(evt, handleModalEvent));
  unlistenTransferComplete = await listen("sftp_transfer_complete", () => {
    localExplorer.loadDirectory(localExplorer.currentPath.value);
    if (session.sessionId.value) {
      remoteExplorer.loadDirectory(remoteExplorer.currentPath.value);
    }
  });
});

onUnmounted(() => {
  if (unlistenTransferComplete) unlistenTransferComplete();
  modalEvents.forEach((evt) => globalThis.removeEventListener(evt, handleModalEvent));
});

async function handleConnect() {
  if (!selectedProfileId.value) return;
  const ok = await session.connect(selectedProfileId.value);
  if (ok && session.sessionId.value) {
    await remoteExplorer.loadDirectory(session.homeDir.value || "/");
    transfers.startPolling();
  }
}

async function handleDisconnect() {
  await session.disconnect();
  remoteExplorer.clearSelection();
  transfers.stopPolling();
}

async function handleLocalOpen(item: FileEntry) {
  if (item.fileType === "directory") {
    await localExplorer.navigateTo(item.path);
  } else {
    try {
      const { openPath } = await import("@tauri-apps/plugin-opener");
      await openPath(item.path);
    } catch {
      openOverlay("sftp-file-editor-modal", { file: item, isLocal: true });
    }
  }
}

const TEXT_EXTS = new Set(["txt", "md", "log", "json", "js", "ts", "vue", "jsx", "tsx", "rs", "go", "py", "c", "cpp", "h", "hpp", "html", "css", "scss", "yaml", "yml", "sh", "bash", "zsh", "toml", "ini", "conf", "env", "sql", "xml", "csv", "dockerfile", "makefile", "gitignore"]);
const MEDIA_EXTS = new Set(["jpg", "jpeg", "png", "gif", "svg", "webp", "mp4", "webm", "mp3", "wav", "ogg"]);

async function handleRemoteOpen(item: FileEntry) {
  if (item.fileType === "directory") {
    await remoteExplorer.navigateTo(item.path);
    return;
  }
  const ext = item.name.split(".").pop()?.toLowerCase() || "";
  if (MEDIA_EXTS.has(ext)) {
    openOverlay("sftp-file-preview-modal", { file: item, sessionId: session.sessionId.value });
  } else if (TEXT_EXTS.has(ext) || (!ext && (item.size || 0) < 500000)) {
    openOverlay("sftp-file-editor-modal", { file: item, sessionId: session.sessionId.value, isLocal: false });
  } else {
    // Binary, PDF, archive -> download to local current path
    queueDownloads([item.path]);
  }
}

function openCreateModal(isLocal: boolean, type: "dir" | "file") {
  const currentPath = isLocal ? localExplorer.currentPath.value : remoteExplorer.currentPath.value;
  const modalId = type === "dir" ? "sftp-create-directory-modal" : "sftp-create-file-modal";
  openOverlay(modalId, { currentPath, isLocal });
}

function handleToolbarAction(action: "edit" | "rename" | "delete", isLocal: boolean) {
  const explorer = isLocal ? localExplorer : remoteExplorer;
  const paths = Array.from(explorer.selectedPaths.value);
  if (paths.length === 0) return;
  const items = explorer.filteredEntries.value.filter((e) => explorer.selectedPaths.value.has(e.path));
  if (items.length === 0) return;
  if (action === "edit" && items.length === 1 && items[0].fileType !== "directory") {
    openOverlay("sftp-file-editor-modal", { file: items[0], sessionId: session.sessionId.value, isLocal });
  } else if (action === "rename" && items.length === 1) {
    openOverlay("sftp-file-rename-modal", { file: items[0], isLocal });
  } else if (action === "delete") {
    openOverlay("sftp-file-delete-modal", { file: items[0], files: items, isLocal });
  }
}

function handleAction(action: string, item: FileEntry, isLocal: boolean) {
  const explorer = isLocal ? localExplorer : remoteExplorer;
  const isMulti = explorer.selectedPaths.value.size > 1 && explorer.selectedPaths.value.has(item?.path);
  const targetPaths = isMulti ? Array.from(explorer.selectedPaths.value) : (item?.path ? [item.path] : []);

  if (action === "upload") queueUploads(targetPaths);
  else if (action === "download") queueDownloads(targetPaths);
  else if (action === "open-system") handleLocalOpen(item);
  else if (action === "edit") openOverlay("sftp-file-editor-modal", { file: item, sessionId: session.sessionId.value, isLocal });
  else if (action === "preview") openOverlay("sftp-file-preview-modal", { file: item, sessionId: session.sessionId.value });
  else if (action === "permissions") openOverlay("sftp-file-permissions-modal", { file: item, sessionId: session.sessionId.value });
  else if (action === "rename") openOverlay("sftp-file-rename-modal", { file: item, isLocal });
  else if (action === "delete") {
    const selected = isMulti ? explorer.filteredEntries.value.filter((e) => explorer.selectedPaths.value.has(e.path)) : (item ? [item] : []);
    openOverlay("sftp-file-delete-modal", { file: item, files: selected, isLocal });
  } else if (action === "copy-path") {
    navigator.clipboard.writeText(targetPaths.join("\n"));
    message.success(isMulti ? `Copied ${targetPaths.length} paths` : "Copied path");
  } else if (action === "new-folder-current") openCreateModal(isLocal, "dir");
  else if (action === "new-file-current") openCreateModal(isLocal, "file");
  else if (action === "new-folder-inside" && item) openOverlay("sftp-create-directory-modal", { currentPath: item.path, isLocal });
  else if (action === "new-file-inside" && item) openOverlay("sftp-create-file-modal", { currentPath: item.path, isLocal });
}

function queueUploads(localPaths: string[]) {
  if (!session.sessionId.value) return;
  const remoteTargetDir = remoteExplorer.currentPath.value.replace(/\/+$/, "");
  for (const p of localPaths) {
    const name = p.split(/[/\\]/).pop() || "upload";
    transfers.addUploadTask(session.sessionId.value, p, `${remoteTargetDir}/${name}`);
  }
  transfers.isDrawerOpen.value = true;
}

function queueDownloads(remotePaths: string[]) {
  if (!session.sessionId.value) return;
  const localTargetDir = localExplorer.currentPath.value.replace(/\/+$/, "");
  for (const p of remotePaths) {
    const name = p.split(/[/\\]/).pop() || "download";
    transfers.addDownloadTask(session.sessionId.value, p, `${localTargetDir}/${name}`);
  }
  transfers.isDrawerOpen.value = true;
}

async function handleModalEvent(event: Event) {
  const e = event as CustomEvent;
  const { isLocal, path, name, oldPath, newPath, isDirectory } = e.detail;
  if (e.type === "sftp-create-directory") {
    const target = `${path.replace(/\/+$/, "")}/${name}`;
    if (isLocal) await ops.createLocalDir(target);
    else if (session.sessionId.value) await ops.createDirectory(session.sessionId.value, target);
  } else if (e.type === "sftp-create-file") {
    const target = `${path.replace(/\/+$/, "")}/${name}`;
    if (isLocal) await ops.createLocalNewFile(target, "");
    else if (session.sessionId.value) await ops.createFile(session.sessionId.value, target, "");
  } else if (e.type === "sftp-rename") {
    if (isLocal) await ops.renameLocal(oldPath, newPath);
    else if (session.sessionId.value) await ops.renameItem(session.sessionId.value, oldPath, newPath);
  } else if (e.type === "sftp-delete") {
    if (isLocal) await ops.deleteLocalItems([path]);
    else if (session.sessionId.value) await ops.deleteItems(session.sessionId.value, [path], isDirectory);
  }

  if (isLocal) await localExplorer.loadDirectory(localExplorer.currentPath.value);
  else if (session.sessionId.value) await remoteExplorer.loadDirectory(remoteExplorer.currentPath.value);
}
</script>

<style scoped>
@import "splitpanes/dist/splitpanes.css";

:deep(.splitpanes__splitter) {
  background-color: #1a1a1a;
  border-left: 1px solid #262626;
  border-right: 1px solid #262626;
  position: relative;
  width: 5px;
  cursor: col-resize;
  transition: background-color 0.2s;
}

:deep(.splitpanes__splitter:hover) {
  background-color: #3b82f6;
}
</style>
