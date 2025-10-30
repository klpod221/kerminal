<template>
  <Modal
    id="font-settings-modal"
    title="Terminal Font Settings"
    :icon="Type"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    size="md"
    :show-close-button="true"
  >
    <div class="space-y-6">
      <!-- Current Font Preview -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <label
            class="text-sm font-medium text-gray-300 flex items-center gap-2"
          >
            <component :is="Eye" class="w-4 h-4 text-blue-400" />
            Currently Using
          </label>
          <div class="flex items-center gap-2">
            <span
              class="text-xs font-medium text-gray-400"
              :style="{
                fontFamily: `'${settingsStore.fontFamily}', monospace`,
              }"
            >
              {{ settingsStore.fontFamily }}
            </span>
            <span class="text-xs text-blue-400"
              >{{ settingsStore.fontSize }}px</span
            >
          </div>
        </div>
        <div
          class="w-full p-4 bg-[#0a0a0a] rounded-lg border border-gray-700 shadow-inner overflow-x-auto"
          :style="{
            fontFamily: `'${settingsStore.fontFamily}', monospace`,
            fontSize: `${settingsStore.fontSize}px`,
          }"
        >
          <div class="text-gray-300 space-y-1">
            <!-- Command line examples -->
            <div class="flex items-center gap-2">
              <span class="text-green-400">❯</span>
              <span class="text-blue-400">echo</span>
              <span class="text-yellow-400">"Hello, World!"</span>
            </div>
            <div class="text-gray-400 pl-4">Hello, World!</div>

            <div class="flex items-center gap-2 mt-3">
              <span class="text-green-400">❯</span>
              <span class="text-blue-400">ls</span>
              <span class="text-purple-400">-lah</span>
            </div>
            <div class="pl-4 space-y-0.5">
              <div class="flex gap-3 text-xs opacity-75">
                <span class="text-cyan-400">drwxr-xr-x</span>
                <span>10</span>
                <span>user</span>
                <span class="text-yellow-400">4.0K</span>
                <span>src/</span>
              </div>
              <div class="flex gap-3 text-xs opacity-75">
                <span class="text-cyan-400">-rw-r--r--</span>
                <span> 1</span>
                <span>user</span>
                <span class="text-yellow-400">2.1K</span>
                <span>main.ts</span>
              </div>
            </div>

            <!-- Special characters -->
            <div class="mt-3 text-xs opacity-60 border-t border-gray-800 pt-2">
              <span>Special chars: </span>
              <span class="text-purple-400">│ ├ └ ┐ ┘ ┌ ─ ║ ═</span>
              <span class="text-cyan-400 ml-2">→ ← ↑ ↓</span>
              <span class="text-green-400 ml-2"> ✓ ✗ ⚠</span>
            </div>
          </div>
        </div>
      </div>

      <div class="border-t border-gray-700" />

      <!-- Font Size -->
      <div class="space-y-3">
        <!-- Size Presets -->
        <div class="flex gap-2">
          <button
            v-for="preset in sizePresets"
            :key="preset.size"
            class="flex-1 px-3 py-2 rounded-lg border transition-all duration-200"
            :class="
              fontSizeValue === preset.size
                ? 'border-blue-500 bg-blue-500/20 text-blue-400'
                : 'border-gray-600 bg-gray-800 text-gray-400 hover:border-gray-500 hover:bg-gray-700'
            "
            @click="setPresetSize(preset.size)"
          >
            <div class="text-xs font-medium">{{ preset.label }}</div>
            <div class="text-[10px] opacity-70">{{ preset.size }}px</div>
          </button>
        </div>

        <!-- Slider -->
        <Slider
          v-model="fontSizeValue"
          label="Font Size"
          :icon="Maximize2"
          :min="8"
          :max="24"
          :step="1"
          unit="px"
          :show-value="true"
          :show-input="true"
          :show-marks="true"
          :marks="[16]"
          @change="updateFontSize"
        />
      </div>

      <div class="border-t border-gray-700" />

      <!-- Font Family -->
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <label
            class="text-sm font-medium text-gray-300 flex items-center gap-2"
          >
            <component :is="Type" class="w-4 h-4 text-blue-400" />
            Font Family
          </label>
          <span class="text-xs text-gray-500">
            {{ filteredFonts.length }}
            {{ filteredFonts.length === 1 ? "font" : "fonts" }}
          </span>
        </div>

        <!-- Search Box -->
        <div class="relative">
          <Input
            id="font-search-input"
            v-model="searchQuery"
            placeholder="Search fonts..."
            :icon="Search"
          />
        </div>

        <!-- Font List -->
        <div class="max-h-64 overflow-y-auto space-y-2 custom-scrollbar pr-1">
          <Card
            v-for="font in filteredFonts"
            :key="font"
            no-padding
            :custom-class="
              settingsStore.fontFamily === font
                ? 'p-3 cursor-pointer !border-blue-500 !bg-blue-600/10 shadow-lg shadow-blue-500/20 font-card-selected font-card-hover'
                : 'p-3 cursor-pointer font-card font-card-hover'
            "
            @click="selectFont(font)"
          >
            <div class="flex items-center justify-between gap-3">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-1">
                  <span class="text-sm font-medium text-white truncate">
                    {{ font }}
                  </span>
                  <span
                    v-if="
                      font.toLowerCase().includes('mono') ||
                      font.toLowerCase().includes('code')
                    "
                    class="shrink-0 px-1.5 py-0.5 text-[9px] bg-blue-500/20 text-blue-400 rounded font-medium uppercase tracking-wide"
                  >
                    Mono
                  </span>
                </div>
                <div
                  class="text-xs text-gray-400 truncate"
                  :style="{ fontFamily: `'${font}', monospace` }"
                >
                  The quick brown fox jumps
                </div>
              </div>
              <component
                v-if="settingsStore.fontFamily === font"
                :is="Check"
                class="w-5 h-5 text-blue-400 shrink-0 animate-scale-in"
              />
            </div>
          </Card>

          <div v-if="filteredFonts.length === 0" class="text-center py-12">
            <component
              :is="Search"
              class="w-12 h-12 text-gray-600 mx-auto mb-3"
            />
            <p class="text-sm text-gray-400">No fonts found</p>
            <p class="text-xs text-gray-500 mt-1">
              Try a different search term
            </p>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-between items-center w-full">
        <div class="text-sm text-gray-400">
          {{ availableFonts.length }} fonts available
        </div>
        <Button variant="outline" @click="resetToDefaults">
          Reset to Defaults
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { Type, Check, Search, Maximize2, Eye } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import Card from "../ui/Card.vue";
import Input from "../ui/Input.vue";
import Slider from "../ui/Slider.vue";
import { useSettingsStore } from "../../stores/settings";
import { message } from "../../utils/message";

const settingsStore = useSettingsStore();

const availableFonts = ref<string[]>([]);
const searchQuery = ref("");
const fontSizeValue = ref(settingsStore.fontSize);

const sizePresets = [
  { label: "Small", size: 11 },
  { label: "Medium", size: 13 },
  { label: "Large", size: 16 },
];

// Load system fonts
onMounted(async () => {
  try {
    availableFonts.value = await invoke<string[]>("get_system_fonts");
  } catch (error) {
    console.error("Failed to load system fonts:", error);
    message.error("Failed to load system fonts");
  }
});

// Watch for font size changes from store
watch(
  () => settingsStore.fontSize,
  (newSize) => {
    fontSizeValue.value = newSize;
  },
);

// Filter fonts based on search query
const filteredFonts = computed(() => {
  if (!searchQuery.value) {
    return availableFonts.value;
  }

  const query = searchQuery.value.toLowerCase();
  return availableFonts.value.filter((font) =>
    font.toLowerCase().includes(query),
  );
});

// Select font
const selectFont = async (font: string) => {
  try {
    await settingsStore.setFontFamily(font);
    message.success(`Font changed to ${font}`);
  } catch (error) {
    console.error("Failed to set font:", error);
    message.error("Failed to change font");
  }
};

// Update font size
const updateFontSize = async () => {
  try {
    await settingsStore.setFontSize(fontSizeValue.value);
  } catch (error) {
    console.error("Failed to set font size:", error);
    message.error("Failed to change font size");
  }
};

// Set preset size
const setPresetSize = async (size: number) => {
  fontSizeValue.value = size;
  await updateFontSize();
};

// Reset to defaults
const resetToDefaults = async () => {
  try {
    await settingsStore.setFontFamily("FiraCode Nerd Font");
    await settingsStore.setFontSize(13);
    fontSizeValue.value = 13;
    message.success("Font settings reset to defaults");
  } catch (error) {
    console.error("Failed to reset font settings:", error);
    message.error("Failed to reset font settings");
  }
};
</script>

<style scoped>
/* Custom scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: #1f2937;
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #6b7280;
}

/* Font card animations */
.font-card {
  transition: all 0.2s ease;
}

.font-card-hover:hover {
  background-color: rgba(31, 41, 55, 0.5) !important;
}

.font-card-selected {
  animation: pulse-border 2s ease-in-out infinite;
}

@keyframes pulse-border {
  0%,
  100% {
    border-color: rgba(59, 130, 246, 0.5);
  }
  50% {
    border-color: rgba(59, 130, 246, 0.8);
  }
}

@keyframes scale-in {
  from {
    transform: scale(0);
    opacity: 0;
  }
  to {
    transform: scale(1);
    opacity: 1;
  }
}

.animate-scale-in {
  animation: scale-in 0.2s ease-out;
}
</style>
