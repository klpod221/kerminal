<template>
  <Modal
    id="theme-selector-modal"
    title="Terminal Theme"
    :icon="Palette"
    icon-background="bg-purple-500/20"
    icon-color="text-purple-400"
    size="lg"
    :show-close-button="true"
  >
    <!-- Action Buttons -->
    <div class="mb-4 flex gap-2">
      <Button
        variant="primary"
        :icon="Plus"
        size="sm"
        class="flex-1"
        @click="openCreateModal"
      >
        Create Custom Theme
      </Button>
      <Button
        variant="outline"
        :icon="Type"
        size="sm"
        class="flex-1"
        @click="openFontSettings"
      >
        Font Settings
      </Button>
    </div>

    <!-- Custom Themes Section -->
    <div v-if="settingsStore.customThemes.length > 0" class="space-y-2 mb-6">
      <div class="flex items-center gap-2 px-2 py-1.5">
        <span class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
          Custom Themes
        </span>
        <div class="flex-1 h-px bg-purple-500/30"></div>
        <span class="text-xs text-gray-500">
          {{ settingsStore.customThemes.length }}
        </span>
      </div>
      <Card
        v-for="theme in customThemesList"
        :key="theme"
        :hover="true"
        no-padding
        :custom-class="
          settingsStore.terminalTheme === theme
            ? 'p-3 cursor-pointer !border-purple-500'
            : 'p-3 cursor-pointer'
        "
        @click="selectTheme(theme)"
      >
        <div class="flex items-center gap-3">
          <!-- Theme Preview -->
          <div
            class="shrink-0 w-16 h-10 rounded border border-gray-600 overflow-hidden shadow-md"
            :style="{
              backgroundColor: getThemeColors(theme).background,
            }"
          >
            <div class="flex items-center justify-center h-full gap-0.5">
              <div
                class="w-1.5 h-1.5 rounded-full"
                :style="{ backgroundColor: getThemeColors(theme).red }"
              ></div>
              <div
                class="w-1.5 h-1.5 rounded-full"
                :style="{ backgroundColor: getThemeColors(theme).green }"
              ></div>
              <div
                class="w-1.5 h-1.5 rounded-full"
                :style="{ backgroundColor: getThemeColors(theme).blue }"
              ></div>
            </div>
          </div>

          <!-- Theme Name -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="font-medium text-white text-sm truncate">{{ theme }}</span>
              <span class="shrink-0 px-1.5 py-0.5 text-[10px] bg-purple-500/20 text-purple-300 rounded font-medium">
                CUSTOM
              </span>
            </div>
            <div class="text-xs text-gray-500 mt-0.5">
              {{ getThemeDescription(theme) }}
            </div>
          </div>

          <!-- Actions -->
          <div class="shrink-0 flex items-center gap-1">
            <!-- Edit button -->
            <Button
              variant="ghost"
              size="sm"
              :icon="Edit2"
              title="Edit theme"
              class="p-1.5! text-gray-400 hover:text-blue-400 hover:bg-blue-600/20"
              @click.stop="openEditModal(theme)"
            />

            <!-- Delete button -->
            <Button
              variant="ghost"
              size="sm"
              :icon="Trash2"
              title="Delete theme"
              class="p-1.5! text-gray-400 hover:text-red-400 hover:bg-red-600/20"
              @click.stop="handleDeleteTheme(theme)"
            />

            <!-- Selected Indicator -->
            <div
              v-if="settingsStore.terminalTheme === theme"
              class="shrink-0 ml-1"
            >
              <component :is="Check" class="w-5 h-5 text-purple-400" />
            </div>
          </div>
        </div>
      </Card>
    </div>

    <!-- Built-in Themes Section -->
    <div class="space-y-2">
      <div class="flex items-center gap-2 px-2 py-1.5">
        <span class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
          Built-in Themes
        </span>
        <div class="flex-1 h-px bg-gray-700"></div>
        <span class="text-xs text-gray-500">
          {{ settingsStore.builtInThemes.length }}
        </span>
      </div>
      <Card
        v-for="theme in builtInThemesList"
        :key="theme"
        :hover="true"
        no-padding
        :custom-class="
          settingsStore.terminalTheme === theme
            ? 'p-3 cursor-pointer !border-purple-500 !bg-purple-600/10 shadow-lg shadow-purple-500/20'
            : 'p-3 cursor-pointer'
        "
        @click="selectTheme(theme)"
      >
        <div class="flex items-center gap-3">
          <!-- Theme Preview -->
          <div
            class="shrink-0 w-16 h-10 rounded border border-gray-600 overflow-hidden shadow-md"
            :style="{
              backgroundColor: getThemeColors(theme).background,
            }"
          >
            <div class="flex items-center justify-center h-full gap-0.5">
              <div
                class="w-1.5 h-1.5 rounded-full"
                :style="{ backgroundColor: getThemeColors(theme).red }"
              ></div>
              <div
                class="w-1.5 h-1.5 rounded-full"
                :style="{ backgroundColor: getThemeColors(theme).green }"
              ></div>
              <div
                class="w-1.5 h-1.5 rounded-full"
                :style="{ backgroundColor: getThemeColors(theme).blue }"
              ></div>
            </div>
          </div>

          <!-- Theme Name -->
          <div class="flex-1 min-w-0">
            <div class="font-medium text-white text-sm truncate">{{ theme }}</div>
            <div class="text-xs text-gray-500 mt-0.5">
              {{ getThemeDescription(theme) }}
            </div>
          </div>

          <!-- Selected Indicator -->
          <div
            v-if="settingsStore.terminalTheme === theme"
            class="shrink-0"
          >
            <component :is="Check" class="w-5 h-5 text-purple-400" />
          </div>
        </div>
      </Card>
    </div>

    <template #footer>
      <div class="text-sm text-gray-400">
        {{ settingsStore.availableThemes.length }} themes available
        <span v-if="settingsStore.customThemes.length > 0">
          ({{ settingsStore.customThemes.length }} custom)
        </span>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { Palette, Check, Plus, Edit2, Trash2, Type } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import Card from "../ui/Card.vue";
import { useSettingsStore } from "../../stores/settings";
import { getTerminalTheme } from "../../utils/terminalTheme";
import { useOverlay } from "../../composables/useOverlay";
import { message, showConfirm } from "../../utils/message";

const settingsStore = useSettingsStore();
const { openOverlay } = useOverlay();

// Separate custom and built-in themes
const customThemesList = computed(() => {
  return settingsStore.customThemes.map(t => t.name);
});

const builtInThemesList = computed(() => {
  return settingsStore.builtInThemes;
});

const selectTheme = async (theme: string) => {
  await settingsStore.setTerminalTheme(theme);
};

const openCreateModal = () => {
  openOverlay("custom-theme-modal");
};

const openEditModal = (themeName: string) => {
  const customTheme = settingsStore.getCustomTheme(themeName);
  if (customTheme) {
    openOverlay("custom-theme-modal", { themeId: customTheme.id });
  }
};

const openFontSettings = () => {
  openOverlay("font-settings-modal");
};

const handleDeleteTheme = async (themeName: string) => {
  const customTheme = settingsStore.getCustomTheme(themeName);
  if (!customTheme) return;

  // Confirm before delete
  const confirmed = await showConfirm(
    "Delete Theme",
    `Are you sure you want to delete "${themeName}" theme?`,
  );
  if (!confirmed) return;

  try {
    await settingsStore.deleteCustomTheme(customTheme.id);
    message.success("Theme deleted successfully");
  } catch (error) {
    console.error("Failed to delete theme:", error);
    message.error("Failed to delete theme");
  }
};

const getThemeColors = (themeName: string) => {
  const customTheme = settingsStore.getCustomTheme(themeName);
  const theme = customTheme
    ? customTheme.colors
    : getTerminalTheme(themeName as any);

  return {
    background: theme.background,
    foreground: theme.foreground,
    red: theme.red || "#ff0000",
    green: theme.green || "#00ff00",
    blue: theme.blue || "#0000ff",
  };
};

const getThemeDescription = (themeName: string) => {
  const customTheme = settingsStore.getCustomTheme(themeName);
  const theme = customTheme
    ? customTheme.colors
    : getTerminalTheme(themeName as any);

  const isDark =
    parseInt(theme.background.substring(1, 3), 16) < 128;
  return isDark ? "Dark theme" : "Light theme";
};
</script>


