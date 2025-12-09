<template>
  <Modal
    id="sftp-file-preview-modal"
    :title="`Preview ${file?.name || 'File'}`"
    :icon="previewIcon"
    icon-background="bg-purple-500/20"
    icon-color="text-purple-400"
    :size="isFullscreen ? 'full' : '6xl'"
  >
    <!-- Header Controls -->
    <template #header-actions>
      <div class="flex items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          :icon="isFullscreen ? Minimize2 : Maximize2"
          @click="toggleFullscreen"
          :title="isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'"
        />
      </div>
    </template>

    <div v-if="loading" class="flex items-center justify-center py-12">
      <div class="text-gray-400">Loading preview...</div>
    </div>
    <div v-else-if="error" class="py-4">
      <div class="text-red-400 text-sm mb-4">{{ error }}</div>
      <Button variant="primary" @click="loadPreview">Retry</Button>
    </div>
    <div v-else class="flex flex-col gap-4 h-full">
      <!-- File Info & Navigation -->
      <div
        class="flex items-center justify-between border-b border-gray-700 pb-2"
      >
        <div class="flex items-center gap-4 text-xs text-gray-500">
          <span>
            <span class="font-mono text-gray-400">{{ file?.path }}</span>
          </span>
          <span class="text-gray-600">{{
            formatFileSize(file?.size || 0)
          }}</span>
        </div>

        <!-- Gallery Navigation -->
        <div class="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            :icon="ChevronLeft"
            :disabled="!hasPrev"
            @click="navigate('prev')"
          />
          <span class="text-xs text-gray-500 w-16 text-center">
            {{ currentFileIndex + 1 }} / {{ previewableFiles.length }}
          </span>
          <Button
            variant="ghost"
            size="sm"
            :icon="ChevronRight"
            :disabled="!hasNext"
            @click="navigate('next')"
          />
        </div>
      </div>

      <!-- Image Preview -->
      <div
        v-if="previewType === 'image'"
        class="flex-1 flex flex-col min-h-0 bg-gray-900 rounded-lg overflow-hidden relative group"
        @wheel="handleWheel"
      >
        <!-- Image Toolbar -->
        <div
          class="absolute top-4 right-4 z-10 flex items-center gap-1 bg-gray-800/80 p-1 rounded backdrop-blur opacity-0 group-hover:opacity-100 transition-opacity"
        >
          <Button
            variant="ghost"
            size="sm"
            :icon="ZoomIn"
            @click="handleZoomIn"
            title="Zoom In"
          />
          <div class="text-xs text-gray-300 w-12 text-center">
            {{ Math.round(zoomLevel * 100) }}%
          </div>
          <Button
            variant="ghost"
            size="sm"
            :icon="ZoomOut"
            @click="handleZoomOut"
            title="Zoom Out"
          />
          <Button
            variant="ghost"
            size="sm"
            :icon="RotateCcw"
            @click="resetZoom"
            title="Reset"
          />
        </div>

        <div
          class="flex-1 w-full h-full overflow-hidden flex items-center justify-center"
          :class="{ 'cursor-move': zoomLevel > 1 }"
          @mousedown="startPan"
        >
          <img
            :src="previewUrl"
            :alt="file?.name"
            class="max-w-none transition-transform duration-100 ease-out select-none"
            :style="{
              transform: `scale(${zoomLevel}) translate(${panX / zoomLevel}px, ${panY / zoomLevel}px)`,
              maxWidth: zoomLevel <= 1 ? '100%' : 'none',
              maxHeight: zoomLevel <= 1 ? '100%' : 'none',
            }"
            @error="handleImageError"
            draggable="false"
          />
        </div>
      </div>

      <!-- Video Preview - Show as document info -->
      <div
        v-else-if="previewType === 'video'"
        class="flex flex-col items-center justify-center py-12 gap-4 flex-1"
      >
        <component :is="Video" :size="64" class="text-gray-500" />
        <div class="text-center">
          <p class="text-gray-300 mb-2">Video preview is not available</p>
          <p class="text-sm text-gray-500 mb-4">
            File type: <span class="font-mono">{{ fileType }}</span>
          </p>
          <Button variant="primary" @click="downloadFile">
            Download to View
          </Button>
        </div>
      </div>

      <!-- HTML Preview -->
      <div
        v-else-if="previewType === 'html'"
        class="flex flex-col gap-2 flex-1 min-h-0"
      >
        <div class="flex gap-2 border-b border-gray-700 pb-2">
          <Button
            :variant="viewMode === 'rendered' ? 'primary' : 'ghost'"
            size="sm"
            @click="viewMode = 'rendered'"
          >
            Rendered
          </Button>
          <Button
            :variant="viewMode === 'source' ? 'primary' : 'ghost'"
            size="sm"
            @click="viewMode = 'source'"
          >
            Source
          </Button>
        </div>
        <div
          v-if="viewMode === 'rendered'"
          class="bg-gray-900 rounded-lg overflow-hidden flex-1"
        >
          <iframe
            :srcdoc="htmlContent"
            class="w-full h-full border-0"
            sandbox="allow-same-origin allow-scripts"
            title="HTML Preview"
          />
        </div>
        <div
          v-else
          class="bg-gray-900 rounded-lg p-4 overflow-auto flex-1 text-xs text-gray-300 font-mono"
        >
          <pre class="whitespace-pre-wrap">{{ htmlContent }}</pre>
        </div>
      </div>

      <!-- Text Preview -->
      <div
        v-else-if="previewType === 'text'"
        class="flex flex-col gap-2 flex-1 min-h-0"
      >
        <div
          class="bg-gray-900 rounded-lg p-4 overflow-auto border border-gray-700 flex-1"
        >
          <pre class="text-xs text-gray-300 whitespace-pre-wrap font-mono">{{
            textContent
          }}</pre>
        </div>
      </div>

      <!-- Markdown Preview -->
      <div
        v-else-if="previewType === 'markdown'"
        class="flex flex-col gap-2 flex-1 min-h-0"
      >
        <div class="flex gap-2 border-b border-gray-700 pb-2">
          <Button
            :variant="viewMode === 'rendered' ? 'primary' : 'ghost'"
            size="sm"
            @click="viewMode = 'rendered'"
          >
            Preview
          </Button>
          <Button
            :variant="viewMode === 'source' ? 'primary' : 'ghost'"
            size="sm"
            @click="viewMode = 'source'"
          >
            Source
          </Button>
        </div>
        <div
          v-if="viewMode === 'rendered'"
          class="bg-gray-900 rounded-lg p-6 overflow-auto prose prose-invert prose-sm max-w-none flex-1"
        >
          <div v-html="renderedMarkdown" />
        </div>
        <div v-else class="bg-gray-900 rounded-lg p-4 overflow-auto flex-1">
          <pre class="text-xs text-gray-300 whitespace-pre-wrap font-mono">{{
            markdownContent
          }}</pre>
        </div>
      </div>

      <!-- PDF Preview -->
      <div
        v-else-if="previewType === 'pdf'"
        class="bg-gray-900 rounded-lg overflow-hidden flex-1"
      >
        <iframe
          :src="previewUrl"
          class="w-full h-full border-0"
          type="application/pdf"
          title="PDF Preview"
        />
        <div
          v-if="!previewUrl"
          class="flex items-center justify-center h-full text-gray-400"
        >
          <p>
            Your browser does not support PDFs.
            <a :href="previewUrl" download class="text-blue-400 hover:underline"
              >Download the PDF</a
            >.
          </p>
        </div>
      </div>

      <!-- Office Documents Info -->
      <div
        v-else-if="previewType === 'office'"
        class="flex flex-col items-center justify-center py-12 gap-4 flex-1"
      >
        <component :is="FileText" :size="64" class="text-gray-500" />
        <div class="text-center">
          <p class="text-gray-300 mb-2">
            Office document preview is not available
          </p>
          <p class="text-sm text-gray-500 mb-4">
            File type: <span class="font-mono">{{ fileType }}</span>
          </p>
          <Button variant="primary" @click="downloadFile">
            Download to View
          </Button>
        </div>
      </div>

      <!-- Unknown/Unsupported -->
      <div
        v-else
        class="flex flex-col items-center justify-center py-12 gap-4 flex-1"
      >
        <component :is="FileQuestion" :size="64" class="text-gray-500" />
        <div class="text-center">
          <p class="text-gray-300 mb-2">
            Preview not available for this file type
          </p>
          <p class="text-sm text-gray-500 mb-4">
            File type: <span class="font-mono">{{ fileType }}</span>
          </p>
          <div class="flex gap-2 justify-center">
            <Button variant="ghost" @click="openInEditor" v-if="canEdit">
              Open in Editor
            </Button>
            <Button variant="primary" @click="downloadFile"> Download </Button>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <Button variant="ghost" @click="closeModal">Close</Button>
      <Button
        v-if="
          previewType !== 'office' &&
          previewType !== 'video' &&
          previewType !== 'unknown'
        "
        variant="ghost"
        @click="downloadFile"
      >
        Download
      </Button>
      <Button
        v-if="previewType === 'office' || previewType === 'video'"
        variant="primary"
        @click="downloadFile"
      >
        Download
      </Button>
      <Button
        v-if="canEdit && previewType === 'text'"
        variant="primary"
        @click="openInEditor"
      >
        Edit
      </Button>
      <Button
        v-if="
          canEdit &&
          previewType !== 'image' &&
          previewType !== 'video' &&
          previewType !== 'pdf' &&
          previewType !== 'office' &&
          previewType !== 'text'
        "
        variant="primary"
        @click="openInEditor"
      >
        Edit
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import {
  FileText,
  Image as ImageIcon,
  Video,
  FileQuestion,
  FileCode,
  File,
  Maximize2,
  Minimize2,
  ChevronLeft,
  ChevronRight,
  ZoomIn,
  ZoomOut,
  RotateCcw,
} from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import { message } from "../../utils/message";
import { useSFTPStore } from "../../stores/sftp";
import type { FileEntry } from "../../types/sftp";
import { readFile, remove } from "@tauri-apps/plugin-fs";
import { tempDir, join } from "@tauri-apps/api/path";
import { save } from "@tauri-apps/plugin-dialog";

const { closeOverlay, getOverlayProp, updateOverlayProp } = useOverlay();
const sftpStore = useSFTPStore();

const loading = ref(false);
const error = ref<string | null>(null);
const previewUrl = ref<string>("");
const htmlContent = ref<string>("");
const markdownContent = ref<string>("");
const renderedMarkdown = ref<string>("");
const textContent = ref<string>("");
const viewMode = ref<"rendered" | "source">("rendered");
const tempFilePath = ref<string | null>(null);

const isFullscreen = ref(false);

// Zoom & Pan state
const zoomLevel = ref(1);
const panX = ref(0);
const panY = ref(0);
const isPanning = ref(false);
const startPanX = ref(0);
const startPanY = ref(0);

const file = getOverlayProp<FileEntry | null>(
  "sftp-file-preview-modal",
  "file",
  null,
  null,
);

const isLocal = getOverlayProp<boolean>(
  "sftp-file-preview-modal",
  "isLocal",
  false,
  false,
);

const allFiles = getOverlayProp<FileEntry[]>(
  "sftp-file-preview-modal",
  "files",
  [],
  [],
);

// Detect file type
const fileType = computed(() => {
  if (!file.value) return "unknown";
  const ext = file.value.name.split(".").pop()?.toLowerCase() || "";
  return ext;
});

const previewType = computed(() => {
  if (!file.value) return "unknown";

  const ext = fileType.value;

  // Images
  if (
    ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico"].includes(ext)
  ) {
    return "image";
  }

  // Videos - treat as document (no preview)
  if (["mp4", "webm", "ogg", "avi", "mov", "wmv", "flv", "mkv"].includes(ext)) {
    return "video";
  }

  // HTML
  if (["html", "htm"].includes(ext)) {
    return "html";
  }

  // Markdown
  if (["md", "markdown"].includes(ext)) {
    return "markdown";
  }

  // PDF
  if (ext === "pdf") {
    return "pdf";
  }

  // Text files
  const textExtensions = [
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
  ];
  if (textExtensions.includes(ext)) {
    return "text";
  }

  // Office documents
  if (
    ["doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp"].includes(
      ext,
    )
  ) {
    return "office";
  }

  return "unknown";
});

const previewIcon = computed(() => {
  switch (previewType.value) {
    case "image":
      return ImageIcon;
    case "video":
    case "office":
      return File;
    case "html":
    case "markdown":
    case "text":
      return FileCode;
    case "pdf":
      return File;
    default:
      return FileQuestion;
  }
});

const canEdit = computed(() => {
  return ["html", "markdown", "text"].includes(previewType.value);
});

// Gallery Navigation logic
const previewableFiles = computed(() => {
  if (!allFiles.value.length) return [];
  return allFiles.value.filter((f: FileEntry) => f.fileType === "file");
});

const currentFileIndex = computed(() => {
  if (!file.value || !previewableFiles.value.length) return -1;
  return previewableFiles.value.findIndex(
    (f: FileEntry) => f.path === file.value?.path,
  );
});

const hasNext = computed(
  () =>
    currentFileIndex.value !== -1 &&
    currentFileIndex.value < previewableFiles.value.length - 1,
);
const hasPrev = computed(() => currentFileIndex.value > 0);

function navigate(direction: "next" | "prev") {
  if (currentFileIndex.value === -1) return;

  let newIndex =
    direction === "next"
      ? currentFileIndex.value + 1
      : currentFileIndex.value - 1;
  if (newIndex >= 0 && newIndex < previewableFiles.value.length) {
    const nextFile = previewableFiles.value[newIndex];
    updateOverlayProp("sftp-file-preview-modal", "file", nextFile);
  }
}

// Zoom / Pan Logic
function resetZoom() {
  zoomLevel.value = 1;
  panX.value = 0;
  panY.value = 0;
}

function handleZoomIn() {
  zoomLevel.value = Math.min(zoomLevel.value + 0.25, 5);
}

function handleZoomOut() {
  zoomLevel.value = Math.max(zoomLevel.value - 0.25, 0.5);
}

function handleWheel(e: WheelEvent) {
  // Allow normal scrolling if not holding Ctrl key, unless we want to map scroll to zoom always.
  // Standard pattern: Ctrl+Scroll to zoom, Scroll to pan (if zoomed) or ignored.
  // But here we might want simple zoom on scroll for image viewer.
  if (e.ctrlKey || e.metaKey) {
    e.preventDefault();
    const delta = e.deltaY > 0 ? -0.25 : 0.25;
    const newZoom = Math.max(0.5, Math.min(5, zoomLevel.value + delta));
    zoomLevel.value = newZoom;
  }
}

function startPan(e: MouseEvent) {
  if (zoomLevel.value <= 1) return;
  isPanning.value = true;
  startPanX.value = e.clientX - panX.value;
  startPanY.value = e.clientY - panY.value;
  document.addEventListener("mousemove", handlePan);
  document.addEventListener("mouseup", endPan);
}

function handlePan(e: MouseEvent) {
  if (!isPanning.value) return;
  e.preventDefault();
  panX.value = e.clientX - startPanX.value;
  panY.value = e.clientY - startPanY.value;
}

function endPan() {
  isPanning.value = false;
  document.removeEventListener("mousemove", handlePan);
  document.removeEventListener("mouseup", endPan);
}

function renderMarkdown(md: string): string {
  let html = md
    .replaceAll(/^### (.*$)/gim, "<h3>$1</h3>")
    .replaceAll(/^## (.*$)/gim, "<h2>$1</h2>")
    .replaceAll(/^# (.*$)/gim, "<h1>$1</h1>")
    .replaceAll(/\*\*(.*?)\*\*/gim, "<strong>$1</strong>")
    .replaceAll(/__(.*?)__/gim, "<strong>$1</strong>")
    .replaceAll(/\*(.*?)\*/gim, "<em>$1</em>")
    .replaceAll(/_(.*?)_/gim, "<em>$1</em>")
    .replaceAll(/```([\s\S]*?)```/gim, "<pre><code>$1</code></pre>")
    .replaceAll(/`(.*?)`/gim, "<code>$1</code>")
    .replaceAll(
      /\[([^\]]+)\]\(([^)]+)\)/gim,
      '<a href="$2" target="_blank" rel="noopener noreferrer" class="text-blue-400 hover:underline">$1</a>',
    )
    .replaceAll(/\n\n/gim, "</p><p>")
    .replaceAll(/\n/gim, "<br>");

  return `<p>${html}</p>`;
}

// ... (skipping unchanged parts)

function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${Number.parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

// ...

onMounted(() => {
  if (file.value) {
    loadPreview();
  }
  globalThis.addEventListener("keydown", handleKeydown);
});

onUnmounted(async () => {
  globalThis.removeEventListener("keydown", handleKeydown);
  await cleanupTempFile();

  if (previewUrl.value && previewUrl.value.startsWith("blob:")) {
    URL.revokeObjectURL(previewUrl.value);
  }
});

async function loadPreview() {
  // Reset zoom on file change
  resetZoom();

  if (!file.value) return;

  loading.value = true;
  error.value = null;

  try {
    if (isLocal.value) {
      await loadLocalPreview();
    } else {
      await loadRemotePreview();
    }
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : String(err);
    console.error("Failed to load preview:", errorMessage);
    error.value = errorMessage;
  } finally {
    loading.value = false;
  }
}

async function processPreviewFile(filePath: string) {
  if (previewType.value === "video" || previewType.value === "office") {
    return;
  }

  if (previewType.value === "image" || previewType.value === "pdf") {
    try {
      const fileContent = await readFile(filePath);
      const mimeType = getMimeType(fileType.value);
      const blob = new Blob([fileContent], { type: mimeType });
      previewUrl.value = URL.createObjectURL(blob);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error("Failed to create blob URL:", errorMessage);
      throw new Error(`Failed to load file: ${errorMessage}`);
    }
  } else if (previewType.value === "html") {
    const fileContent = await readFile(filePath);
    const decoder = new TextDecoder();
    htmlContent.value = decoder.decode(fileContent);
  } else if (previewType.value === "markdown") {
    const fileContent = await readFile(filePath);
    const decoder = new TextDecoder();
    markdownContent.value = decoder.decode(fileContent);
    renderedMarkdown.value = renderMarkdown(markdownContent.value);
  } else if (previewType.value === "text") {
    const fileContent = await readFile(filePath);
    const decoder = new TextDecoder();
    textContent.value = decoder.decode(fileContent);
  }
}

async function loadLocalPreview() {
  if (!file.value) return;
  await processPreviewFile(file.value.path);
}

function checkTransferStatus(transferId: string): boolean {
  const transfers = sftpStore.activeTransfers;
  const progress = transfers.find((t) => t.transferId === transferId);

  if (progress && progress.status === "completed") {
    return true;
  }

  if (progress && progress.status === "failed") {
    throw new Error(progress.error || "Download failed");
  }

  return false;
}

async function waitForDownload(transferId: string) {
  let attempts = 0;
  const maxAttempts = 60; // 30 seconds max wait time
  while (attempts < maxAttempts) {
    await new Promise((resolve) => setTimeout(resolve, 500));

    try {
      if (checkTransferStatus(transferId)) {
        return;
      }
    } catch (err) {
      // Re-throw if it's our "Download failed" error, otherwise ignore likely transient issues
      if (err instanceof Error && err.message === "Download failed") throw err;
      if (err instanceof Error && err.message.includes("Download failed"))
        throw err;
      // If we couldn't find the transfer, it might have been cleared or not started yet?
      // We'll continue waiting unless it's a specific error.
    }

    attempts++;
  }
  throw new Error("Download timed out");
}

async function loadRemotePreview() {
  if (!file.value || !sftpStore.activeSessionId) {
    throw new Error("No active SFTP session");
  }

  // For text files, check file size (limit to 10MB for preview)
  if (
    previewType.value === "text" &&
    file.value.size &&
    file.value.size > 10 * 1024 * 1024
  ) {
    error.value = "File too large to preview (max 10MB)";
    return;
  }

  await cleanupTempFile();

  const tempDirPath = await tempDir();
  const fileName = file.value.name;
  const tempFile = await join(
    tempDirPath,
    `sftp_preview_${Date.now()}_${fileName}`,
  );

  try {
    const transferId = await sftpStore.downloadFile(
      sftpStore.activeSessionId,
      file.value.path,
      tempFile,
    );

    await waitForDownload(transferId);

    // Short delay to ensure file system is ready
    try {
      await readFile(tempFile);
    } catch (err) {
      // Ignore initial read error, file might not be ready yet
      console.debug("Initial preview read failed, retrying...", err);
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    tempFilePath.value = tempFile;
    await processPreviewFile(tempFile);
  } catch (err) {
    await cleanupTempFile();
    const errorMessage = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to download file for preview: ${errorMessage}`);
  }
}

function getMimeType(ext: string): string {
  const mimeTypes: Record<string, string> = {
    jpg: "image/jpeg",
    jpeg: "image/jpeg",
    png: "image/png",
    gif: "image/gif",
    webp: "image/webp",
    svg: "image/svg+xml",
    ico: "image/x-icon",
    mp4: "video/mp4",
    webm: "video/webm",
    ogg: "video/ogg",
    avi: "video/x-msvideo",
    mov: "video/quicktime",
    wmv: "video/x-ms-wmv",
    flv: "video/x-flv",
    mkv: "video/x-matroska",
    pdf: "application/pdf",
    html: "text/html",
    htm: "text/html",
    md: "text/markdown",
  };
  return mimeTypes[ext] || "application/octet-stream";
}

function handleImageError() {
  error.value = "Failed to load image";
}

async function cleanupTempFile() {
  if (tempFilePath.value) {
    try {
      await remove(tempFilePath.value);
    } catch (err) {
      console.warn("Failed to cleanup temp file:", err);
    } finally {
      tempFilePath.value = null;
    }
  }
}

async function downloadFile() {
  if (!file.value) return;

  if (isLocal.value) {
    message.info("File is already local");
    return;
  }

  if (!sftpStore.activeSessionId) {
    throw new Error("No active SFTP session");
  }

  const localPathResult = await save({
    defaultPath: file.value.name,
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
      file.value.path,
      localPathResult,
    );
    message.success("Download started");
  }
}

function openInEditor() {
  if (!file.value) return;
  closeModal();

  setTimeout(() => {
    const { openOverlay } = useOverlay();
    openOverlay("sftp-file-editor-modal", {
      file: file.value,
      isLocal: isLocal.value,
    });
  }, 100);
}

async function closeModal() {
  if (previewUrl.value && previewUrl.value.startsWith("blob:")) {
    URL.revokeObjectURL(previewUrl.value);
  }

  await cleanupTempFile();

  previewUrl.value = "";
  htmlContent.value = "";
  markdownContent.value = "";
  renderedMarkdown.value = "";
  error.value = null;
  viewMode.value = "rendered";
  isFullscreen.value = false;
  resetZoom();
  closeOverlay("sftp-file-preview-modal");
}

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

// Keyboard navigation
function handleKeydown(e: KeyboardEvent) {
  if (e.key === "ArrowRight") {
    navigate("next");
  } else if (e.key === "ArrowLeft") {
    navigate("prev");
  } else if (e.key === "Escape") {
    if (isFullscreen.value) {
      isFullscreen.value = false;
    }
  }
}

watch(
  () => file.value,
  (fileValue) => {
    if (fileValue) {
      loadPreview();
    }
  },
  { immediate: true },
);
</script>

<style scoped>
/* Markdown preview styles */
:deep(.prose) {
  color: #e5e7eb;
}

:deep(.prose h1) {
  color: #f3f4f6;
  font-size: 2em;
  font-weight: 700;
  margin-top: 0.5em;
  margin-bottom: 0.5em;
}

:deep(.prose h2) {
  color: #f3f4f6;
  font-size: 1.5em;
  font-weight: 600;
  margin-top: 0.5em;
  margin-bottom: 0.5em;
}

:deep(.prose h3) {
  color: #f3f4f6;
  font-size: 1.25em;
  font-weight: 600;
  margin-top: 0.5em;
  margin-bottom: 0.5em;
}

:deep(.prose code) {
  background-color: #374151;
  color: #fbbf24;
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-size: 0.875em;
}

:deep(.prose pre) {
  background-color: #1f2937;
  padding: 1rem;
  border-radius: 0.5rem;
  overflow-x: auto;
  margin: 1rem 0;
}

:deep(.prose a) {
  color: #60a5fa;
  text-decoration: none;
}

:deep(.prose a:hover) {
  text-decoration: underline;
}

:deep(.prose p) {
  margin-bottom: 1em;
  line-height: 1.6;
}

:deep(.prose ul) {
  list-style-type: disc;
  padding-left: 1.5em;
  margin-bottom: 1em;
}

:deep(.prose ol) {
  list-style-type: decimal;
  padding-left: 1.5em;
  margin-bottom: 1em;
}

:deep(.prose blockquote) {
  border-left: 4px solid #4b5563;
  padding-left: 1rem;
  font-style: italic;
  color: #9ca3af;
  margin-bottom: 1em;
}
</style>
