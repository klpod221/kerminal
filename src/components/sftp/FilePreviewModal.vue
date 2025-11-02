<template>
  <Modal
    id="sftp-file-preview-modal"
    :title="`Preview ${file?.name || 'File'}`"
    :icon="previewIcon"
    icon-background="bg-purple-500/20"
    icon-color="text-purple-400"
    size="2xl"
  >
    <div v-if="loading" class="flex items-center justify-center py-12">
      <div class="text-gray-400">Loading preview...</div>
    </div>
    <div v-else-if="error" class="py-4">
      <div class="text-red-400 text-sm mb-4">{{ error }}</div>
      <Button variant="primary" @click="loadPreview">Retry</Button>
    </div>
    <div v-else class="flex flex-col gap-4">
      <!-- File Info -->
      <div class="text-xs text-gray-500 border-b border-gray-700 pb-2">
        <div class="flex items-center justify-between">
          <span>Previewing: <span class="font-mono text-gray-400">{{ file?.path }}</span></span>
          <span class="text-gray-600">{{ formatFileSize(file?.size || 0) }}</span>
        </div>
      </div>

      <!-- Image Preview -->
      <div v-if="previewType === 'image'" class="flex items-center justify-center bg-gray-900 rounded-lg p-4">
        <img
          :src="previewUrl"
          :alt="file?.name"
          class="max-w-full max-h-[70vh] object-contain rounded"
          @error="handleImageError"
        />
      </div>

      <!-- Video Preview - Show as document info -->
      <div v-else-if="previewType === 'video'" class="flex flex-col items-center justify-center py-12 gap-4">
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
      <div v-else-if="previewType === 'html'" class="flex flex-col gap-2">
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
        <div v-if="viewMode === 'rendered'" class="bg-gray-900 rounded-lg overflow-hidden" style="height: 70vh;">
          <iframe
            :srcdoc="htmlContent"
            class="w-full h-full border-0"
            sandbox="allow-same-origin allow-scripts"
          />
        </div>
        <div v-else class="bg-gray-900 rounded-lg p-4 overflow-auto" style="max-height: 70vh;">
          <pre class="text-xs text-gray-300 whitespace-pre-wrap font-mono">{{ htmlContent }}</pre>
        </div>
      </div>

      <!-- Markdown Preview -->
      <div v-else-if="previewType === 'markdown'" class="flex flex-col gap-2">
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
        <div v-if="viewMode === 'rendered'" class="bg-gray-900 rounded-lg p-6 overflow-auto prose prose-invert prose-sm max-w-none" style="max-height: 70vh;">
          <div v-html="renderedMarkdown" />
        </div>
        <div v-else class="bg-gray-900 rounded-lg p-4 overflow-auto" style="max-height: 70vh;">
          <pre class="text-xs text-gray-300 whitespace-pre-wrap font-mono">{{ markdownContent }}</pre>
        </div>
      </div>

      <!-- PDF Preview -->
      <div v-else-if="previewType === 'pdf'" class="bg-gray-900 rounded-lg overflow-hidden" style="height: 70vh;">
        <iframe
          :src="previewUrl"
          class="w-full h-full border-0"
          type="application/pdf"
        />
        <div v-if="!previewUrl" class="flex items-center justify-center h-full text-gray-400">
          <p>Your browser does not support PDFs. <a :href="previewUrl" download class="text-blue-400 hover:underline">Download the PDF</a>.</p>
        </div>
      </div>

      <!-- Office Documents Info -->
      <div v-else-if="previewType === 'office'" class="flex flex-col items-center justify-center py-12 gap-4">
        <component :is="FileText" :size="64" class="text-gray-500" />
        <div class="text-center">
          <p class="text-gray-300 mb-2">Office document preview is not available</p>
          <p class="text-sm text-gray-500 mb-4">
            File type: <span class="font-mono">{{ fileType }}</span>
          </p>
          <Button variant="primary" @click="downloadFile">
            Download to View
          </Button>
        </div>
      </div>

      <!-- Unknown/Unsupported -->
      <div v-else class="flex flex-col items-center justify-center py-12 gap-4">
        <component :is="FileQuestion" :size="64" class="text-gray-500" />
        <div class="text-center">
          <p class="text-gray-300 mb-2">Preview not available for this file type</p>
          <p class="text-sm text-gray-500 mb-4">
            File type: <span class="font-mono">{{ fileType }}</span>
          </p>
          <div class="flex gap-2 justify-center">
            <Button variant="ghost" @click="openInEditor" v-if="canEdit">
              Open in Editor
            </Button>
            <Button variant="primary" @click="downloadFile">
              Download
            </Button>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <Button variant="ghost" @click="closeModal">Close</Button>
      <Button
        v-if="previewType !== 'office' && previewType !== 'video' && previewType !== 'unknown'"
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
        v-if="canEdit && previewType !== 'image' && previewType !== 'video' && previewType !== 'pdf' && previewType !== 'office'"
        variant="primary"
        @click="openInEditor"
      >
        Edit
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import {
  FileText,
  Image,
  Video,
  FileQuestion,
  FileCode,
  File,
} from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import { useSFTPStore } from "../../stores/sftp";
import type { FileEntry } from "../../types/sftp";
import { readFile } from "@tauri-apps/plugin-fs";
import { tempDir, join } from "@tauri-apps/api/path";
import { save } from "@tauri-apps/plugin-dialog";

const { closeOverlay, getOverlayProp } = useOverlay();
const sftpStore = useSFTPStore();

const loading = ref(false);
const error = ref<string | null>(null);
const previewUrl = ref<string>("");
const htmlContent = ref<string>("");
const markdownContent = ref<string>("");
const renderedMarkdown = ref<string>("");
const viewMode = ref<"rendered" | "source">("rendered");
const tempFilePath = ref<string | null>(null);

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
  if (["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico"].includes(ext)) {
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

  // Office documents
  if (["doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp"].includes(ext)) {
    return "office";
  }

  return "unknown";
});

const previewIcon = computed(() => {
  switch (previewType.value) {
    case "image":
      return Image;
    case "video":
    case "office":
      return File;
    case "html":
    case "markdown":
      return FileCode;
    case "pdf":
      return File;
    default:
      return FileQuestion;
  }
});

const canEdit = computed(() => {
  return ["html", "markdown"].includes(previewType.value);
});

// Simple markdown renderer (basic implementation)
function renderMarkdown(md: string): string {
  // Convert headers
  let html = md
    .replace(/^### (.*$)/gim, "<h3>$1</h3>")
    .replace(/^## (.*$)/gim, "<h2>$1</h2>")
    .replace(/^# (.*$)/gim, "<h1>$1</h1>")
    // Convert bold
    .replace(/\*\*(.*?)\*\*/gim, "<strong>$1</strong>")
    .replace(/__(.*?)__/gim, "<strong>$1</strong>")
    // Convert italic
    .replace(/\*(.*?)\*/gim, "<em>$1</em>")
    .replace(/_(.*?)_/gim, "<em>$1</em>")
    // Convert code blocks
    .replace(/```([\s\S]*?)```/gim, "<pre><code>$1</code></pre>")
    .replace(/`(.*?)`/gim, "<code>$1</code>")
    // Convert links
    .replace(/\[([^\]]+)\]\(([^)]+)\)/gim, '<a href="$2" target="_blank" rel="noopener noreferrer" class="text-blue-400 hover:underline">$1</a>')
    // Convert line breaks
    .replace(/\n\n/gim, "</p><p>")
    .replace(/\n/gim, "<br>");

  return `<p>${html}</p>`;
}

async function loadPreview() {
  if (!file.value) return;

  loading.value = true;
  error.value = null;

  try {
    if (isLocal.value) {
      // Load local file
      await loadLocalPreview();
    } else {
      // Load remote file - download to temp first
      await loadRemotePreview();
    }
  } catch (err) {
    console.error("Failed to load preview:", err);
    error.value = getErrorMessage(err, "Failed to load preview");
  } finally {
    loading.value = false;
  }
}

async function loadLocalPreview() {
  if (!file.value) return;

  // Videos are treated like office docs - no preview needed
  if (previewType.value === "video" || previewType.value === "office") {
    // No preview needed for videos and office docs
    return;
  }

  if (previewType.value === "image" || previewType.value === "pdf") {
    // For binary files, create blob URL
    try {
      const fileContent = await readFile(file.value.path);
      const mimeType = getMimeType(fileType.value);
      console.log(`Creating blob for ${file.value.name}, type: ${mimeType}, size: ${fileContent.length} bytes`);
      const blob = new Blob([fileContent], { type: mimeType });
      previewUrl.value = URL.createObjectURL(blob);
      console.log(`Blob URL created: ${previewUrl.value}`);
    } catch (err) {
      console.error("Failed to create blob URL:", err);
      throw new Error(`Failed to load file: ${getErrorMessage(err, "Unknown error")}`);
    }
  } else if (previewType.value === "html") {
    const fileContent = await readFile(file.value.path);
    const decoder = new TextDecoder();
    htmlContent.value = decoder.decode(fileContent);
  } else if (previewType.value === "markdown") {
    const fileContent = await readFile(file.value.path);
    const decoder = new TextDecoder();
    markdownContent.value = decoder.decode(fileContent);
    renderedMarkdown.value = renderMarkdown(markdownContent.value);
  }
}

async function loadRemotePreview() {
  if (!file.value || !sftpStore.activeSessionId) {
    throw new Error("No active SFTP session");
  }

  // Download file to temp directory
  const tempDirPath = await tempDir();
  const fileName = file.value.name;
  const tempFile = await join(
    tempDirPath,
    `sftp_preview_${Date.now()}_${fileName}`,
  );

  try {
    // Start download
    const transferId = await sftpStore.downloadFile(
      sftpStore.activeSessionId,
      file.value.path,
      tempFile,
    );

    // Wait for download to complete by polling transfer progress
    let attempts = 0;
    const maxAttempts = 60; // 30 seconds max wait time
    while (attempts < maxAttempts) {
      await new Promise((resolve) => setTimeout(resolve, 500));

      try {
        const transfers = sftpStore.activeTransfers;
        const progress = transfers.find(
          (t) => t.transferId === transferId
        );

        if (progress && progress.status === "completed") {
          break;
        }

        if (progress && progress.status === "failed") {
          throw new Error(progress.error || "Download failed");
        }
      } catch (err) {
        // Continue polling - might not be in transfers list yet
      }

      attempts++;
    }

    // Check if file exists
    try {
      await readFile(tempFile);
    } catch (err) {
      // File might not be ready yet, wait a bit more
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    tempFilePath.value = tempFile;

    // Videos are treated like office docs - no preview needed
    if (previewType.value === "video" || previewType.value === "office") {
      // No preview needed for videos and office docs
      return;
    }

    if (previewType.value === "image" || previewType.value === "pdf") {
      try {
        const fileContent = await readFile(tempFile);
        const mimeType = getMimeType(fileType.value);
        console.log(`Creating blob for remote ${file.value?.name}, type: ${mimeType}, size: ${fileContent.length} bytes`);
        const blob = new Blob([fileContent], { type: mimeType });
        previewUrl.value = URL.createObjectURL(blob);
        console.log(`Blob URL created: ${previewUrl.value}`);
      } catch (err) {
        console.error("Failed to create blob URL for remote file:", err);
        throw new Error(`Failed to load file: ${getErrorMessage(err, "Unknown error")}`);
      }
    } else if (previewType.value === "html") {
      const fileContent = await readFile(tempFile);
      const decoder = new TextDecoder();
      htmlContent.value = decoder.decode(fileContent);
    } else if (previewType.value === "markdown") {
      const fileContent = await readFile(tempFile);
      const decoder = new TextDecoder();
      markdownContent.value = decoder.decode(fileContent);
      renderedMarkdown.value = renderMarkdown(markdownContent.value);
    }
  } catch (err) {
    throw new Error(`Failed to download file for preview: ${getErrorMessage(err, "Unknown error")}`);
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


async function downloadFile() {
  if (!file.value) return;

  try {
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
  } catch (err) {
    console.error("Failed to download file:", err);
    message.error(getErrorMessage(err, "Failed to download file"));
  }
}

function openInEditor() {
  if (!file.value) return;
  closeModal();

  // Open in editor modal
  setTimeout(() => {
    const { openOverlay } = useOverlay();
    openOverlay("sftp-file-editor-modal", {
      file: file.value,
      isLocal: isLocal.value,
    });
  }, 100);
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

function closeModal() {
  // Cleanup blob URLs
  if (previewUrl.value.startsWith("blob:")) {
    URL.revokeObjectURL(previewUrl.value);
  }

  // Cleanup temp file
  if (tempFilePath.value) {
    // Note: In production, you might want to schedule cleanup of temp files
    // For now, Tauri will handle temp directory cleanup
    tempFilePath.value = null;
  }

  previewUrl.value = "";
  htmlContent.value = "";
  markdownContent.value = "";
  renderedMarkdown.value = "";
  error.value = null;
  viewMode.value = "rendered";
  closeOverlay("sftp-file-preview-modal");
}

// Load preview when modal opens
watch(
  () => file.value,
  (fileValue) => {
    if (fileValue) {
      loadPreview();
    }
  },
  { immediate: true },
);

onMounted(() => {
  if (file.value) {
    loadPreview();
  }
});
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

:deep(.prose pre code) {
  background-color: transparent;
  color: #e5e7eb;
  padding: 0;
}

:deep(.prose a) {
  color: #60a5fa;
  text-decoration: underline;
}

:deep(.prose a:hover) {
  color: #93c5fd;
}

:deep(.prose strong) {
  font-weight: 700;
  color: #f3f4f6;
}

:deep(.prose em) {
  font-style: italic;
}
</style>

