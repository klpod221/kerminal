<template>
  <Modal
    id="custom-theme-modal"
    :title="isEditing ? 'Edit Custom Theme' : 'Create Custom Theme'"
    :icon="Palette"
    icon-background="bg-purple-500/20"
    icon-color="text-purple-400"
    size="lg"
    :show-close-button="true"
  >
    <div class="space-y-4">
      <!-- Theme Name -->
      <Input
        id="theme-name"
        v-model="themeName"
        label="Theme Name"
        placeholder="My Custom Theme"
        rules="required|min:3|max:50"
      />

      <!-- JSON Editor -->
      <div class="space-y-2">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm font-medium text-gray-300"> Theme JSON </span>
          <button
            class="text-xs text-purple-400 hover:text-purple-300 transition-colors px-2 py-1 rounded hover:bg-purple-500/10"
            @click="copyTemplate"
          >
            Copy Template
          </button>
        </div>
        <SimpleCodeEditor
          id="theme-json-editor"
          v-model="jsonInput"
          language="json"
          height="250px"
          rules="required"
          :error-message="jsonError"
          @change="handleJsonInput"
        />
      </div>

      <!-- Quick Reference -->
      <Collapsible title="Available Color Properties" :default-expanded="false">
        <div
          class="p-3 bg-gray-800 rounded text-xs text-gray-400 font-mono space-y-1"
        >
          <div>
            <span class="text-purple-400">background</span> * - Background color
          </div>
          <div>
            <span class="text-purple-400">foreground</span> * - Text color
          </div>
          <div><span class="text-gray-500">cursor</span> - Cursor color</div>
          <div class="pt-2 text-gray-500">Normal colors:</div>
          <div>
            <span class="text-gray-500"
              >black, red, green, yellow, blue, magenta, cyan, white</span
            >
          </div>
          <div class="pt-2 text-gray-500">Bright colors:</div>
          <div>
            <span class="text-gray-500"
              >brightBlack, brightRed, brightGreen, brightYellow, brightBlue,
              brightMagenta, brightCyan, brightWhite</span
            >
          </div>
        </div>
      </Collapsible>

      <!-- Preview -->
      <div v-if="!jsonError && parsedColors" class="mt-4">
        <div class="text-sm font-medium text-gray-300 mb-2">Preview</div>
        <div
          class="w-full h-24 rounded-lg border border-gray-600 p-4 font-mono text-sm"
          :style="{
            backgroundColor: parsedColors.background,
            color: parsedColors.foreground,
          }"
        >
          <div>
            $
            <span :style="{ color: parsedColors.green || '#00ff00' }"
              >echo</span
            >
            <span :style="{ color: parsedColors.yellow || '#ffff00' }"
              >"Hello World"</span
            >
          </div>
          <div :style="{ color: parsedColors.green || '#00ff00' }">
            Hello World
          </div>
          <div>
            $
            <span :style="{ color: parsedColors.cyan || '#00ffff' }">ls</span>
            <span :style="{ color: parsedColors.blue || '#0000ff' }">-la</span>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2 w-full">
        <Button variant="outline" @click="closeOverlay('custom-theme-modal')">
          Cancel
        </Button>
        <Button
          variant="primary"
          :icon="Save"
          :disabled="!isValid"
          @click="handleSave"
        >
          {{ isEditing ? "Update" : "Create" }}
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { Palette, Save } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import Input from "../ui/Input.vue";
import SimpleCodeEditor from "../ui/SimpleCodeEditor.vue";
import Collapsible from "../ui/Collapsible.vue";
import { useSettingsStore } from "../../stores/settings";
import { useOverlay } from "../../composables/useOverlay";
import { message } from "../../utils/message";
import type { TerminalTheme } from "../../utils/terminalTheme";

const settingsStore = useSettingsStore();
const { closeOverlay, getOverlayProp } = useOverlay();

const themeId = getOverlayProp("custom-theme-modal", "themeId", null, null);

const themeName = ref("");
const jsonInput = ref("");
const jsonError = ref("");
const parsedColors = ref<TerminalTheme | null>(null);

const isEditing = computed(() => !!themeId.value);
const isValid = computed(() => {
  return (
    themeName.value.trim().length >= 3 &&
    !jsonError.value &&
    parsedColors.value !== null &&
    parsedColors.value.background &&
    parsedColors.value.foreground
  );
});

const handleJsonInput = () => {
  try {
    jsonError.value = "";
    const parsed = JSON.parse(jsonInput.value);

    // Validate required fields
    if (!parsed.background || !parsed.foreground) {
      jsonError.value = "Required fields: background and foreground";
      parsedColors.value = null;
      return;
    }

    // Validate hex colors
    const hexRegex = /^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$/;
    for (const [key, value] of Object.entries(parsed)) {
      if (value && typeof value === "string" && !hexRegex.test(value)) {
        jsonError.value = `Invalid hex color for "${key}": ${value}`;
        parsedColors.value = null;
        return;
      }
    }

    parsedColors.value = parsed as TerminalTheme;
  } catch (error) {
    console.error("JSON parse error:", error);
    jsonError.value = "Invalid JSON format";
    parsedColors.value = null;
  }
};

const copyTemplate = () => {
  const template = {
    background: "#1a1a1a",
    foreground: "#ffffff",
    cursor: "#ffffff",
    black: "#000000",
    red: "#ff0000",
    green: "#00ff00",
    yellow: "#ffff00",
    blue: "#0000ff",
    magenta: "#ff00ff",
    cyan: "#00ffff",
    white: "#ffffff",
    brightBlack: "#555555",
    brightRed: "#ff5555",
    brightGreen: "#55ff55",
    brightYellow: "#ffff55",
    brightBlue: "#5555ff",
    brightMagenta: "#ff55ff",
    brightCyan: "#55ffff",
    brightWhite: "#ffffff",
  };
  jsonInput.value = JSON.stringify(template, null, 2);
  handleJsonInput();
  message.success("Template copied to editor");
};

// Load theme data when editing
watch(
  () => themeId.value,
  (id) => {
    if (id) {
      const theme = settingsStore.customThemes.find((t) => t.id === id);
      if (theme) {
        themeName.value = theme.name;
        jsonInput.value = JSON.stringify(theme.colors, null, 2);
        handleJsonInput();
      }
    } else {
      // Reset for new theme
      themeName.value = "";
      jsonInput.value = JSON.stringify(
        {
          background: "#1a1a1a",
          foreground: "#ffffff",
        },
        null,
        2,
      );
      handleJsonInput();
    }
  },
  { immediate: true },
);

const handleSave = async () => {
  if (!isValid.value || !parsedColors.value) return;

  try {
    if (isEditing.value && themeId.value) {
      await settingsStore.updateCustomTheme(
        themeId.value,
        themeName.value,
        parsedColors.value,
      );
      message.success("Custom theme updated successfully");
    } else {
      await settingsStore.createCustomTheme(
        themeName.value,
        parsedColors.value,
      );
      message.success("Custom theme created successfully");
    }

    closeOverlay("custom-theme-modal");
  } catch (error) {
    console.error("Failed to save custom theme:", error);
    message.error("Failed to save custom theme");
  }
};
</script>
