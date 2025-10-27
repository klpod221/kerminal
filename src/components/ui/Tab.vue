<template>
  <div
    class="no-drag flex items-center h-full border-r border-gray-800 cursor-pointer group transition-all duration-300 ease-out flex-1 relative overflow-hidden touch-manipulation sm:max-h-[30px]"
    :class="[
      {
        'active-tab bg-[#171717] border-b-2 border-b-blue-500': isActive,
        'hover:bg-gray-800': !isActive,
        'opacity-50': isDragging,
      },
      isMobile ? 'px-1.5 max-h-[36px]' : 'px-2 max-h-[30px]',
    ]"
    :style="{ minWidth: minWidth + 'px', maxWidth: maxWidth + 'px' }"
    draggable="true"
    @click="$emit('select')"
    @contextmenu="onContextMenu"
    @dragstart="onDragStart"
    @dragend="onDragEnd"
    @dragover="onDragOver"
    @drop="onDrop"
  >
    <div
      v-if="isConnecting && (minWidth >= 80 || isMobile)"
      class="transition-colors duration-200 flex-shrink-0"
      :class="isMobile ? 'mr-1.5' : 'mr-2'"
    >
      <div
        class="animate-spin rounded-full border border-blue-400 border-t-transparent"
        :class="isMobile ? 'h-3.5 w-3.5' : 'h-3 w-3'"
      ></div>
    </div>
    <Terminal
      v-else-if="minWidth >= 80 || isMobile"
      :size="isMobile ? 16 : 14"
      class="transition-colors duration-200 flex-shrink-0"
      :class="[
        isActive ? 'text-blue-400' : 'text-gray-400',
        isMobile ? 'mr-1.5' : 'mr-2',
      ]"
    />
    <div
      v-if="tab.color && (minWidth >= 60 || isMobile)"
      class="rounded-full flex-shrink-0"
      :class="isMobile ? 'w-2.5 h-2.5 mr-1.5' : 'w-2 h-2 mr-2'"
      :style="{ backgroundColor: tab.color }"
    ></div>
    <span
      class="truncate flex-1 transition-colors duration-200"
      :class="[
        isActive ? 'text-white' : 'text-gray-300',
        isMobile ? 'text-xs' : 'text-sm',
      ]"
    >
      {{ isConnecting ? "Connecting..." : tab.title }}
    </span>
    <X
      v-if="minWidth >= 100 || isTouch"
      :size="isMobile ? 16 : 14"
      class="text-gray-500 hover:text-red-400 transition-all duration-300 ease-out flex-shrink-0 transform hover:scale-110 cursor-pointer touch-manipulation"
      :class="[
        isTouch ? 'ml-1 opacity-100' : 'ml-2 opacity-0 group-hover:opacity-100',
      ]"
      @click.stop="$emit('close')"
    />

    <!-- Context Menu -->
    <ContextMenu
      ref="contextMenuRef"
      :items="contextMenuItems"
      @item-click="handleContextMenuAction"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import {
  Terminal,
  X,
  Copy,
  ExternalLink,
  Trash2,
  ArrowRight,
  Minus,
} from "lucide-vue-next";
import ContextMenu from "./ContextMenu.vue";
import type { ContextMenuItem } from "./ContextMenu.vue";
import type { Tab } from "../../types/panel";
import { safeJsonParse, safeJsonStringify } from "../../utils/helpers";
import { useWindowSize } from "../../composables/useWindowSize";

const { isMobile, isTouch } = useWindowSize();

interface TabProps {
  tab: Tab;
  isActive: boolean;
  isConnecting?: boolean;
  minWidth: number;
  maxWidth: number;
  panelId: string;
}

interface TabEmits {
  select: [];
  close: [];
  duplicate: [tab: Tab];
  moveToNewPanel: [tab: Tab];
  closeOthers: [tab: Tab];
  closeToRight: [tab: Tab];
  dragStart: [tab: Tab];
  drop: [draggedTab: Tab, targetTab: Tab];
}

const props = withDefaults(defineProps<TabProps>(), {
  isConnecting: false,
});

const emit = defineEmits<TabEmits>();

const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null);

const isDragging = ref(false);

/**
 * Context menu items configuration
 */
const contextMenuItems: ContextMenuItem[] = [
  {
    id: "duplicate",
    label: "Duplicate Tab",
    icon: Copy,
    action: "duplicate",
  },
  {
    id: "move-new-panel",
    label: "Move to New Panel",
    icon: ExternalLink,
    action: "moveToNewPanel",
  },
  {
    id: "divider-1",
    type: "divider",
  },
  {
    id: "close-others",
    label: "Close Other Tabs",
    icon: Minus,
    action: "closeOthers",
  },
  {
    id: "close-to-right",
    label: "Close Tabs to the Right",
    icon: ArrowRight,
    action: "closeToRight",
  },
  {
    id: "divider-2",
    type: "divider",
  },
  {
    id: "close",
    label: "Close Tab",
    icon: Trash2,
    danger: true,
    action: "close",
    shortcut: "Ctrl+W",
  },
];

/**
 * Handle context menu event
 */
const onContextMenu = (event: MouseEvent): void => {
  event.preventDefault();
  event.stopPropagation();

  if (contextMenuRef.value) {
    contextMenuRef.value.show(event.clientX, event.clientY);
  }
};

/**
 * Handle context menu action selection
 */
const handleContextMenuAction = (item: ContextMenuItem): void => {
  switch (item.action) {
    case "duplicate":
      emit("duplicate", props.tab);
      break;
    case "moveToNewPanel":
      emit("moveToNewPanel", props.tab);
      break;
    case "closeOthers":
      emit("closeOthers", props.tab);
      break;
    case "closeToRight":
      emit("closeToRight", props.tab);
      break;
    case "close":
      emit("close");
      break;
  }
};

const onDragStart = (event: DragEvent): void => {
  isDragging.value = true;
  if (event.dataTransfer) {
    const dragData = {
      tab: props.tab,
      sourcePanelId: props.panelId,
    };
    event.dataTransfer.setData("application/json", safeJsonStringify(dragData));
    event.dataTransfer.effectAllowed = "move";
  }
  emit("dragStart", props.tab);
};

const onDragEnd = (): void => {
  isDragging.value = false;
};

const onDragOver = (event: DragEvent): void => {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = "move";
  }
};

const onDrop = (event: DragEvent): void => {
  event.preventDefault();
  if (event.dataTransfer) {
    const draggedTabData = event.dataTransfer.getData("application/json");
    if (draggedTabData) {
      const dragData = safeJsonParse<{
        tab: Tab;
        sourcePanelId: string;
      } | null>(draggedTabData, null);
      if (!dragData) return;

      const draggedTab = dragData.tab;
      emit("drop", draggedTab, props.tab);
    }
  }
};
</script>

<style scoped>
/* Drag state styles */
.group.opacity-50 {
  transform: rotate(2deg) scale(0.98);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.4);
  z-index: 1000;
  position: relative;
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.3);
}

/* Add a subtle shimmer effect for new tabs */
@keyframes shimmer {
  0% {
    background-position: -200px 0;
  }

  100% {
    background-position: calc(200px + 100%) 0;
  }
}

.group::before {
  content: "";
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255, 255, 255, 0.1),
    transparent
  );
  transition: left 0.5s;
}

.group:hover::before {
  left: 100%;
}

/* Enhanced active tab indicator */
.active-tab::after {
  content: "";
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, #3b82f6, #06b6d4, #3b82f6);
  background-size: 200% 100%;
  animation: gradientShift 2s ease-in-out infinite;
}

@keyframes gradientShift {
  0%,
  100% {
    background-position: 0% 50%;
  }

  50% {
    background-position: 100% 50%;
  }
}

/* Drag preview enhancement */
.group[draggable="true"]:active {
  cursor: grabbing;
}

.group[draggable="true"]:hover {
  cursor: grab;
}
</style>
