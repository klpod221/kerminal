<template>
  <div class="h-full flex flex-col bg-[#0D0D0D]">
    <!-- Header with connection selector -->
    <div class="flex items-center justify-between px-4 py-2 border-b border-gray-800">
      <div class="flex items-center gap-3">
        <Select
          id="sftp-profile-select"
          v-model="selectedProfileId"
          :options="sshProfiles"
          placeholder="Select SSH Profile"
          :space="false"
          @update:modelValue="handleProfileSelect"
        />
        <Button
          v-if="sftpStore.activeSessionId"
          variant="ghost"
          size="sm"
          @click="handleDisconnect"
        >
          Disconnect
        </Button>
      </div>

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

    <!-- Transfer Manager Modal -->
    <Modal
      id="sftp-transfer-manager-modal"
      title="Transfer Manager"
      :icon="Activity"
      icon-background="bg-blue-500/20"
      icon-color="text-blue-400"
      size="lg"
    >
      <TransferManager />
    </Modal>

    <!-- Modals -->
    <FileRenameModal />
    <FileDeleteModal />
    <FilePermissionsModal />
    <CreateDirectoryModal />
    <CreateFileModal />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { Splitpanes, Pane } from "splitpanes";
import "splitpanes/dist/splitpanes.css";
import { FolderOpen, Activity } from "lucide-vue-next";
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
import Button from "../ui/Button.vue";
import Select from "../ui/Select.vue";
import EmptyState from "../ui/EmptyState.vue";
import Modal from "../ui/Modal.vue";
import type { FileEntry } from "../../types/sftp";

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
      const { rename } = await import("@tauri-apps/plugin-fs");
      const { dirname } = await import("@tauri-apps/api/path");

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
      const { remove, stat } = await import("@tauri-apps/plugin-fs");
      const { dirname } = await import("@tauri-apps/api/path");

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
  const { homeDir } = await import("@tauri-apps/api/path");
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
          const { writeFile } = await import("@tauri-apps/plugin-fs");
          const { tempDir } = await import("@tauri-apps/api/path");
          const { join } = await import("@tauri-apps/api/path");

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
        const { remove } = await import("@tauri-apps/plugin-fs");
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
    const { openPath } = await import("@tauri-apps/plugin-opener");
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

async function handleRemoteDownload(file: FileEntry) {
  if (!sftpStore.activeSessionId) return;

  try {
    const { save } = await import("@tauri-apps/plugin-dialog");
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
      const { mkdir } = await import("@tauri-apps/plugin-fs");
      const { join } = await import("@tauri-apps/api/path");

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
      const { writeFile } = await import("@tauri-apps/plugin-fs");
      const { join } = await import("@tauri-apps/api/path");

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

      // For remote, we need to create an empty file
      // SFTP can create file by opening it for write and closing immediately
      // But we don't have a direct API for this, so we might need to use upload with empty content
      // For now, let's use a workaround: create a temp empty file and upload it
      const { writeFile } = await import("@tauri-apps/plugin-fs");
      const { tempDir, join } = await import("@tauri-apps/api/path");

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
      const { remove } = await import("@tauri-apps/plugin-fs");
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

