<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <div class="code-editor-wrapper w-full">
    <span v-if="label" class="block text-sm font-medium text-gray-300 mb-1.5">
      {{ label }}
    </span>
    <div
      ref="editorContainer"
      class="code-editor-container border border-gray-700 bg-[#121212] rounded-lg overflow-hidden transition-colors focus-within:border-blue-500/70"
      :class="{
        'border-red-500': error,
        'opacity-60 cursor-not-allowed': disabled,
      }"
      :style="containerStyle"
    ></div>
    <p v-if="error" class="text-xs text-red-400 flex items-center gap-1 mt-1.5">
      <component :is="AlertCircle" class="w-3.5 h-3.5 flex-shrink-0" />
      {{ error }}
    </p>
    <p v-else-if="helperText" class="text-xs text-gray-400 mt-1.5">
      {{ helperText }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from "vue";
import { AlertCircle } from "lucide-vue-next";
import {
  EditorView,
  lineNumbers as cmLineNumbers,
  highlightActiveLineGutter,
  highlightSpecialChars,
  drawSelection,
  dropCursor,
  rectangularSelection,
  crosshairCursor,
  highlightActiveLine,
  keymap,
  placeholder as cmPlaceholder,
} from "@codemirror/view";
import { EditorState, Compartment } from "@codemirror/state";
import {
  defaultHighlightStyle,
  syntaxHighlighting,
  indentOnInput,
  bracketMatching,
  foldGutter,
  foldKeymap,
} from "@codemirror/language";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete";
import { languages } from "@codemirror/language-data";
import { kerminalEditorTheme } from "../../utils/codemirrorTheme";

export interface CodeEditorProps {
  modelValue?: string;
  language?: string;
  height?: string;
  minHeight?: string;
  maxHeight?: string;
  label?: string;
  placeholder?: string;
  error?: string;
  helperText?: string;
  readOnly?: boolean;
  disabled?: boolean;
  lineNumbers?: boolean;
  foldGutter?: boolean;
  autofocus?: boolean;
  fitContent?: boolean;
}

const props = withDefaults(defineProps<CodeEditorProps>(), {
  modelValue: "",
  language: "text",
  height: "100%",
  minHeight: "80px",
  readOnly: false,
  disabled: false,
  lineNumbers: true,
  foldGutter: true,
  autofocus: false,
  fitContent: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: string];
  change: [value: string];
  mount: [view: EditorView];
}>();

const editorContainer = ref<HTMLDivElement | null>(null);
let view: EditorView | null = null;

// Dynamic configuration compartments
const readOnlyCompartment = new Compartment();
const languageCompartment = new Compartment();
const lineNumbersCompartment = new Compartment();

const containerStyle = computed(() => {
  if (props.fitContent) {
    return {
      height: "auto",
      minHeight: props.minHeight,
      maxHeight: props.maxHeight,
    };
  }
  return {
    height: props.height,
    minHeight: props.minHeight,
    maxHeight: props.maxHeight,
  };
});

async function getLanguageExtension(langName: string) {
  const normalized = langName.toLowerCase().trim();
  const langDesc = languages.find(
    (l) =>
      l.name.toLowerCase() === normalized ||
      l.alias.some((a) => a.toLowerCase() === normalized) ||
      l.extensions.some((ext) => ext.toLowerCase() === normalized),
  );

  if (langDesc) {
    try {
      const support = await langDesc.load();
      return support;
    } catch (e) {
      console.warn(`[CodeEditor] Failed to load language '${langName}':`, e);
    }
  }
  return [];
}

async function initEditor() {
  if (!editorContainer.value) return;

  const initialLang = await getLanguageExtension(props.language);

  const extensions = [
    // Line Numbers & Gutters
    props.lineNumbers
      ? lineNumbersCompartment.of([cmLineNumbers(), highlightActiveLineGutter()])
      : lineNumbersCompartment.of([]),
    props.foldGutter ? foldGutter() : [],
    highlightSpecialChars(),
    history(),
    drawSelection(),
    dropCursor(),
    EditorState.allowMultipleSelections.of(true),
    indentOnInput(),
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    bracketMatching(),
    closeBrackets(),
    autocompletion(),
    rectangularSelection(),
    crosshairCursor(),
    highlightActiveLine(),
    highlightSelectionMatches(),

    // Keymaps
    keymap.of([
      ...closeBracketsKeymap,
      ...defaultKeymap,
      ...searchKeymap,
      ...historyKeymap,
      ...foldKeymap,
      ...completionKeymap,
      indentWithTab,
    ]),

    // Kerminal Dark Catppuccin Theme
    kerminalEditorTheme,

    // Dynamic Language
    languageCompartment.of(initialLang),

    // Readonly / Editable state
    readOnlyCompartment.of(EditorState.readOnly.of(props.readOnly || props.disabled)),

    // Change listener
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        const newValue = update.state.doc.toString();
        emit("update:modelValue", newValue);
        emit("change", newValue);
      }
    }),
  ];

  if (props.placeholder) {
    extensions.push(cmPlaceholder(props.placeholder));
  }

  // Handle auto-expansion and custom styling
  extensions.push(
    EditorView.theme({
      "&": {
        height: props.fitContent ? "auto" : "100%",
        maxHeight: props.maxHeight || "none",
      },
      ".cm-scroller": {
        overflow: props.fitContent ? "visible" : "auto",
        fontFamily: "inherit",
      },
    }),
  );

  const state = EditorState.create({
    doc: props.modelValue,
    extensions,
  });

  view = new EditorView({
    state,
    parent: editorContainer.value,
  });

  if (props.autofocus) {
    view.focus();
  }

  emit("mount", view);
}

// Watchers for reactive props
watch(
  () => props.modelValue,
  (newVal) => {
    if (!view) return;
    const currentVal = view.state.doc.toString();
    if (newVal !== currentVal) {
      view.dispatch({
        changes: { from: 0, to: currentVal.length, insert: newVal || "" },
      });
    }
  },
);

watch(
  () => props.language,
  async (newLang) => {
    if (!view) return;
    const langExt = await getLanguageExtension(newLang);
    view.dispatch({
      effects: languageCompartment.reconfigure(langExt),
    });
  },
);

watch(
  () => [props.readOnly, props.disabled],
  ([isReadOnly, isDisabled]) => {
    if (!view) return;
    view.dispatch({
      effects: readOnlyCompartment.reconfigure(EditorState.readOnly.of(Boolean(isReadOnly || isDisabled))),
    });
  },
);

watch(
  () => props.lineNumbers,
  (showLineNumbers) => {
    if (!view) return;
    view.dispatch({
      effects: lineNumbersCompartment.reconfigure(
        showLineNumbers ? [cmLineNumbers(), highlightActiveLineGutter()] : [],
      ),
    });
  },
);

onMounted(() => {
  initEditor();
});

onBeforeUnmount(() => {
  if (view) {
    view.destroy();
    view = null;
  }
});

// Expose editor API methods
function focus() {
  view?.focus();
}

function revealLine(lineNumber: number) {
  if (!view || lineNumber < 1) return;
  try {
    const doc = view.state.doc;
    const line = doc.line(Math.min(lineNumber, doc.lines));
    view.dispatch({
      selection: { anchor: line.from },
      scrollIntoView: true,
    });
    view.focus();
  } catch (e) {
    console.error("[CodeEditor] Failed to reveal line:", e);
  }
}

function getContent(): string {
  return view ? view.state.doc.toString() : props.modelValue;
}

defineExpose({
  focus,
  revealLine,
  getContent,
  view: computed(() => view),
});
</script>

<style scoped>
:deep(.cm-editor) {
  outline: none !important;
}
:deep(.cm-scroller) {
  line-height: 1.6;
}
</style>
