<template>
  <div class="h-full flex flex-col bg-[#0D0D0D] relative">
    <!-- Connecting Overlay -->
    <div
      v-if="sftpStore.connecting"
      class="absolute inset-0 bg-[#0D0D0D]/95 flex items-center justify-center z-50"
    >
      <div class="flex flex-col items-center space-y-4">
        <!-- Large spinning icon -->
        <div class="relative">
          <div
            class="animate-spin rounded-full h-12 w-12 border-2 border-gray-600 border-t-blue-400"
          ></div>
          <!-- Pulse effect -->
          <div
            class="absolute inset-0 animate-ping rounded-full h-12 w-12 border border-blue-400/20"
          ></div>
        </div>
        <!-- Loading text -->
        <div class="text-center">
          <p class="text-lg font-medium text-white mb-1">
            Connecting to SFTP...
          </p>
          <p class="text-sm text-gray-400">
            Please wait while establishing connection
          </p>
        </div>
      </div>
    </div>

    <!-- Header with connection selector -->
    <div
      class="flex items-center justify-between px-4 py-2 border-b border-gray-800"
    >
      <div class="flex items-center gap-3">
        <Select
          id="sftp-profile-select"
          v-model="selectedProfileId"
          :options="sshProfiles"
          placeholder="Select SSH Profile"
          :space="false"
          :disabled="sftpStore.connecting"
          @update:modelValue="handleProfileSelect"
        />
        <Button
          v-if="sftpStore.activeSessionId"
          variant="ghost"
          size="sm"
          :disabled="sftpStore.connecting"
          @click="handleDisconnect"
        >
          Disconnect
        </Button>
      </div>

      <div class="flex items-center gap-2">
        <!-- Sync & Compare button -->
        <Button
          v-if="sftpStore.activeSessionId"
          variant="ghost"
          size="sm"
          :icon="GitCompare"
          @click="handleSyncCompare"
          title="Sync & Compare Directories"
        >
          Sync
        </Button>

        <!-- Transfer button -->
        <Button
          variant="ghost"
          size="sm"
          :icon="Activity"
          @click="toggleTransferManager"
          title="Transfer Manager"
        >
          <span class="flex items-center gap-1.5">
            <span>Transfers</span>
            <span
              v-if="sftpStore.activeTransfers.length > 0"
              class="px-1.5 py-0.5 bg-blue-500/20 text-blue-400 rounded text-xs font-medium"
            >
              {{ sftpStore.activeTransfers.length }}
            </span>
          </span>
        </Button>
      </div>
    </div>

    <!-- Dual-pane layout -->
    <div v-if="sftpStore.activeSessionId" class="flex-1 overflow-hidden">
      <Splitpanes class="default-theme" @resize="onPaneResize">
        <Pane :size="50" :min-size="20">
          <!-- Local file browser -->
          <FileBrowser
            :files="sftpStore.browserState.localFiles"
            :current-path="sftpStore.browserState.localPath"
            :loading="sftpStore.browserState.loading.local"
            :selected-files="sftpStore.browserState.selectedLocalFiles"
            @navigate="handleLocalNavigate"
            @refresh="handleLocalRefresh"
            @select="handleLocalSelect"
            @upload="handleLocalUpload"
            @drag-files="handleLocalDragFiles"
            @open="handleLocalOpen"
            @edit="handleLocalEdit"
            @rename="handleLocalRename"
            @delete="handleLocalDelete"
            @create-directory="handleLocalCreateDirectory"
            @create-file="handleLocalCreateFile"
          />
        </Pane>

        <Pane :size="50" :min-size="20">
          <!-- Remote file browser -->
          <FileBrowser
            :files="sftpStore.browserState.remoteFiles"
            :current-path="sftpStore.browserState.remotePath"
            :loading="sftpStore.browserState.loading.remote"
            :is-remote="true"
            :selected-files="sftpStore.browserState.selectedRemoteFiles"
            @navigate="handleRemoteNavigate"
            @refresh="handleRemoteRefresh"
            @select="handleRemoteSelect"
            @download="handleRemoteDownload"
            @drag-files="handleRemoteDragFiles"
            @open="handleRemoteOpen"
            @edit="handleRemoteEdit"
            @rename="handleRemoteRename"
            @delete="handleRemoteDelete"
            @permissions="handleRemotePermissions"
            @create-directory="handleRemoteCreateDirectory"
            @create-file="handleRemoteCreateFile"
          />
        </Pane>
      </Splitpanes>
    </div>

    <!-- Empty state when not connected -->
    <div v-else class="flex-1 flex items-center justify-center">
      <EmptyState
        :icon="FolderOpen"
        title="No SFTP Connection"
        description="Select an SSH profile to connect"
      />
    </div>

    <!-- Modals -->
    <TransferManager />
    <FileRenameModal />
    <FileDeleteModal />
    <FilePermissionsModal />
    <CreateDirectoryModal />
    <CreateFileModal />
    <SyncCompareModal />
    <FileEditorModal />
    <FilePreviewModal />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { Splitpanes, Pane } from "splitpanes";
import "splitpanes/dist/splitpanes.css";
import { FolderOpen, Activity, GitCompare } from "lucide-vue-next";
import { useSFTPStore } from "../../stores/sftp";
import { useSSHStore } from "../../stores/ssh";
import { message } from "../../utils/message";
import { useOverlay } from "../../composables/useOverlay";
import FileBrowser from "./FileBrowser.vue";
import TransferManager from "./TransferManager.vue";
import FileRenameModal from "./FileRenameModal.vue";
import FileDeleteModal from "./FileDeleteModal.vue";
import FilePermissionsModal from "./FilePermissionsModal.vue";
import CreateDirectoryModal from "./CreateDirectoryModal.vue";
import CreateFileModal from "./CreateFileModal.vue";
import SyncCompareModal from "./SyncCompareModal.vue";
import FileEditorModal from "./FileEditorModal.vue";
import FilePreviewModal from "./FilePreviewModal.vue";
import Button from "../ui/Button.vue";
import Select from "../ui/Select.vue";
import EmptyState from "../ui/EmptyState.vue";
import type { FileEntry } from "../../types/sftp";
import { rename, remove, stat, writeFile, mkdir } from "@tauri-apps/plugin-fs";
import { dirname, homeDir, tempDir, join } from "@tauri-apps/api/path";
import { openPath } from "@tauri-apps/plugin-opener";
import { save } from "@tauri-apps/plugin-dialog";

const sftpStore = useSFTPStore();
const sshStore = useSSHStore();
const { openOverlay, closeOverlay, isOverlayVisible } = useOverlay();

const selectedProfileId = ref<string>("");

watch(
  () => sftpStore.activeTransfers.length,
  (newLength, oldLength) => {
    if (newLength > 0 && oldLength === 0) {
      if (!isOverlayVisible("sftp-transfer-manager-modal")) {
        openOverlay("sftp-transfer-manager-modal");
      }
    }
  },
);

function toggleTransferManager() {
  if (isOverlayVisible("sftp-transfer-manager-modal")) {
    closeOverlay("sftp-transfer-manager-modal");
  } else {
    openOverlay("sftp-transfer-manager-modal");
  }
}

function handleSyncCompare() {
  const localPath = sftpStore.browserState.localPath || "/";
  const remotePath = sftpStore.browserState.remotePath || "/";
  openOverlay("sftp-sync-compare-modal", {
    localPath,
    remotePath,
  });
}

const sshProfiles = computed(() => {
  return sshStore.profiles.map((p) => ({
    value: p.id,
    label: p.name,
  }));
});

onMounted(async () => {
  await sshStore.loadProfiles();

  window.addEventListener("sftp-rename", handleRenameSubmit);
  window.addEventListener("sftp-delete", handleDeleteSubmit);
  window.addEventListener("sftp-permissions", handlePermissionsSubmit);
  window.addEventListener("sftp-create-directory", handleCreateDirectorySubmit);
  window.addEventListener("sftp-create-file", handleCreateFileSubmit);
});

onUnmounted(() => {
  window.removeEventListener("sftp-rename", handleRenameSubmit);
  window.removeEventListener("sftp-delete", handleDeleteSubmit);
  window.removeEventListener("sftp-permissions", handlePermissionsSubmit);
  window.removeEventListener(
    "sftp-create-directory",
    handleCreateDirectorySubmit,
  );
  window.removeEventListener("sftp-create-file", handleCreateFileSubmit);
});

async function handleRenameSubmit(event: Event) {
  const customEvent = event as CustomEvent<{
    oldPath: string;
    newPath: string;
    isLocal?: boolean;
  }>;

  try {
    if (customEvent.detail.isLocal) {
      await rename(customEvent.detail.oldPath, customEvent.detail.newPath);

      const dirPath = await dirname(customEvent.detail.newPath);
      await sftpStore.listLocalDirectory(dirPath);
      message.success("File renamed successfully");
    } else {
      if (!sftpStore.activeSessionId) return;

      await sftpStore.renameFile(
        sftpStore.activeSessionId,
        customEvent.detail.oldPath,
        customEvent.detail.newPath,
      );
      message.success("File renamed successfully");
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error("Failed to rename file:", errorMessage);
    message.error(errorMessage);
  }
}

async function handleDeleteSubmit(event: Event) {
  const customEvent = event as CustomEvent<{
    path: string;
    isDirectory: boolean;
    isLocal?: boolean;
  }>;

  try {
    if (customEvent.detail.isLocal) {
      const fileStat = await stat(customEvent.detail.path);
      const isDirectory = fileStat.isDirectory;

      if (isDirectory) {
        await remove(customEvent.detail.path, { recursive: true });
      } else {
        await remove(customEvent.detail.path);
      }

      const dirPath = await dirname(customEvent.detail.path);
      await sftpStore.listLocalDirectory(dirPath);
      message.success("File deleted successfully");
    } else {
      if (!sftpStore.activeSessionId) return;

      await sftpStore.deleteFile(
        sftpStore.activeSessionId,
        customEvent.detail.path,
        customEvent.detail.isDirectory,
      );
      message.success("File deleted successfully");
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error("Failed to delete file:", errorMessage);
    message.error(errorMessage);
  }
}

async function handlePermissionsSubmit(event: Event) {
  const customEvent = event as CustomEvent<{
    path: string;
    mode: number;
  }>;
  if (!sftpStore.activeSessionId) return;

  await sftpStore.setPermissions(
    sftpStore.activeSessionId,
    customEvent.detail.path,
    customEvent.detail.mode,
  );
  message.success("Permissions updated successfully");
}

/**
 * Handle SSH profile selection and connect to SFTP
 * @param profileId - SSH profile ID to connect with
 */
async function handleProfileSelect(profileId: string) {
  if (!profileId || profileId === "") return;

  await sftpStore.connect(profileId);

  const homeDir = await getHomeDirectory();
  await sftpStore.listLocalDirectory(homeDir);

  if (sftpStore.activeSessionId) {
    await sftpStore.listRemoteDirectory(sftpStore.activeSessionId, "/");
  }

  message.success("SFTP connected successfully");
}

/**
 * Disconnect from SFTP
 */
async function handleDisconnect() {
  if (!sftpStore.activeSessionId) return;

  await sftpStore.disconnect(sftpStore.activeSessionId);
  message.success("SFTP disconnected");
  selectedProfileId.value = "";
}

async function getHomeDirectory(): Promise<string> {
  return await homeDir();
}

function handleLocalNavigate(path: string) {
  sftpStore.listLocalDirectory(path);
}

function handleLocalRefresh() {
  if (sftpStore.browserState.localPath) {
    sftpStore.listLocalDirectory(sftpStore.browserState.localPath);
  }
}

function handleLocalSelect(path: string) {
  const selected = sftpStore.browserState.selectedLocalFiles;
  if (selected.has(path)) {
    selected.delete(path);
  } else {
    selected.add(path);
  }
}

/**
 * Handle local file upload to remote SFTP server
 * @param files - FileList or File array to upload
 */
async function handleLocalUpload(files: FileList | File[]) {
  if (!sftpStore.activeSessionId) return;

  const remotePath = sftpStore.browserState.remotePath || "/";
  const fileArray = Array.from(files);
  const tempFilesToCleanup: string[] = [];

  try {
    for (const file of fileArray) {
      let filePath = "";
      const fileWithPath = file as File & { path?: string };
      if (fileWithPath.path) {
        filePath = fileWithPath.path;
      } else if (file instanceof File) {
        const tempDirPath = await tempDir();
        const tempFilePath = await join(
          tempDirPath,
          `sftp_upload_${Date.now()}_${file.name}`,
        );

        const arrayBuffer = await file.arrayBuffer();
        const uint8Array = new Uint8Array(arrayBuffer);

        await writeFile(tempFilePath, uint8Array);

        filePath = tempFilePath;
        tempFilesToCleanup.push(tempFilePath);
      } else {
        continue;
      }
      const normalizedRemotePath = remotePath.endsWith("/")
        ? remotePath.slice(0, -1)
        : remotePath;
      const remoteFilePath = `${normalizedRemotePath}/${file.name}`;

      await sftpStore.uploadFile(
        sftpStore.activeSessionId,
        filePath,
        remoteFilePath,
      );
      message.success(`Uploading ${file.name}...`);
    }
  } finally {
    if (tempFilesToCleanup.length > 0) {
      setTimeout(async () => {
        for (const tempPath of tempFilesToCleanup) {
          try {
            await remove(tempPath);
          } catch (error) {
            console.warn("Failed to cleanup temp file:", tempPath, error);
          }
        }
      }, 5000); // Cleanup after 5 seconds (enough time for upload to start)
    }
  }
}

/**
 * Check if a file can be previewed
 */
function canPreviewFile(file: FileEntry): boolean {
  if (file.fileType !== "file") return false;

  const previewableExtensions = new Set([
    // Images
    "jpg",
    "jpeg",
    "png",
    "gif",
    "bmp",
    "webp",
    "svg",
    "ico",
    // Videos
    "mp4",
    "webm",
    "ogg",
    "avi",
    "mov",
    "wmv",
    "flv",
    "mkv",
    // Documents
    "html",
    "htm",
    "md",
    "markdown",
    "pdf",
    // Office (show info, not actual preview)
    "doc",
    "docx",
    "xls",
    "xlsx",
    "ppt",
    "pptx",
    "odt",
    "ods",
    "odp",
  ]);

  const ext = file.name.split(".").pop()?.toLowerCase();
  return ext ? previewableExtensions.has(ext) : false;
}

async function handleLocalOpen(file: FileEntry) {
  if (file.fileType === "directory") {
    handleLocalNavigate(file.path);
  } else if (canPreviewFile(file)) {
    openOverlay("sftp-file-preview-modal", {
      file,
      isLocal: true,
    });
  } else {
    await openPath(file.path);
  }
}

function handleLocalEdit(file: FileEntry) {
  if (file.fileType === "file") {
    openOverlay("sftp-file-editor-modal", {
      file,
      isLocal: true,
    });
  }
}

function handleLocalRename(file: FileEntry) {
  openOverlay("sftp-file-rename-modal", {
    file,
    isLocal: true,
  });
}

function handleLocalDelete(file: FileEntry) {
  openOverlay("sftp-file-delete-modal", {
    file,
    isLocal: true,
  });
}

function handleRemoteNavigate(path: string) {
  if (!sftpStore.activeSessionId) return;
  sftpStore.listRemoteDirectory(sftpStore.activeSessionId, path);
}

function handleRemoteRefresh() {
  if (!sftpStore.activeSessionId || !sftpStore.browserState.remotePath) {
    return;
  }
  sftpStore.listRemoteDirectory(
    sftpStore.activeSessionId,
    sftpStore.browserState.remotePath,
  );
}

function handleRemoteSelect(path: string) {
  const selected = sftpStore.browserState.selectedRemoteFiles;
  if (selected.has(path)) {
    selected.delete(path);
  } else {
    selected.add(path);
  }
}

async function handleLocalDragFiles(
  files: FileEntry[],
  targetPath: string,
  isSourceRemote: boolean,
) {
  if (!sftpStore.activeSessionId) return;

  if (isSourceRemote) {
    const localPath = targetPath || sftpStore.browserState.localPath || "/";

    for (const file of files) {
      if (file.fileType === "directory") {
        message.warning(
          `Skipping directory: ${file.name} (directory drag not supported yet)`,
        );
        continue;
      }

      const localFilePath =
        localPath === "/"
          ? await join("/", file.name)
          : await join(localPath, file.name);

      await sftpStore.downloadFile(
        sftpStore.activeSessionId,
        file.path,
        localFilePath,
      );
      message.success(`Downloading ${file.name}...`);
    }
  }
}

async function handleRemoteDragFiles(
  files: FileEntry[],
  targetPath: string,
  isSourceRemote: boolean,
) {
  if (!sftpStore.activeSessionId) return;

  if (!isSourceRemote) {
    const remotePath = targetPath || sftpStore.browserState.remotePath || "/";

    for (const file of files) {
      if (file.fileType === "directory") {
        message.warning(
          `Skipping directory: ${file.name} (directory drag not supported yet)`,
        );
        continue;
      }

      const normalizedRemotePath = remotePath.endsWith("/")
        ? remotePath.slice(0, -1)
        : remotePath;
      const remoteFilePath = `${normalizedRemotePath}/${file.name}`;

      await sftpStore.uploadFile(
        sftpStore.activeSessionId,
        file.path,
        remoteFilePath,
      );
      message.success(`Uploading ${file.name}...`);
    }
  }
}

async function handleRemoteDownload(file: FileEntry) {
  if (!sftpStore.activeSessionId) return;

  const localPathResult = await save({
    defaultPath: file.name,
    filters: [
      {
        name: "All Files",
        extensions: ["*"],
      },
    ],
  });

  if (localPathResult) {
    await sftpStore.downloadFile(
      sftpStore.activeSessionId,
      file.path,
      localPathResult,
    );
    message.success("Download started");
  }
}

function handleRemoteOpen(file: FileEntry) {
  if (file.fileType === "directory") {
    handleRemoteNavigate(file.path);
  } else if (canPreviewFile(file)) {
    openOverlay("sftp-file-preview-modal", {
      file,
      isLocal: false,
    });
  } else {
    message.info("Please download the file to open it");
  }
}

function handleRemoteEdit(file: FileEntry) {
  if (file.fileType === "file") {
    openOverlay("sftp-file-editor-modal", {
      file,
      isLocal: false,
    });
  }
}

function handleRemoteRename(file: FileEntry) {
  openOverlay("sftp-file-rename-modal", { file });
}

function handleRemoteDelete(file: FileEntry) {
  openOverlay("sftp-file-delete-modal", { file });
}

function handleRemotePermissions(file: FileEntry) {
  openOverlay("sftp-file-permissions-modal", { file });
}

function handleLocalCreateDirectory() {
  const currentPath = sftpStore.browserState.localPath || "/";
  openOverlay("sftp-create-directory-modal", { currentPath, isLocal: true });
}

function handleRemoteCreateDirectory() {
  const currentPath = sftpStore.browserState.remotePath || "/";
  openOverlay("sftp-create-directory-modal", { currentPath, isLocal: false });
}

async function handleCreateDirectorySubmit(event: Event) {
  const customEvent = event as CustomEvent<{
    path: string;
    name: string;
    isLocal?: boolean;
  }>;

  try {
    if (customEvent.detail.isLocal) {
      const directoryPath =
        customEvent.detail.path === "/"
          ? `/${customEvent.detail.name}`
          : await join(customEvent.detail.path, customEvent.detail.name);

      await mkdir(directoryPath);

      await sftpStore.listLocalDirectory(customEvent.detail.path);
      message.success("Directory created successfully");
    } else {
      if (!sftpStore.activeSessionId) return;

      const directoryPath =
        customEvent.detail.path === "/"
          ? `/${customEvent.detail.name}`
          : `${customEvent.detail.path}/${customEvent.detail.name}`;
      await sftpStore.createDirectory(sftpStore.activeSessionId, directoryPath);
      message.success("Directory created successfully");
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error("Failed to create directory:", errorMessage);
    message.error(errorMessage);
  }
}

function handleLocalCreateFile() {
  const currentPath = sftpStore.browserState.localPath || "/";
  openOverlay("sftp-create-file-modal", { currentPath, isLocal: true });
}

function handleRemoteCreateFile() {
  const currentPath = sftpStore.browserState.remotePath || "/";
  openOverlay("sftp-create-file-modal", { currentPath, isLocal: false });
}

async function handleCreateFileSubmit(event: Event) {
  const customEvent = event as CustomEvent<{
    path: string;
    name: string;
    isLocal?: boolean;
  }>;

  try {
    if (customEvent.detail.isLocal) {
      const filePath =
        customEvent.detail.path === "/"
          ? `/${customEvent.detail.name}`
          : await join(customEvent.detail.path, customEvent.detail.name);

      await writeFile(filePath, new Uint8Array());

      await sftpStore.listLocalDirectory(customEvent.detail.path);
      message.success("File created successfully");
    } else {
      if (!sftpStore.activeSessionId) return;

      const filePath =
        customEvent.detail.path === "/"
          ? `/${customEvent.detail.name}`
          : `${customEvent.detail.path}/${customEvent.detail.name}`;

      const tempDirPath = await tempDir();
      const tempFilePath = await join(
        tempDirPath,
        `sftp_temp_${Date.now()}_${customEvent.detail.name}`,
      );

      await writeFile(tempFilePath, new Uint8Array());

      await sftpStore.uploadFile(
        sftpStore.activeSessionId,
        tempFilePath,
        filePath,
      );

      await remove(tempFilePath).catch(() => {});

      message.success("File created successfully");
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error("Failed to create file:", errorMessage);
    message.error(errorMessage);
  }
}

function onPaneResize() {}
</script>
