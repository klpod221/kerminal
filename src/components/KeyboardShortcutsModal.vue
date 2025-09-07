<template>
  <Modal
    :visible="isVisible"
    title="Keyboard Shortcuts"
    :icon="Keyboard"
    icon-background="bg-purple-500/20"
    icon-color="text-purple-400"
    size="xl"
    @close="closeModal"
  >
    <!-- Category Tabs -->
    <div class="space-y-4">
      <div
        class="flex gap-1 bg-gray-800/50 p-1.5 rounded-lg backdrop-blur-sm border border-gray-700/50"
      >
        <button
          v-for="category in categories"
          :key="category.key"
          :class="[
            'px-4 py-2.5 rounded-md text-sm font-medium transition-all duration-200 ease-in-out flex-1 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900',
            activeCategory === category.key
              ? 'bg-blue-600 text-white shadow-lg transform scale-[0.98]'
              : 'text-gray-300 hover:text-white hover:bg-gray-700/70 hover:scale-[0.99] active:scale-[0.97]'
          ]"
          @click="activeCategory = category.key"
        >
          {{ category.label }}
        </button>
      </div>

      <!-- Shortcuts List -->
      <div class="space-y-3 h-full overflow-y-auto max-h-[500px]">
        <div
          v-for="shortcut in filteredShortcuts"
          :key="shortcut.id"
          class="group flex items-center justify-between p-4 bg-gray-800/40 rounded-lg border border-gray-700/50 hover:border-gray-600/70 hover:bg-gray-800/60 transition-all duration-200 backdrop-blur-sm"
        >
          <div class="flex-1">
            <div class="flex items-center space-x-3">
              <h3 class="text-white font-medium group-hover:text-gray-100">{{ shortcut.name }}</h3>
              <span
                v-if="!shortcut.enabled"
                class="px-2.5 py-1 text-xs bg-red-900/30 text-red-400 rounded-md border border-red-700/50"
              >
                Disabled
              </span>
            </div>
            <p class="text-gray-400 text-sm mt-1 group-hover:text-gray-300">
              {{ shortcut.description }}
            </p>
          </div>
          <div class="flex items-center space-x-3">
            <!-- Keyboard shortcut display -->
            <div class="flex items-center gap-1">
              <template v-for="(key, index) in shortcut.keys" :key="index">
                <KeyBadge :keys="[key]" />
                <span
                  v-if="index < shortcut.keys.length - 1"
                  class="text-gray-500 text-sm font-medium"
                >
                  +
                </span>
              </template>
            </div>
            <!-- Enable/Disable toggle -->
            <button
              :class="[
                'relative inline-flex h-6 w-11 items-center rounded-full transition-all duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900',
                shortcut.enabled ? 'bg-blue-600 hover:bg-blue-500' : 'bg-gray-600 hover:bg-gray-500'
              ]"
              @click="toggleShortcut(shortcut.id, shortcut.enabled)"
            >
              <span
                :class="[
                  'inline-block h-4 w-4 transform rounded-full bg-white shadow-lg transition-all duration-200 ease-in-out',
                  shortcut.enabled ? 'translate-x-6' : 'translate-x-1'
                ]"
              ></span>
            </button>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="filteredShortcuts.length === 0" class="text-center py-12 text-gray-400">
        <Keyboard class="h-12 w-12 mx-auto mb-4 opacity-50" />
        <p>No shortcuts found in this category.</p>
      </div>
    </div>

    <!-- Footer -->
    <template #footer>
      <div class="flex items-center justify-end w-full">
        <PopConfirm
          title="Reset Shortcuts"
          content="Are you sure you want to reset all shortcuts to defaults? This action cannot be undone."
          ok-text="Reset"
          cancel-text="Cancel"
          ok-type="danger"
          @confirm="handleResetConfirm"
        >
          <Button
            size="sm"
            class="text-gray-400 hover:text-white hover:bg-gray-700/50 transition-all duration-200"
            :icon="RotateCcw"
          >
            Reset to Defaults
          </Button>
        </PopConfirm>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Keyboard, RotateCcw } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import Button from './ui/Button.vue'
import KeyBadge from './ui/KeyBadge.vue'
import PopConfirm from './ui/PopConfirm.vue'
import { useKeyboardShortcuts } from '../services/keyboard-shortcut-service'
import { KEYBOARD_SHORTCUT_CATEGORY_LABELS } from '../types/keyboard'
import type { KeyboardShortcutCategory } from '../types/keyboard'
import type { KeyboardShortcutsModalProps } from '../types/modals'

const { isVisible } = defineProps<KeyboardShortcutsModalProps>()

const emit = defineEmits<{
  'update:isVisible': [value: boolean]
}>()

const {
  shortcutsByCategory,
  enableShortcut,
  disableShortcut,
  resetToDefaults: resetShortcuts
} = useKeyboardShortcuts()

const activeCategory = ref<KeyboardShortcutCategory>('general')

// Available categories
const categories = computed(() => {
  return Object.keys(shortcutsByCategory.value).map((key) => ({
    key: key as KeyboardShortcutCategory,
    label: KEYBOARD_SHORTCUT_CATEGORY_LABELS[key as KeyboardShortcutCategory]
  }))
})

// Filtered shortcuts based on active category
const filteredShortcuts = computed(() => {
  return shortcutsByCategory.value[activeCategory.value] || []
})

// Close modal
const closeModal = (): void => {
  emit('update:isVisible', false)
}

// Toggle shortcut enabled state
const toggleShortcut = (id: string, currentEnabled: boolean): void => {
  if (currentEnabled) {
    disableShortcut(id)
  } else {
    enableShortcut(id)
  }
}

// Handle reset confirmation
const handleResetConfirm = (): void => {
  resetShortcuts()
}

// Set default active category when modal opens
watch(
  () => isVisible,
  (newValue) => {
    if (newValue && categories.value.length > 0) {
      activeCategory.value = categories.value[0].key
    }
  }
)
</script>
