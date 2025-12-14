<template>
  <div class="space-y-0.5">
    <!-- Label -->
    <label
      v-if="label"
      :for="inputId"
      class="block text-sm font-medium text-gray-300"
    >
      {{ label }}
      <span
        v-if="props.rules && props.rules.includes('required')"
        class="text-red-400"
        >*</span
      >
    </label>

    <div
      class="simple-code-editor w-full rounded-lg border transition-all duration-200 overflow-hidden relative"
      :class="[
        stateClasses,
        {
          'opacity-50 cursor-not-allowed': disabled,
          'fit-content': fitContent
        }
      ]"
      :style="{ height: fitContent ? 'auto' : height }"
    >
      <div class="editor-container w-full h-full relative">
        <!-- Highlighted code (visual layer) -->
        <pre
          ref="preElement"
          class="highlight-layer"
          :class="`language-${language}`"
          aria-hidden="true"
        ><code ref="codeElement" :class="`language-${language}`" v-html="highlightedCode"></code></pre>

        <!-- Textarea (input layer) -->
        <textarea
          :id="inputId"
          ref="textareaElement"
          class="input-layer focus:outline-none"
          :value="modelValue"
          :disabled="disabled || readOnly"
          :readonly="readOnly"
          autocapitalize="off"
          autocomplete="off"
          autocorrect="off"
          spellcheck="false"
          @input="handleInput"
          @keydown="handleKeydown"
          @scroll="syncScroll"
          @blur="handleBlur"
          @focus="handleFocus"
        ></textarea>
      </div>
    </div>

    <!-- Helper text (only show if no error) -->
    <p v-if="helperText && !generatedErrorMessage" class="text-xs text-gray-400 min-h-[1.25rem]">
      {{ helperText }}
    </p>

    <!-- Error message -->
    <p v-if="generatedErrorMessage" class="text-xs text-red-400 flex items-center min-h-[1.25rem]">
      <component :is="TriangleAlert" class="mr-1 w-3 h-3" />
      {{ generatedErrorMessage }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import Prism from "prismjs";
import { TriangleAlert } from "lucide-vue-next";
import { useFormField } from "../../composables/useFormField";
import { useFormStyles } from "../../composables/useFormStyles";
import "prismjs/themes/prism-tomorrow.css"; // Dark theme
import "prismjs/components/prism-json";
import "prismjs/components/prism-bash";
import "prismjs/components/prism-yaml";
import "prismjs/components/prism-css";
import "prismjs/components/prism-javascript";
import "prismjs/components/prism-typescript";
import "prismjs/components/prism-markdown";

interface Props {
  // Common Form Props
  id: string; // Made required to match FormFieldProps better, though useFormField handles it if missing
  modelValue: string;
  label?: string;
  placeholder?: string;
  rules?: string;
  helperText?: string;
  errorMessage?: string; // Renamed from error
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
  readOnly?: boolean;

  // Editor specific
  language?: string;
  height?: string;
  fitContent?: boolean; // New prop for auto-height
}

const props = withDefaults(defineProps<Props>(), {
  language: "plaintext",
  height: "300px",
  size: "md",
  disabled: false,
  readOnly: false,
  fitContent: false,
});

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "change", value: string): void;
  (e: "blur", event: FocusEvent): void;
  (e: "focus", event: FocusEvent): void;
  (e: "keydown", event: KeyboardEvent): void;
}>();

// Wrapper for useFormField to match interface
// We can now pass props directly if we align interfaces, but props is a Proxy of Props
// useFormField expects FormFieldProps. Our Props has id, modelValue, rules, errorMessage.
// So we can pass props directly.
const {
  errorMessage: generatedErrorMessage,
  touched, // Need touched for validation logic
  inputId,
  validate,
  handleBlur,
  handleFocus: originalHandleFocus,
  handleKeydown: originalHandleKeydown,
} = useFormField(props, emit);

const { stateClasses } = useFormStyles(props);

const textareaElement = ref<HTMLTextAreaElement | null>(null);
const preElement = ref<HTMLPreElement | null>(null);

const highlightedCode = computed(() => {
  const code = props.modelValue || "";
  // Ensure the last newline is visible
  const content = code.endsWith("\n") ? code + " " : code;

  try {
    if (Prism.languages[props.language]) {
      return Prism.highlight(
        content,
        Prism.languages[props.language],
        props.language
      );
    }
    return Prism.util.encode(content);
  } catch (e) {
    return Prism.util.encode(content);
  }
});

const handleInput = (e: Event) => {
  const target = e.target as HTMLTextAreaElement;
  const newValue = target.value;
  emit("update:modelValue", newValue);
  emit("change", newValue);
  syncScroll();

  // Validation logic from Textarea.vue
  if (touched.value) {
    validate();
  }
};

const handleFocus = (event: FocusEvent) => {
  originalHandleFocus(event);
};

const handleKeydown = (e: KeyboardEvent) => {
  originalHandleKeydown(e);

  if (props.readOnly || props.disabled) return;

  if (e.key === "Tab") {
    e.preventDefault();
    const textarea = textareaElement.value;
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const spaces = "  "; // 2 spaces

    const newValue =
      props.modelValue.substring(0, start) +
      spaces +
      props.modelValue.substring(end);

    emit("update:modelValue", newValue);
    emit("change", newValue);

    nextTick(() => {
      textarea.selectionStart = textarea.selectionEnd = start + spaces.length;
    });
  }
};

const syncScroll = () => {
  // If fitContent is true, we rely on parent scroll, so no internal sync needed?
  // But textarea is still separate from pre.
  // Actually if fitContent: textarea matches parent height (which is pre height).
  // Both are full height. No internal scroll.
  if (props.fitContent) return;

  if (textareaElement.value && preElement.value) {
    preElement.value.scrollTop = textareaElement.value.scrollTop;
    preElement.value.scrollLeft = textareaElement.value.scrollLeft;
  }
};

watch(() => props.modelValue, () => {
  nextTick(syncScroll);
});
</script>

<style scoped>


.editor-container {
  position: relative;
  width: 100%;
  height: 100%;
}

.highlight-layer,
.input-layer {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  padding: 0.75rem; /* px-3 equivalent roughly */
  margin: 0;
  border: 0;
  font-family: "FiraCode Nerd Font", "Courier New", monospace;
  font-size: 13px;
  line-height: 1.5;
  tab-size: 2;
  white-space: pre;
  overflow: auto;
  box-sizing: border-box;
  text-align: left;
}

/* Modifiers for fit-content mode */
.fit-content .highlight-layer {
  position: relative;
  height: auto;
  min-height: 100%;
  overflow: hidden; /* No scroll, let parent scroll */
}

.fit-content .input-layer {
  position: absolute;
  height: 100%;
  overflow: hidden; /* No scroll */
}

.highlight-layer {
  pointer-events: none;
  z-index: 1;
  background: transparent !important;
}

/* Override Prism's pre/code background */
:deep(pre[class*="language-"]),
:deep(code[class*="language-"]) {
  text-shadow: none !important;
  background: transparent !important;
}

.input-layer {
  z-index: 2;
  color: transparent;
  background: transparent;
  caret-color: #d4d4d4; /* Light cursor */
  resize: none;
}
</style>
