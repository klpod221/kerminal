<template>
  <Modal
    id="keyboard-shortcuts-modal"
    title="Keyboard Shortcuts"
    :icon="Keyboard"
    icon-background="bg-purple-500/20"
    icon-color="text-purple-400"
    size="2xl"
    :show-close-button="true"
  >
    <div class="space-y-4">
      <!-- Search -->
      <Input
        id="search-shortcuts"
        v-model="searchQuery"
        placeholder="Search shortcuts..."
        :icon="Search"
      />

      <!-- Shortcuts by Category -->
      <div
        v-for="(shortcuts, category) in filteredShortcutsByCategory"
        :key="category"
        class="space-y-3"
      >
        <h3 class="text-sm font-semibold text-gray-300 uppercase tracking-wide">
          {{ category }}
        </h3>

        <div class="space-y-2">
          <Card
            v-for="shortcut in shortcuts"
            :key="shortcut.id"
            :hover="true"
            no-padding
            :custom-class="
              hasConflict(shortcut.id)
                ? 'p-3 !border-red-500/50 !bg-red-900/10'
                : 'p-3'
            "
          >
            <div class="flex items-center justify-between">
              <div class="flex-1">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-white">
                    {{ shortcut.label }}
                  </span>
                  <Badge
                    v-if="hasConflict(shortcut.id)"
                    variant="danger"
                    size="xs"
                  >
                    Conflict
                  </Badge>
                </div>
              </div>

              <div class="flex items-center gap-3">
                <!-- Current Shortcut Display -->
                <div
                  v-if="!editingShortcut || editingShortcut.id !== shortcut.id"
                  class="flex items-center gap-2"
                >
                  <KeyBadge :keys="formatShortcutKeys(shortcut)" size="sm" />
                  <Button
                    v-if="shortcut.customizable"
                    variant="ghost"
                    size="sm"
                    :icon="Edit3"
                    class="p-1.5! text-gray-400 hover:text-blue-400 hover:bg-blue-600/20"
                    @click="startEditing(shortcut)"
                  >
                    Edit
                  </Button>
                </div>

                <!-- Editing Mode -->
                <div v-else class="flex items-center gap-2">
                  <div
                    class="px-3 py-1.5 bg-gray-800 border border-blue-500 rounded text-sm text-gray-300 min-w-[120px] text-center cursor-pointer"
                    :class="{
                      'border-blue-400': !isCapturing,
                      'border-yellow-400 animate-pulse': isCapturing,
                    }"
                    @click="startCapturing"
                  >
                    <span v-if="!isCapturing" class="text-gray-400">
                      Click to capture
                    </span>
                    <span v-else class="text-yellow-400"> Press keys... </span>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    :icon="X"
                    class="p-1.5! text-gray-400 hover:text-red-400 hover:bg-red-600/20"
                    @click="cancelEditing"
                  >
                    Cancel
                  </Button>
                </div>

                <!-- Reset Button -->
                <Button
                  v-if="
                    shortcut.customizable &&
                    (shortcut.customKey || shortcut.customModifiers) &&
                    (!editingShortcut || editingShortcut.id !== shortcut.id)
                  "
                  variant="ghost"
                  size="sm"
                  :icon="RotateCcw"
                  title="Reset to default"
                  class="p-1.5! text-gray-400 hover:text-blue-400 hover:bg-blue-600/20"
                  @click="resetShortcut(shortcut.id)"
                />
              </div>
            </div>
          </Card>
        </div>
      </div>

      <!-- Empty State -->
      <EmptyState
        v-if="Object.keys(filteredShortcutsByCategory).length === 0"
        :icon="Search"
        title="No shortcuts found"
        description="No shortcuts match your search query."
      />

      <!-- Conflict Warning -->
      <div
        v-if="shortcutsStore.conflicts.length > 0"
        class="p-3 bg-red-900/20 border border-red-500/50 rounded-lg"
      >
        <div class="flex items-start gap-2">
          <AlertTriangle :size="16" class="text-red-400 mt-0.5 shrink-0" />
          <div class="flex-1">
            <p class="text-sm font-medium text-red-400 mb-1">
              Shortcut Conflicts Detected
            </p>
            <p class="text-xs text-red-300/80">
              Some shortcuts have conflicts. Please remap them to avoid
              conflicts.
            </p>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between w-full">
        <Button
          variant="secondary"
          :icon="RotateCcw"
          @click="resetAllShortcuts"
        >
          Reset All
        </Button>
        <Button
          variant="primary"
          @click="closeOverlay('keyboard-shortcuts-modal')"
        >
          Close
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from "vue";
import {
  Keyboard,
  Search,
  Edit3,
  X,
  RotateCcw,
  AlertTriangle,
} from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import Badge from "../ui/Badge.vue";
import Card from "../ui/Card.vue";
import EmptyState from "../ui/EmptyState.vue";
import KeyBadge from "../ui/KeyBadge.vue";
import { useKeyboardShortcutsStore } from "../../stores/keyboardShortcuts";
import { useOverlay } from "../../composables/useOverlay";
import { useDebounce } from "../../composables/useDebounce";
import type {
  KeyboardShortcut,
  ShortcutModifiers,
} from "../../types/shortcuts";
import { ShortcutAction } from "../../types/shortcuts";
import { message } from "../../utils/message";

const shortcutsStore = useKeyboardShortcutsStore();
const { closeOverlay } = useOverlay();

const searchQuery = ref("");
const debouncedSearchQuery = useDebounce(searchQuery, { delay: 300 });
const editingShortcut = ref<KeyboardShortcut | null>(null);
const isCapturing = ref(false);
const capturedKey = ref<string>("");
const capturedModifiers = ref<ShortcutModifiers>({});

// Filter shortcuts by search query
const filteredShortcutsByCategory = computed(() => {
  const query = debouncedSearchQuery.value.toLowerCase();
  const grouped: Record<string, KeyboardShortcut[]> = {};

  for (const [category, shortcuts] of Object.entries(
    shortcutsStore.shortcutsByCategory,
  )) {
    const filtered = shortcuts.filter(
      (s) =>
        s.label.toLowerCase().includes(query) ||
        category.toLowerCase().includes(query),
    );

    if (filtered.length > 0) {
      grouped[category] = filtered;
    }
  }

  return grouped;
});

const hasConflict = (action: ShortcutAction): boolean => {
  return shortcutsStore.getConflictsFor(action).length > 0;
};

const formatShortcutKeys = (shortcut: KeyboardShortcut): string[] => {
  const activeShortcut = shortcutsStore.getActiveShortcut(shortcut.id);
  if (!activeShortcut) return [];

  const keys: string[] = [];

  // For cross-platform shortcuts, we only show ONE modifier: Ctrl OR Cmd/Win
  // Priority: Ctrl on Windows/Linux, Cmd on Mac
  const hasCtrl = !!activeShortcut.modifiers.ctrlKey;
  const hasMeta = !!activeShortcut.modifiers.metaKey;

  // Show only one: Ctrl (Windows/Linux) or Cmd (Mac), never both
  if (hasCtrl || hasMeta) {
    if (navigator.platform.includes("Mac")) {
      // On Mac: prefer Cmd (metaKey), fallback to Ctrl
      keys.push(hasMeta ? "Cmd" : "Ctrl");
    } else {
      // On Windows/Linux: prefer Ctrl, fallback to Win (metaKey)
      keys.push(hasCtrl ? "Ctrl" : "Win");
    }
  }

  if (activeShortcut.modifiers.altKey) {
    keys.push("Alt");
  }
  if (activeShortcut.modifiers.shiftKey) {
    keys.push("Shift");
  }
  keys.push(activeShortcut.key);
  return keys;
};

const startEditing = (shortcut: KeyboardShortcut) => {
  editingShortcut.value = shortcut;
  isCapturing.value = false;
  capturedKey.value = "";
  capturedModifiers.value = {};
};

const cancelEditing = () => {
  editingShortcut.value = null;
  isCapturing.value = false;
  capturedKey.value = "";
  capturedModifiers.value = {};
};

const startCapturing = () => {
  if (!editingShortcut.value) return;
  isCapturing.value = true;
};

const handleKeyCapture = (event: KeyboardEvent) => {
  if (!isCapturing.value || !editingShortcut.value) return;

  // Ignore Escape key to cancel
  if (event.key === "Escape") {
    cancelEditing();
    return;
  }

  // Ignore modifier-only presses (wait for actual key)
  if (
    ["Control", "Alt", "Shift", "Meta", "Ctrl"].includes(event.key) &&
    !capturedKey.value
  ) {
    return;
  }

  event.preventDefault();
  event.stopPropagation();

  // Normalize key name
  let key = event.key.toLowerCase();
  if (key === " ") {
    key = "space";
  } else if (key.startsWith("arrow")) {
    key = key.replace("arrow", "");
  }

  capturedKey.value = key;

  // For cross-platform: only save one modifier (Ctrl OR Meta, not both)
  // On Mac, prefer metaKey (Cmd). On Windows/Linux, prefer ctrlKey (Ctrl)
  const isMac = navigator.platform.includes("Mac");
  const hasCtrl = event.ctrlKey;
  const hasMeta = event.metaKey;

  // Normalize: only save one modifier based on platform preference
  capturedModifiers.value = {
    ctrlKey: isMac ? false : hasCtrl, // Only save Ctrl on Windows/Linux
    altKey: event.altKey,
    shiftKey: event.shiftKey,
    metaKey: isMac ? hasMeta : false, // Only save Meta (Cmd) on Mac
  };

  // Auto-save after capturing
  setTimeout(async () => {
    if (editingShortcut.value && capturedKey.value) {
      await shortcutsStore.updateShortcut(
        editingShortcut.value.id,
        capturedKey.value,
        capturedModifiers.value,
      );

      // Check for conflicts
      const conflicts = shortcutsStore.getConflictsFor(
        editingShortcut.value.id,
      );
      if (conflicts.length > 0) {
        const conflictNames = conflicts.map((c) => {
          const otherAction =
            c.action === editingShortcut.value!.id
              ? c.conflictingAction
              : c.action;
          const shortcut = shortcutsStore.shortcuts.get(otherAction);
          return shortcut?.label || otherAction;
        });
        message.warning(`Shortcut conflicts with: ${conflictNames.join(", ")}`);
      } else {
        message.success("Shortcut updated");
      }

      cancelEditing();
    }
  }, 100);
};

const resetShortcut = async (action: ShortcutAction) => {
  await shortcutsStore.resetShortcut(action);
  message.success("Shortcut reset to default");
};

const resetAllShortcuts = async () => {
  await shortcutsStore.resetAllShortcuts();
};

onMounted(async () => {
  await shortcutsStore.loadShortcuts();

  // Add key capture listener
  if (isCapturing.value) {
    document.addEventListener("keydown", handleKeyCapture);
  }
});

onBeforeUnmount(() => {
  document.removeEventListener("keydown", handleKeyCapture);
});

// Watch for capturing state
watch(isCapturing, (capturing) => {
  if (capturing) {
    document.addEventListener("keydown", handleKeyCapture, true);
  } else {
    document.removeEventListener("keydown", handleKeyCapture, true);
  }
});
</script>

<style scoped>
/* Additional styles if needed */
</style>
