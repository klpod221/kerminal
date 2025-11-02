<template>
  <div class="code-editor-wrapper">
    <label v-if="label" class="block text-sm font-medium text-gray-300 mb-2">
      {{ label }}
    </label>
    <div
      class="border border-gray-700 rounded-lg overflow-hidden"
      :class="{ 'border-red-500': error }"
    >
      <CodeEditor
        v-model:value="code"
        :language="language"
        :theme="theme"
        :options="editorOptions"
        :height="height"
        @change="handleChange"
        @mount="handleMount"
      />
    </div>
    <p v-if="error" class="text-xs text-red-400 flex items-center gap-1 mt-2">
      <component :is="AlertCircle" class="w-3 h-3" />
      {{ error }}
    </p>
    <p v-else-if="helperText" class="text-xs text-gray-500 mt-2">
      {{ helperText }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { CodeEditor } from "monaco-editor-vue3";
import { AlertCircle } from "lucide-vue-next";
import type { editor } from "monaco-editor";

interface CodeEditorProps {
  modelValue: string;
  language?: string;
  theme?: string;
  height?: string;
  label?: string;
  error?: string;
  helperText?: string;
  readOnly?: boolean;
  minimap?: boolean;
  lineNumbers?: boolean;
  formatOnPaste?: boolean;
  formatOnType?: boolean;
}

const props = withDefaults(defineProps<CodeEditorProps>(), {
  language: "json",
  theme: "vs-dark",
  height: "300px",
  readOnly: false,
  minimap: false,
  lineNumbers: true,
  formatOnPaste: true,
  formatOnType: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: string];
  change: [value: string];
  mount: [editor: editor.IStandaloneCodeEditor];
}>();

const code = ref(props.modelValue);

const editorOptions = computed(() => ({
  automaticLayout: true,
  formatOnPaste: props.formatOnPaste,
  formatOnType: props.formatOnType,
  readOnly: props.readOnly,
  minimap: {
    enabled: props.minimap,
  },
  lineNumbers: props.lineNumbers ? "on" : "off",
  scrollBeyondLastLine: false,
  wordWrap: "on",
  wrappingIndent: "indent",
  tabSize: 2,
  fontSize: 13,
  fontFamily: "'FiraCode Nerd Font', 'Courier New', monospace",
  scrollbar: {
    vertical: "auto",
    horizontal: "auto",
    verticalScrollbarSize: 8,
    horizontalScrollbarSize: 8,
  },
  padding: {
    top: 12,
    bottom: 12,
  },
}));

watch(
  () => props.modelValue,
  (newValue) => {
    if (newValue !== code.value) {
      code.value = newValue;
    }
  },
);

const handleChange = (value: string) => {
  emit("update:modelValue", value);
  emit("change", value);
};

const handleMount = (editor: editor.IStandaloneCodeEditor) => {
  emit("mount", editor);
};
</script>

<style scoped>
.code-editor-wrapper {
  position: relative;
}

:deep(.monaco-editor) {
  border-radius: 0.5rem;
}

:deep(.monaco-editor .margin),
:deep(.monaco-editor .monaco-editor-background) {
  background-color: #1f2937;
}

:deep(.monaco-editor .line-numbers) {
  color: #6b7280;
}
</style>
