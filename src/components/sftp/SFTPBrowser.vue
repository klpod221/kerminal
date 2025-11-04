<template>
  <div class="h-full flex flex-col bg-[#0D0D0D] relative">
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
          variant="danger"
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
    <div class="flex-1 overflow-hidden">
      <Splitpanes class="default-theme" @resize="onPaneResize">
        <Pane
          :size="sftpStore.activeSessionId || sftpStore.connecting ? 50 : 100"
          :min-size="20"
          :class="
            isDraggingOverPane === 'local'
              ? 'bg-blue-500/10 border-2 border-blue-500 border-dashed'
              : ''
          "
        >
          <!-- Local file browser -->
          <div
            class="h-full w-full"
            @dragover="handlePaneDragOver('local', $event)"
            @dragenter="handlePaneDragEnter('local', $event)"
            @dragleave="handlePaneDragLeave('local', $event)"
            @drop.prevent="handlePaneDrop('local', $event)"
          >
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
              @download="handleLocalDownload"
              @create-directory="handleLocalCreateDirectory"
              @create-file="handleLocalCreateFile"
            />
          </div>
        </Pane>

        <Pane
          v-if="sftpStore.activeSessionId || sftpStore.connecting"
          :size="50"
          :min-size="20"
          :class="
            isDraggingOverPane === 'remote'
              ? 'bg-blue-500/10 border-2 border-blue-500 border-dashed'
              : ''
          "
        >
          <!-- Remote file browser -->
          <div
            class="h-full w-full relative"
            @dragover="handlePaneDragOver('remote', $event)"
            @dragenter="handlePaneDragEnter('remote', $event)"
            @dragleave="handlePaneDragLeave('remote', $event)"
            @drop.prevent="handlePaneDrop('remote', $event)"
          >
            <!-- Connecting Overlay for Remote Panel -->
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
            <FileBrowser
              :files="sftpStore.browserState.remoteFiles"
              :current-path="sftpStore.browserState.remotePath"
              :loading="
                sftpStore.browserState.loading.remote || sftpStore.connecting
              "
              :is-remote="true"
              :selected-files="sftpStore.browserState.selectedRemoteFiles"
              @navigate="handleRemoteNavigate"
              @refresh="handleRemoteRefresh"
              @select="handleRemoteSelect"
              @download="handleRemoteDownload"
              @delete="handleRemoteDelete"
              @permissions="handleRemotePermissions"
              @drag-files="handleRemoteDragFiles"
              @open="handleRemoteOpen"
              @edit="handleRemoteEdit"
              @rename="handleRemoteRename"
              @create-directory="handleRemoteCreateDirectory"
              @create-file="handleRemoteCreateFile"
            />
          </div>
        </Pane>
      </Splitpanes>
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
import { Activity, GitCompare } from "lucide-vue-next";
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
import type { FileEntry } from "../../types/sftp";
import {
  rename,
  remove,
  stat,
  writeFile,
  mkdir,
  readDir,
} from "@tauri-apps/plugin-fs";
import { dirname, homeDir, tempDir, join } from "@tauri-apps/api/path";
import * as sftpService from "../../services/sftp";
import { openPath } from "@tauri-apps/plugin-opener";
import { save, open } from "@tauri-apps/plugin-dialog";

const sftpStore = useSFTPStore();
const sshStore = useSSHStore();
const { openOverlay, closeOverlay, isOverlayVisible } = useOverlay();

const selectedProfileId = ref<string>("");
const isDraggingOverPane = ref<"local" | "remote" | null>(null);
const dragEnterCounter = ref<{ local: number; remote: number }>({
  local: 0,
  remote: 0,
});

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

  if (
    !sftpStore.browserState.localPath ||
    sftpStore.browserState.localFiles.length === 0
  ) {
    const homeDir = await getHomeDirectory();
    await sftpStore.listLocalDirectory(homeDir);
  }

  if (sftpStore.activeSessionId && sftpStore.activeSession) {
    selectedProfileId.value = sftpStore.activeSession.profileId;
  }

  watch(
    () => sftpStore.activeSession,
    (session) => {
      if (session) {
        selectedProfileId.value = session.profileId;
      } else {
        selectedProfileId.value = "";
      }
    },
    { immediate: true },
  );

  watch(
    () => sftpStore.activeSessionId,
    () => {},
  );

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
    path?: string;
    paths?: string[];
    mode: number;
  }>;
  if (!sftpStore.activeSessionId) return;

  const paths =
    customEvent.detail.paths ||
    (customEvent.detail.path ? [customEvent.detail.path] : []);

  if (paths.length === 0) return;

  try {
    for (const path of paths) {
      await sftpStore.setPermissions(
        sftpStore.activeSessionId,
        path,
        customEvent.detail.mode,
      );
    }

    const dirPath =
      paths.length > 0
        ? paths[0].substring(0, paths[0].lastIndexOf("/")) || "/"
        : sftpStore.browserState.remotePath || "/";

    await sftpStore.listRemoteDirectory(sftpStore.activeSessionId, dirPath);

    message.success(
      `Permissions updated successfully for ${paths.length} item${paths.length > 1 ? "s" : ""}`,
    );
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    message.error(`Failed to update permissions: ${errorMessage}`);
  }
}

/**
 * Handle SSH profile selection and connect to SFTP
 * @param profileId - SSH profile ID to connect with
 */
async function handleProfileSelect(profileId: string) {
  if (!profileId || profileId === "") return;

  await sftpStore.connect(profileId);

  selectedProfileId.value = profileId;

  if (!sftpStore.browserState.localPath) {
    const homeDir = await getHomeDirectory();
    await sftpStore.listLocalDirectory(homeDir);
  }

  if (sftpStore.activeSessionId) {
    const profile = sshStore.profiles.find((p) => p.id === profileId);
    const remotePath = profile?.username ? `/home/${profile.username}` : "/";
    await sftpStore.listRemoteDirectory(sftpStore.activeSessionId, remotePath);
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

      if (fileWithPath.path && !fileWithPath.path.includes("/")) {
        // File has a direct path (not from directory structure)
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

    await sftpStore.listRemoteDirectory(sftpStore.activeSessionId, remotePath);
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
      }, 5000);
    }
  }
}

/**
 * Check if a file can be previewed
 */
function canPreviewFile(file: FileEntry): boolean {
  if (file.fileType !== "file") return false;

  const previewableExtensions = new Set([
    "jpg",
    "jpeg",
    "png",
    "gif",
    "bmp",
    "webp",
    "svg",
    "ico",
    "mp4",
    "webm",
    "ogg",
    "avi",
    "mov",
    "wmv",
    "flv",
    "mkv",
    "html",
    "htm",
    "md",
    "markdown",
    "pdf",
    "txt",
    "text",
    "log",
    "json",
    "yaml",
    "yml",
    "xml",
    "csv",
    "tsv",
    "ini",
    "conf",
    "config",
    "cfg",
    "properties",
    "env",
    "gitignore",
    "gitattributes",
    "dockerfile",
    "makefile",
    "sh",
    "bash",
    "zsh",
    "fish",
    "ps1",
    "bat",
    "cmd",
    "js",
    "ts",
    "jsx",
    "tsx",
    "vue",
    "svelte",
    "py",
    "java",
    "c",
    "cpp",
    "cc",
    "h",
    "hpp",
    "cs",
    "go",
    "rs",
    "rb",
    "php",
    "swift",
    "kt",
    "scala",
    "clj",
    "hs",
    "ml",
    "sql",
    "r",
    "m",
    "pl",
    "pm",
    "lua",
    "vim",
    "css",
    "scss",
    "sass",
    "less",
    "styl",
    "stylus",
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

function handleLocalDelete(files: FileEntry[]) {
  if (files.length === 0) return;

  if (files.length === 1) {
    openOverlay("sftp-file-delete-modal", {
      file: files[0],
      isLocal: true,
    });
  } else {
    deleteLocalFiles(files);
  }
}

async function deleteLocalFiles(files: FileEntry[]) {
  try {
    for (const file of files) {
      const fileStat = await stat(file.path);
      const isDirectory = fileStat.isDirectory;

      if (isDirectory) {
        await remove(file.path, { recursive: true });
      } else {
        await remove(file.path);
      }
    }

    const dirPath =
      files.length > 0
        ? await dirname(files[0].path)
        : sftpStore.browserState.localPath || "/";

    await sftpStore.listLocalDirectory(dirPath);
    message.success(
      `Deleted ${files.length} item${files.length > 1 ? "s" : ""} successfully`,
    );
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    message.error(`Failed to delete: ${errorMessage}`);
  }
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

/**
 * Generate a unique filename by appending a number
 */
async function generateUniqueLocalPath(
  basePath: string,
  fileName: string,
): Promise<string> {
  const baseName = fileName.substring(0, fileName.lastIndexOf(".")) || fileName;
  const ext = fileName.includes(".")
    ? fileName.substring(fileName.lastIndexOf("."))
    : "";

  let counter = 1;
  let newPath =
    basePath === "/"
      ? await join("/", fileName)
      : await join(basePath, fileName);

  while (true) {
    try {
      await stat(newPath);
      const newName = `${baseName} (${counter})${ext}`;
      newPath =
        basePath === "/"
          ? await join("/", newName)
          : await join(basePath, newName);
      counter++;
    } catch {
      break;
    }
  }

  return newPath;
}

/**
 * Generate a unique remote filename by appending a number
 */
async function generateUniqueRemotePath(
  sessionId: string,
  basePath: string,
  fileName: string,
): Promise<string> {
  const baseName = fileName.substring(0, fileName.lastIndexOf(".")) || fileName;
  const ext = fileName.includes(".")
    ? fileName.substring(fileName.lastIndexOf("."))
    : "";

  let counter = 1;
  let newPath = basePath === "/" ? `/${fileName}` : `${basePath}/${fileName}`;

  while (true) {
    try {
      await sftpService.statSFTP(sessionId, newPath);
      const newName = `${baseName} (${counter})${ext}`;
      newPath = basePath === "/" ? `/${newName}` : `${basePath}/${newName}`;
      counter++;
    } catch {
      break;
    }
  }

  return newPath;
}

async function handleLocalDragFiles(
  files: FileEntry[],
  targetPath: string,
  isSourceRemote: boolean,
) {
  if (!sftpStore.activeSessionId && isSourceRemote) {
    return;
  }

  if (isSourceRemote) {
    const localPath = targetPath || sftpStore.browserState.localPath || "/";

    for (const file of files) {
      if (file.fileType === "directory") {
        try {
          await downloadDirectoryRecursive(
            sftpStore.activeSessionId!,
            file.path,
            localPath,
            file.name,
          );
          message.success(`Downloading directory ${file.name}...`);
        } catch (error) {
          const errorMessage =
            error instanceof Error ? error.message : String(error);
          message.error(
            `Failed to download directory ${file.name}: ${errorMessage}`,
          );
        }
      } else {
        let localFilePath =
          localPath === "/"
            ? await join("/", file.name)
            : await join(localPath, file.name);

        try {
          await stat(localFilePath);
          localFilePath = await generateUniqueLocalPath(localPath, file.name);
          message.info(
            `${file.name} already exists, using ${localFilePath.split("/").pop()}`,
          );
        } catch {}

        await sftpStore.downloadFile(
          sftpStore.activeSessionId!,
          file.path,
          localFilePath,
        );
        message.success(`Downloading ${file.name}...`);
      }
    }

    await sftpStore.listLocalDirectory(localPath);
  } else {
    const destinationPath =
      targetPath || sftpStore.browserState.localPath || "/";

    for (const file of files) {
      try {
        const currentDir = await dirname(file.path);
        if (currentDir === destinationPath) {
          message.info(`${file.name} is already in the target directory`);
          continue;
        }

        let newPath =
          destinationPath === "/"
            ? await join("/", file.name)
            : await join(destinationPath, file.name);

        try {
          await stat(newPath);
          newPath = await generateUniqueLocalPath(destinationPath, file.name);
          message.info(
            `${file.name} already exists, using ${newPath.split("/").pop()}`,
          );
        } catch {}

        await rename(file.path, newPath);
        message.success(`Moved ${file.name} to ${destinationPath}`);

        await sftpStore.listLocalDirectory(currentDir);
        await sftpStore.listLocalDirectory(destinationPath);
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : String(error);
        message.error(`Failed to move ${file.name}: ${errorMessage}`);
      }
    }
  }
}

async function handleRemoteDragFiles(
  files: FileEntry[],
  targetPath: string,
  isSourceRemote: boolean,
) {
  if (!sftpStore.activeSessionId) {
    return;
  }

  if (!isSourceRemote) {
    const remotePath = targetPath || sftpStore.browserState.remotePath || "/";

    for (const file of files) {
      if (file.fileType === "directory") {
        try {
          await uploadDirectoryRecursive(
            sftpStore.activeSessionId,
            file.path,
            remotePath,
            file.name,
          );
          message.success(`Uploading directory ${file.name}...`);
        } catch (error) {
          const errorMessage =
            error instanceof Error ? error.message : String(error);
          message.error(
            `Failed to upload directory ${file.name}: ${errorMessage}`,
          );
        }
      } else {
        const normalizedRemotePath = remotePath.endsWith("/")
          ? remotePath.slice(0, -1)
          : remotePath;
        let remoteFilePath = `${normalizedRemotePath}/${file.name}`;

        try {
          await sftpService.statSFTP(sftpStore.activeSessionId, remoteFilePath);
          remoteFilePath = await generateUniqueRemotePath(
            sftpStore.activeSessionId,
            normalizedRemotePath,
            file.name,
          );
          message.info(
            `${file.name} already exists, using ${remoteFilePath.split("/").pop()}`,
          );
        } catch {}

        await sftpStore.uploadFile(
          sftpStore.activeSessionId,
          file.path,
          remoteFilePath,
        );
        message.success(`Uploading ${file.name}...`);
      }
    }
  } else {
    const destinationPath =
      targetPath || sftpStore.browserState.remotePath || "/";

    for (const file of files) {
      try {
        const currentDir =
          file.path.substring(0, file.path.lastIndexOf("/")) || "/";
        if (currentDir === destinationPath) {
          message.info(`${file.name} is already in the target directory`);
          continue;
        }

        let newPath =
          destinationPath === "/"
            ? `/${file.name}`
            : `${destinationPath}/${file.name}`;

        try {
          await sftpService.statSFTP(sftpStore.activeSessionId, newPath);
          newPath = await generateUniqueRemotePath(
            sftpStore.activeSessionId,
            destinationPath,
            file.name,
          );
          message.info(
            `${file.name} already exists, using ${newPath.split("/").pop()}`,
          );
        } catch {}

        await sftpStore.renameFile(
          sftpStore.activeSessionId,
          file.path,
          newPath,
        );
        message.success(`Moved ${file.name} to ${destinationPath}`);

        await sftpStore.listRemoteDirectory(
          sftpStore.activeSessionId,
          currentDir,
        );
        await sftpStore.listRemoteDirectory(
          sftpStore.activeSessionId,
          destinationPath,
        );
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : String(error);
        message.error(`Failed to move ${file.name}: ${errorMessage}`);
      }
    }
  }
}

async function handleRemoteDownload(files: FileEntry[]) {
  if (!sftpStore.activeSessionId || files.length === 0) return;

  let targetDir: string | null = null;

  if (files.length === 1) {
    const localPathResult = await save({
      defaultPath: files[0].name,
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
        files[0].path,
        localPathResult,
      );
      const downloadDir = await dirname(localPathResult);
      const currentLocalPath = sftpStore.browserState.localPath || "/";
      if (downloadDir === currentLocalPath) {
        await sftpStore.listLocalDirectory(currentLocalPath);
      }
      message.success("Download started");
    }
  } else {
    const selectedDir = await open({
      directory: true,
      multiple: false,
    });

    if (selectedDir && typeof selectedDir === "string") {
      targetDir = selectedDir;
    } else if (Array.isArray(selectedDir) && selectedDir.length > 0) {
      targetDir = selectedDir[0];
    }

    if (targetDir) {
      for (const file of files) {
        if (file.fileType === "file") {
          const localFilePath = await join(targetDir, file.name);
          await sftpStore.downloadFile(
            sftpStore.activeSessionId,
            file.path,
            localFilePath,
          );
        }
      }

      await sftpStore.listLocalDirectory(targetDir);
      message.success(
        `Downloading ${files.length} file${files.length > 1 ? "s" : ""}...`,
      );
    }
  }
}

async function handleLocalDownload(_files: FileEntry[]) {
  message.info("Local download not implemented");
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

function handleRemoteDelete(files: FileEntry[]) {
  if (files.length === 0) return;

  if (files.length === 1) {
    openOverlay("sftp-file-delete-modal", { file: files[0] });
  } else {
    deleteRemoteFiles(files);
  }
}

async function deleteRemoteFiles(files: FileEntry[]) {
  if (!sftpStore.activeSessionId) return;

  try {
    for (const file of files) {
      await sftpStore.deleteFile(
        sftpStore.activeSessionId,
        file.path,
        file.fileType === "directory",
      );
    }

    const dirPath =
      files.length > 0
        ? files[0].path.substring(0, files[0].path.lastIndexOf("/")) || "/"
        : sftpStore.browserState.remotePath || "/";

    await sftpStore.listRemoteDirectory(sftpStore.activeSessionId, dirPath);
    message.success(
      `Deleted ${files.length} item${files.length > 1 ? "s" : ""} successfully`,
    );
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    message.error(`Failed to delete: ${errorMessage}`);
  }
}

function handleRemotePermissions(files: FileEntry[]) {
  if (files.length === 0) return;

  openOverlay("sftp-file-permissions-modal", { files });
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
      await sftpStore.listRemoteDirectory(
        sftpStore.activeSessionId,
        customEvent.detail.path,
      );
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

      await sftpStore.listRemoteDirectory(
        sftpStore.activeSessionId,
        customEvent.detail.path,
      );

      message.success("File created successfully");
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error("Failed to create file:", errorMessage);
    message.error(errorMessage);
  }
}

function onPaneResize() {}

/**
 * Handle drag enter pane for visual feedback
 */
function handlePaneDragEnter(pane: "local" | "remote", event: DragEvent) {
  event.preventDefault();
  if (!event.dataTransfer) return;

  dragEnterCounter.value[pane]++;

  if (event.dataTransfer.types.includes("application/x-filebrowser-files")) {
    if (dragEnterCounter.value[pane] === 1) {
      isDraggingOverPane.value = pane;
    }
    event.dataTransfer.dropEffect = "copy";
  }
}

/**
 * Handle drag over pane for visual feedback
 */
function handlePaneDragOver(pane: "local" | "remote", event: DragEvent) {
  event.preventDefault();
  if (!event.dataTransfer) return;

  if (event.dataTransfer.types.includes("application/x-filebrowser-files")) {
    isDraggingOverPane.value = pane;
    event.dataTransfer.dropEffect = "copy";
  }
}

/**
 * Handle drag leave pane
 */
function handlePaneDragLeave(pane: "local" | "remote", event: DragEvent) {
  event.preventDefault();
  dragEnterCounter.value[pane]--;

  if (dragEnterCounter.value[pane] === 0) {
    isDraggingOverPane.value = null;
  }
}

/**
 * Handle drop on pane wrapper
 */
function handlePaneDrop(pane: "local" | "remote", event: DragEvent) {
  event.preventDefault();
  event.stopPropagation();

  isDraggingOverPane.value = null;
  dragEnterCounter.value[pane] = 0;

  if (!event.dataTransfer) return;

  const dragData = event.dataTransfer.getData(
    "application/x-filebrowser-files",
  );

  if (dragData) {
    try {
      const data = JSON.parse(dragData) as {
        files: Array<{
          path: string;
          name: string;
          fileType: string;
          size?: number | null;
          modified?: string;
        }>;
        isRemote: boolean;
        sourcePath: string;
      };

      const isSourceRemote = data.isRemote;
      if (
        (pane === "local" && isSourceRemote) ||
        (pane === "remote" && !isSourceRemote)
      ) {
        const targetPath =
          pane === "local"
            ? sftpStore.browserState.localPath || "/"
            : sftpStore.browserState.remotePath || "/";

        const draggedFileEntries: FileEntry[] = data.files.map((f) => ({
          name: f.name || "",
          path: f.path || "",
          fileType: (f.fileType || "file") as FileEntry["fileType"],
          size: f.size ?? null,
          modified: f.modified || new Date().toISOString(),
          accessed: null,
          permissions: 0o644,
          symlinkTarget: null,
          uid: null,
          gid: null,
        }));

        if (pane === "local") {
          handleLocalDragFiles(draggedFileEntries, targetPath, isSourceRemote);
        } else {
          handleRemoteDragFiles(draggedFileEntries, targetPath, isSourceRemote);
        }
      }
    } catch (error) {}
  }
}

/**
 * Recursively collect all files from a local directory
 */
async function collectLocalDirectoryFiles(
  dirPath: string,
  basePath: string,
): Promise<Array<{ path: string; relativePath: string }>> {
  const files: Array<{ path: string; relativePath: string }> = [];
  const entries = await readDir(dirPath);

  for (const entry of entries) {
    const normalizedPath = dirPath.endsWith("/")
      ? dirPath.slice(0, -1)
      : dirPath;
    const entryPath =
      normalizedPath === "/"
        ? `/${entry.name}`
        : `${normalizedPath}/${entry.name}`;

    try {
      const entryStat = await stat(entryPath);
      const relativePath = entryPath.replace(basePath, "").replace(/^\//, "");

      if (entryStat.isDirectory) {
        const subFiles = await collectLocalDirectoryFiles(entryPath, basePath);
        files.push(...subFiles);
      } else {
        files.push({ path: entryPath, relativePath });
      }
    } catch (error) {
      console.warn(`Failed to process ${entryPath}:`, error);
    }
  }

  return files;
}

/**
 * Recursively collect all files from a remote directory
 */
async function collectRemoteDirectoryFiles(
  sessionId: string,
  dirPath: string,
  basePath: string,
): Promise<Array<{ path: string; relativePath: string }>> {
  const files: Array<{ path: string; relativePath: string }> = [];

  const entries = await sftpService.listSFTPDirectory(sessionId, dirPath);

  for (const entry of entries) {
    if (entry.name === "." || entry.name === "..") {
      continue;
    }

    const relativePath = entry.path.replace(basePath, "").replace(/^\//, "");

    if (entry.fileType === "directory") {
      const subFiles = await collectRemoteDirectoryFiles(
        sessionId,
        entry.path,
        basePath,
      );
      files.push(...subFiles);
    } else if (entry.fileType === "file") {
      files.push({ path: entry.path, relativePath });
    }
  }

  return files;
}

/**
 * Upload a directory recursively to remote
 */
async function uploadDirectoryRecursive(
  sessionId: string,
  localDirPath: string,
  remoteBasePath: string,
  dirName: string,
): Promise<void> {
  const remoteDirPath = remoteBasePath.endsWith("/")
    ? `${remoteBasePath}${dirName}`
    : `${remoteBasePath}/${dirName}`;

  try {
    await sftpStore.createDirectory(sessionId, remoteDirPath);
  } catch (error) {}

  const files = await collectLocalDirectoryFiles(localDirPath, localDirPath);

  for (const file of files) {
    const remoteFilePath = remoteDirPath.endsWith("/")
      ? `${remoteDirPath}${file.relativePath}`
      : `${remoteDirPath}/${file.relativePath}`;

    const remoteParentDir = await dirname(remoteFilePath).catch(() => {
      const parts = remoteFilePath.split("/");
      parts.pop();
      return parts.join("/") || "/";
    });

    try {
      await sftpStore.createDirectory(sessionId, remoteParentDir);
    } catch (error) {}

    await sftpStore.uploadFile(sessionId, file.path, remoteFilePath);
  }
}

/**
 * Download a directory recursively from remote
 */
async function downloadDirectoryRecursive(
  sessionId: string,
  remoteDirPath: string,
  localBasePath: string,
  dirName: string,
): Promise<void> {
  const localDirPath = await join(localBasePath, dirName);
  try {
    await mkdir(localDirPath, { recursive: true });
  } catch (error) {}

  const files = await collectRemoteDirectoryFiles(
    sessionId,
    remoteDirPath,
    remoteDirPath,
  );

  for (const file of files) {
    const localFilePath = await join(localDirPath, file.relativePath);

    const localParentDir = await dirname(localFilePath).catch(() => {
      const parts = localFilePath.split("/");
      parts.pop();
      return parts.join("/") || "/";
    });

    try {
      await mkdir(localParentDir, { recursive: true });
    } catch (error) {}

    await sftpStore.downloadFile(sessionId, file.path, localFilePath);
  }
}
</script>
