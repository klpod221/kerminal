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
    <div class="flex items-center justify-between px-4 py-2 border-b border-gray-800">
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
      <Splitpanes
        class="default-theme"
        @resize="onPaneResize"
      >
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
    <div
      v-else
      class="flex-1 flex items-center justify-center"
    >
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
import { getErrorMessage } from "../../utils/helpers";
import { useOverlay } from "../../composables/useOverlay";
import FileBrowser from "./FileBrowser.vue";
import TransferManager from "./TransferManager.vue";
import FileRenameModal from "./FileRenameModal.vue";
import FileDeleteModal from "./FileDeleteModal.vue";
import FilePermissionsModal from "./FilePermissionsModal.vue";
import CreateDirectoryModal from "./CreateDirectoryModal.vue";
import CreateFileModal from "./CreateFileModal.vue";
import SyncCompareModal from "./SyncCompareModal.vue";
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

// Watch for transfers and auto-open modal if new transfers start
watch(
  () => sftpStore.activeTransfers.length,
  (newLength, oldLength) => {
    // Auto-open modal when first transfer starts
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

  // Listen for modal events
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
  window.removeEventListener("sftp-create-directory", handleCreateDirectorySubmit);
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
      // Rename local file using Tauri fs API
      await rename(customEvent.detail.oldPath, customEvent.detail.newPath);

      // Refresh directory
      const dirPath = await dirname(customEvent.detail.newPath);
      await sftpStore.listLocalDirectory(dirPath);
      message.success("File renamed successfully");
    } else {
      // Rename remote file
      if (!sftpStore.activeSessionId) return;

      await sftpStore.renameFile(
        sftpStore.activeSessionId,
        customEvent.detail.oldPath,
        customEvent.detail.newPath,
      );
      message.success("File renamed successfully");
    }
  } catch (error) {
    console.error("Failed to rename file:", error);
    message.error(
      getErrorMessage(error, "Failed to rename file"),
    );
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
      // Delete local file using Tauri fs API
      // Check if it's a directory
      const fileStat = await stat(customEvent.detail.path);
      const isDirectory = fileStat.isDirectory;

      // Delete file or directory
      if (isDirectory) {
        await remove(customEvent.detail.path, { recursive: true });
      } else {
        await remove(customEvent.detail.path);
      }

      // Refresh directory
      const dirPath = await dirname(customEvent.detail.path);
      await sftpStore.listLocalDirectory(dirPath);
      message.success("File deleted successfully");
    } else {
      // Delete remote file
      if (!sftpStore.activeSessionId) return;

      await sftpStore.deleteFile(
        sftpStore.activeSessionId,
        customEvent.detail.path,
        customEvent.detail.isDirectory,
      );
      message.success("File deleted successfully");
    }
  } catch (error) {
    console.error("Failed to delete file:", error);
    message.error(
      getErrorMessage(error, "Failed to delete file"),
    );
  }
}

async function handlePermissionsSubmit(event: Event) {
  const customEvent = event as CustomEvent<{
    path: string;
    mode: number;
  }>;
  if (!sftpStore.activeSessionId) return;

  try {
    await sftpStore.setPermissions(
      sftpStore.activeSessionId,
      customEvent.detail.path,
      customEvent.detail.mode,
    );
    message.success("Permissions updated successfully");
  } catch (error) {
    console.error("Failed to update permissions:", error);
    message.error(
      getErrorMessage(error, "Failed to update permissions"),
    );
  }
}

async function handleProfileSelect(profileId: string) {
  if (!profileId || profileId === "") return;

  try {
    await sftpStore.connect(profileId);

    // Initialize paths
    const homeDir = await getHomeDirectory();
    await sftpStore.listLocalDirectory(homeDir);

    if (sftpStore.activeSessionId) {
      await sftpStore.listRemoteDirectory(
        sftpStore.activeSessionId,
        "/",
      );
    }

    message.success("SFTP connected successfully");
  } catch (error) {
    console.error("Failed to connect SFTP:", error);
    message.error(
      getErrorMessage(error, "Failed to connect SFTP"),
    );
  }
}

async function handleDisconnect() {
  if (!sftpStore.activeSessionId) return;

  try {
    await sftpStore.disconnect(sftpStore.activeSessionId);
    message.success("SFTP disconnected");
    selectedProfileId.value = "";
  } catch (error) {
    console.error("Failed to disconnect SFTP:", error);
    message.error("Failed to disconnect SFTP");
  }
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

async function handleLocalUpload(files: FileList | File[]) {
  if (!sftpStore.activeSessionId) return;

  const remotePath = sftpStore.browserState.remotePath || "/";
  const fileArray = Array.from(files);
  const tempFilesToCleanup: string[] = [];

  try {
    for (const file of fileArray) {
      try {
        let filePath = "";

        // Try to get path from file object (Tauri adds path property for desktop drag & drop)
        if ("path" in file && (file as any).path) {
          filePath = (file as any).path;
        } else if (file instanceof File) {
          // For browser File API, save to temporary directory first
          // Get temp directory
          const tempDirPath = await tempDir();
          const tempFilePath = await join(
            tempDirPath,
            `sftp_upload_${Date.now()}_${file.name}`,
          );

          // Read file as array buffer
          const arrayBuffer = await file.arrayBuffer();
          const uint8Array = new Uint8Array(arrayBuffer);

          // Write to temp file (writeFile accepts Uint8Array for binary)
          await writeFile(tempFilePath, uint8Array);

          filePath = tempFilePath;
          tempFilesToCleanup.push(tempFilePath);
        } else {
          continue;
        }

        // Upload to remote path with same filename
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
      } catch (error) {
        console.error("Failed to upload file:", error);
        message.error(
          getErrorMessage(error, `Failed to upload ${file.name}`),
        );
      }
    }
  } finally {
    // Cleanup temp files after a delay (give time for upload to start)
    // We don't delete immediately because upload happens asynchronously
    if (tempFilesToCleanup.length > 0) {
      setTimeout(async () => {
        for (const tempPath of tempFilesToCleanup) {
          try {
            await remove(tempPath);
          } catch (error) {
            // Ignore cleanup errors
            console.warn("Failed to cleanup temp file:", tempPath, error);
          }
        }
      }, 5000); // Cleanup after 5 seconds (enough time for upload to start)
    }
  }
}

async function handleLocalOpen(file: FileEntry) {
  // If it's a directory, navigate to it (same as click)
  if (file.fileType === "directory") {
    handleLocalNavigate(file.path);
  } else {
    // Open local file with default application
    await openPath(file.path);
  }
}

function handleLocalRename(file: FileEntry) {
  // Use the same rename modal as remote files for consistency
  openOverlay("sftp-file-rename-modal", {
    file,
    isLocal: true,
  });
}

function handleLocalDelete(file: FileEntry) {
  // Use the same delete modal as remote files for consistency
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
  if (
    !sftpStore.activeSessionId ||
    !sftpStore.browserState.remotePath
  ) {
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
  // This handler is called by the local browser when files are dropped
  // If source is local (isSourceRemote=false), it means drag within local browser (ignore or handle move)
  // If source is remote (isSourceRemote=true), it means drag from remote to local (download)
  if (!sftpStore.activeSessionId) return;

  // If source is remote, download files to local
  if (isSourceRemote) {
    const localPath = targetPath || sftpStore.browserState.localPath || "/";

    for (const file of files) {
      try {
        if (file.fileType === "directory") {
          message.warning(`Skipping directory: ${file.name} (directory drag not supported yet)`);
          continue;
        }

        const localFilePath = localPath === "/"
          ? await join("/", file.name)
          : await join(localPath, file.name);

        await sftpStore.downloadFile(
          sftpStore.activeSessionId,
          file.path,
          localFilePath,
        );
        message.success(`Downloading ${file.name}...`);
      } catch (error) {
        console.error("Failed to download file:", error);
        message.error(
          getErrorMessage(error, `Failed to download ${file.name}`),
        );
      }
    }
  }
  // If source is local and dropped on local browser, ignore (same browser)
}

async function handleRemoteDragFiles(
  files: FileEntry[],
  targetPath: string,
  isSourceRemote: boolean,
) {
  // This handler is called by the remote browser when files are dropped
  // If source is remote (isSourceRemote=true), it means drag within remote browser (ignore or handle move)
  // If source is local (isSourceRemote=false), it means drag from local to remote (upload)
  if (!sftpStore.activeSessionId) return;

  // If source is local, upload files to remote
  if (!isSourceRemote) {
    const remotePath = targetPath || sftpStore.browserState.remotePath || "/";

    for (const file of files) {
      try {
        if (file.fileType === "directory") {
          message.warning(`Skipping directory: ${file.name} (directory drag not supported yet)`);
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
      } catch (error) {
        console.error("Failed to upload file:", error);
        message.error(
          getErrorMessage(error, `Failed to upload ${file.name}`),
        );
      }
    }
  }
  // If source is remote and dropped on remote browser, ignore (same browser)
}

async function handleRemoteDownload(file: FileEntry) {
  if (!sftpStore.activeSessionId) return;

  try {
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
  } catch (error) {
    console.error("Failed to download file:", error);
    message.error(
      getErrorMessage(error, "Failed to download file"),
    );
  }
}

function handleRemoteOpen(file: FileEntry) {
  // For remote files, we can't open directly - would need to download first
  if (file.fileType === "directory") {
    handleRemoteNavigate(file.path);
  } else {
    message.info("Please download the file to open it");
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
      // Create local directory using Tauri fs API
      const directoryPath =
        customEvent.detail.path === "/"
          ? `/${customEvent.detail.name}`
          : await join(customEvent.detail.path, customEvent.detail.name);

      await mkdir(directoryPath);

      // Refresh directory
      await sftpStore.listLocalDirectory(customEvent.detail.path);
      message.success("Directory created successfully");
    } else {
      // Create remote directory
      if (!sftpStore.activeSessionId) return;

      const directoryPath =
        customEvent.detail.path === "/"
          ? `/${customEvent.detail.name}`
          : `${customEvent.detail.path}/${customEvent.detail.name}`;
      await sftpStore.createDirectory(
        sftpStore.activeSessionId,
        directoryPath,
      );
      message.success("Directory created successfully");
    }
  } catch (error) {
    console.error("Failed to create directory:", error);
    message.error(
      getErrorMessage(error, "Failed to create directory"),
    );
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
      // Create local file using Tauri fs API
      const filePath =
        customEvent.detail.path === "/"
          ? `/${customEvent.detail.name}`
          : await join(customEvent.detail.path, customEvent.detail.name);

      // Create empty file
      await writeFile(filePath, new Uint8Array());

      // Refresh directory
      await sftpStore.listLocalDirectory(customEvent.detail.path);
      message.success("File created successfully");
    } else {
      // Create remote file via SFTP
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

      // Upload empty file
      await sftpStore.uploadFile(
        sftpStore.activeSessionId,
        tempFilePath,
        filePath,
      );

      // Cleanup temp file
      await remove(tempFilePath).catch(() => {
        // Ignore cleanup errors
      });

      message.success("File created successfully");
    }
  } catch (error) {
    console.error("Failed to create file:", error);
    message.error(
      getErrorMessage(error, "Failed to create file"),
    );
  }
}

function onPaneResize() {
  // Handle pane resize if needed
}
</script>

