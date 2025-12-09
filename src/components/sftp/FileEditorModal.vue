<template>
  <Modal
    id="sftp-file-editor-modal"
    :title="`Edit ${file?.name || 'File'}`"
    :icon="FileText"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    size="6xl"
  >
    <div v-if="loading" class="flex items-center justify-center py-8">
      <div class="text-gray-400">Loading file...</div>
    </div>
    <div v-else-if="error" class="py-4">
      <div class="text-red-400 text-sm">{{ error }}</div>
    </div>
    <div v-else class="flex flex-col gap-4">
      <div class="text-xs text-gray-500">
        Editing: <span class="font-mono text-gray-400">{{ file?.path }}</span>
      </div>
      <CodeEditor
        v-model="content"
        :language="language"
        :height="'600px'"
        :minimap="true"
        :line-numbers="true"
        @mount="handleEditorMount"
      />
    </div>

    <template #footer>
      <Button variant="ghost" @click="closeModal">Cancel</Button>
      <Button
        variant="primary"
        :loading="saving"
        :disabled="loading || !!error"
        @click="handleSave"
      >
        Save
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { FileText } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import CodeEditor from "../ui/CodeEditor.vue";
import { useOverlay } from "../../composables/useOverlay";
import { message } from "../../utils/message";
import { useSFTPStore } from "../../stores/sftp";
import type { FileEntry } from "../../types/sftp";
import { readFile, writeFile } from "@tauri-apps/plugin-fs";

import type { editor } from "monaco-editor";

const { closeOverlay, getOverlayProp } = useOverlay();
const sftpStore = useSFTPStore();

const loading = ref(false);
const saving = ref(false);
const error = ref<string | null>(null);
const content = ref("");
const editorInstance = ref<editor.IStandaloneCodeEditor | null>(null);

const file = getOverlayProp<FileEntry | null>(
  "sftp-file-editor-modal",
  "file",
  null,
  null,
);

const initialLine = getOverlayProp<number | undefined>(
  "sftp-file-editor-modal",
  "line",
  undefined,
  undefined,
);

const isLocal = getOverlayProp<boolean>(
  "sftp-file-editor-modal",
  "isLocal",
  false,
  false,
);

const language = computed(() => {
  if (!file.value) return "plaintext";
  const ext = file.value.name.split(".").pop()?.toLowerCase();
  const languageMap: Record<string, string> = {
    js: "javascript",
    jsx: "javascript",
    ts: "typescript",
    tsx: "typescript",
    json: "json",
    yaml: "yaml",
    yml: "yaml",
    xml: "xml",
    html: "html",
    css: "css",
    scss: "scss",
    sass: "sass",
    py: "python",
    rs: "rust",
    go: "go",
    java: "java",
    c: "c",
    cpp: "cpp",
    h: "c",
    hpp: "cpp",
    sh: "shell",
    bash: "shell",
    zsh: "shell",
    md: "markdown",
    sql: "sql",
    vue: "vue",
    php: "php",
    rb: "ruby",
    txt: "plaintext",
    log: "plaintext",
    ini: "ini",
    conf: "ini",
    toml: "toml",
    dockerfile: "dockerfile",
    makefile: "makefile",
  };
  return languageMap[ext || ""] || "plaintext";
});

/**
 * Load file content into editor with error handling
 * Supports both local and remote files
 */
async function loadFileContent() {
  if (!file.value) return;

  loading.value = true;
  error.value = null;

  try {
    const MAX_FILE_SIZE = 10 * 1024 * 1024;
    if (file.value.size && file.value.size > MAX_FILE_SIZE) {
      throw new Error(
        `File too large to edit (${formatFileSize(file.value.size)}). Maximum size is 10MB`,
      );
    }

    if (isLocal.value) {
      const fileContent = await readFile(file.value.path);
      const decoder = new TextDecoder();
      const decoded = decoder.decode(fileContent);

      if (decoded.includes("\0")) {
        throw new Error("File appears to be binary, not text");
      }

      content.value = decoded;
    } else {
      if (!sftpStore.activeSessionId) {
        throw new Error("No active SFTP session");
      }
      content.value = await sftpStore.readFile(
        sftpStore.activeSessionId,
        file.value.path,
      );
    }

    // Scroll to line if specified
    if (initialLine.value && editorInstance.value) {
      // Small delay to ensure editor has updated model
      setTimeout(() => {
        if (editorInstance.value && initialLine.value) {
          editorInstance.value.revealLineInCenter(initialLine.value);
          editorInstance.value.setPosition({
            lineNumber: initialLine.value,
            column: 1,
          });
          editorInstance.value.focus();
        }
      }, 100);
    }
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : String(err);
    console.error("Failed to load file:", errorMessage);
    error.value = errorMessage;
  } finally {
    loading.value = false;
  }
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${Number.parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

/**
 * Save file content with error handling
 * Supports both local and remote file saving
 */
async function saveLocalFile() {
  if (!file.value || !content.value) return;

  const encoder = new TextEncoder();
  const fileContent = encoder.encode(content.value);
  await writeFile(file.value.path, fileContent);
  message.success("File saved successfully");

  if (sftpStore.browserState.localPath) {
    await sftpStore.listLocalDirectory(sftpStore.browserState.localPath);
  }
}

async function saveRemoteFile() {
  if (!file.value || !content.value) return;
  if (!sftpStore.activeSessionId) {
    throw new Error("No active SFTP session");
  }

  await sftpStore.writeFile(
    sftpStore.activeSessionId,
    file.value.path,
    content.value,
  );
  message.success("File saved successfully");

  if (sftpStore.activeSessionId && sftpStore.browserState.remotePath) {
    await sftpStore.listRemoteDirectory(
      sftpStore.activeSessionId,
      sftpStore.browserState.remotePath,
    );
  }
}

/**
 * Save file content with error handling
 * Supports both local and remote file saving
 */
async function handleSave() {
  if (!file.value || !content.value) return;

  saving.value = true;

  try {
    if (isLocal.value) {
      await saveLocalFile();
    } else {
      await saveRemoteFile();
    }

    closeModal();
  } catch (err) {
    if (isLocal.value) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error("Failed to save file:", errorMessage);
      message.error(errorMessage);
    } else {
      throw err;
    }
  } finally {
    saving.value = false;
  }
}

function closeModal() {
  content.value = "";
  error.value = null;
  closeOverlay("sftp-file-editor-modal");
}

function handleEditorMount(editor: editor.IStandaloneCodeEditor) {
  editorInstance.value = editor;
}

watch(
  () => file.value,
  (fileValue) => {
    if (fileValue) {
      loadFileContent();
    }
  },
  { immediate: true },
);

onMounted(() => {
  if (file.value) {
    loadFileContent();
  }
});
</script>
