<template>
  <Teleport to="body">
    <div
      v-if="visible"
      ref="contextMenuRef"
      class="fixed z-50 bg-gray-900 border border-gray-700 rounded-lg shadow-2xl py-1 min-w-[180px]"
      :style="{ left: position.x + 'px', top: position.y + 'px' }"
      @click.stop
    >
      <template v-for="(item, index) in items" :key="index">
        <div v-if="item.type === 'divider'" class="h-px bg-gray-700 my-1"></div>
        <div
          v-else
          class="flex items-center px-3 py-2 text-sm text-gray-300 hover:bg-gray-800 hover:text-white cursor-pointer transition-colors duration-150"
          :class="{
            'text-red-400 hover:text-red-300': item.danger,
            'opacity-50 cursor-not-allowed': item.disabled,
          }"
          @click="handleItemClick(item)"
        >
          <component
            :is="item.icon"
            v-if="item.icon"
            :size="16"
            class="mr-2 flex-shrink-0"
          />
          <span class="flex-1">{{ item.label || "" }}</span>
          <span v-if="item.shortcut" class="text-xs text-gray-500 ml-2">
            {{ item.shortcut }}
          </span>
        </div>
      </template>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted } from "vue";
import type { Component } from "vue";

export interface ContextMenuItem {
  id: string;
  label?: string;
  icon?: Component;
  shortcut?: string;
  danger?: boolean;
  disabled?: boolean;
  type?: "item" | "divider";
  action?: string;
}

interface ContextMenuProps {
  items: ContextMenuItem[];
}

interface ContextMenuEmits {
  itemClick: [item: ContextMenuItem];
}

defineProps<ContextMenuProps>();
const emit = defineEmits<ContextMenuEmits>();

const contextMenuRef = ref<HTMLElement | null>(null);
const visible = ref(false);
const position = ref({ x: 0, y: 0 });

/**
 * Shows the context menu at the specified position
 */
const show = async (x: number, y: number): Promise<void> => {
  position.value = { x, y };
  visible.value = true;

  await nextTick();

  if (contextMenuRef.value) {
    const rect = contextMenuRef.value.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    if (rect.right > viewportWidth) {
      position.value.x = Math.max(0, x - rect.width);
    }

    if (rect.bottom > viewportHeight) {
      position.value.y = Math.max(0, y - rect.height);
    }
  }
};

/**
 * Hides the context menu
 */
const hide = (): void => {
  visible.value = false;
};

/**
 * Handles clicking on a context menu item
 */
const handleItemClick = (item: ContextMenuItem): void => {
  if (item.disabled) return;

  hide();
  emit("itemClick", item);
};

/**
 * Handles clicks outside the context menu
 */
const handleClickOutside = (event: MouseEvent): void => {
  if (
    contextMenuRef.value &&
    !contextMenuRef.value.contains(event.target as Node)
  ) {
    hide();
  }
};

/**
 * Handles escape key press
 */
const handleKeyDown = (event: KeyboardEvent): void => {
  if (event.key === "Escape") {
    hide();
  }
};

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
  document.addEventListener("contextmenu", hide);
  document.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
  document.removeEventListener("contextmenu", hide);
  document.removeEventListener("keydown", handleKeyDown);
});

defineExpose({
  show,
  hide,
  visible,
});
</script>

<style scoped>
/* Animation for context menu */
.v-enter-active,
.v-leave-active {
  transition: all 0.15s ease-out;
}

.v-enter-from {
  opacity: 0;
  transform: scale(0.95) translateY(-4px);
}

.v-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(-4px);
}
</style>
