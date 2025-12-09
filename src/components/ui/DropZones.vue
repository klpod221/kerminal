<template>
  <div
    v-show="showDropZones"
    class="absolute top-8 left-0 right-0 bottom-0 z-500 pointer-events-auto"
    @drop="onDrop"
    @dragover="onDragOver"
    @dragenter="onDragEnter"
    @dragleave="onDragLeave"
  >
    <!-- Top Drop Zone -->
    <div
      class="absolute top-0 left-[20%] right-[20%] h-20 bg-blue-500/20 border-2 border-dashed border-blue-500/60 rounded-lg flex items-center justify-center transition-all duration-200 opacity-90"
      :class="{
        'bg-blue-500/40 border-blue-500 opacity-100 scale-[1.02]':
          activeZone === 'top',
      }"
      @dragenter="setActiveZone('top')"
    >
      <div
        class="flex flex-col items-center justify-center text-white/90 font-medium"
      >
        <div class="text-2xl mb-1">⬆</div>
        <div
          class="text-xs uppercase tracking-wider bg-black/50 px-1.5 py-0.5 rounded"
        >
          Split Top
        </div>
      </div>
    </div>

    <!-- Bottom Drop Zone -->
    <div
      class="absolute bottom-2 left-[20%] right-[20%] h-20 bg-blue-500/20 border-2 border-dashed border-blue-500/60 rounded-lg flex items-center justify-center transition-all duration-200 opacity-90"
      :class="{
        'bg-blue-500/40 border-blue-500 opacity-100 scale-[1.02]':
          activeZone === 'bottom',
      }"
      @dragenter="setActiveZone('bottom')"
    >
      <div
        class="flex flex-col items-center justify-center text-white/90 font-medium"
      >
        <div class="text-2xl mb-1">⬇</div>
        <div
          class="text-xs uppercase tracking-wider bg-black/50 px-1.5 py-0.5 rounded"
        >
          Split Bottom
        </div>
      </div>
    </div>

    <!-- Left Drop Zone -->
    <div
      class="absolute left-2 top-[20%] bottom-[20%] w-20 bg-blue-500/20 border-2 border-dashed border-blue-500/60 rounded-lg flex items-center justify-center transition-all duration-200 opacity-90"
      :class="{
        'bg-blue-500/40 border-blue-500 opacity-100 scale-[1.02]':
          activeZone === 'left',
      }"
      @dragenter="setActiveZone('left')"
    >
      <div
        class="flex flex-col items-center justify-center text-white/90 font-medium"
      >
        <div class="text-2xl mb-1">⬅</div>
        <div
          class="text-xs uppercase tracking-wider bg-black/50 px-1.5 py-0.5 rounded text-center"
        >
          Split Left
        </div>
      </div>
    </div>

    <!-- Right Drop Zone -->
    <div
      class="absolute right-2 top-[20%] bottom-[20%] w-20 bg-blue-500/20 border-2 border-dashed border-blue-500/60 rounded-lg flex items-center justify-center transition-all duration-200 opacity-90"
      :class="{
        'bg-blue-500/40 border-blue-500 opacity-100 scale-[1.02]':
          activeZone === 'right',
      }"
      @dragenter="setActiveZone('right')"
    >
      <div
        class="flex flex-col items-center justify-center text-white/90 font-medium"
      >
        <div class="text-2xl mb-1">➡</div>
        <div
          class="text-xs uppercase tracking-wider bg-black/50 px-1.5 py-0.5 rounded text-center"
        >
          Split Right
        </div>
      </div>
    </div>

    <!-- Center Drop Zone -->
    <div
      class="absolute top-[30%] bottom-[30%] left-[30%] right-[30%] min-w-[120px] min-h-20 bg-blue-500/20 border-2 border-dashed border-blue-500/60 rounded-lg flex items-center justify-center transition-all duration-200 opacity-90"
      :class="{
        'bg-blue-500/40 border-blue-500 opacity-100 scale-[1.02]':
          activeZone === 'center',
      }"
      @dragenter="setActiveZone('center')"
    >
      <div
        class="flex flex-col items-center justify-center text-white/90 font-medium"
      >
        <div class="text-2xl mb-1">📁</div>
        <div
          class="text-xs uppercase tracking-wider bg-black/50 px-1.5 py-0.5 rounded"
        >
          Add to Panel
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import type { Tab } from "../../types/panel";
import { safeJsonParse } from "../../utils/helpers";

interface DropZonesProps {
  showDropZones: boolean;
  panelId: string;
}

interface DropZonesEmits {
  splitPanel: [
    direction: "top" | "bottom" | "left" | "right",
    draggedTab: Tab,
    sourcePanelId: string,
  ];
  moveTab: [draggedTab: Tab, sourcePanelId: string];
}

const props = defineProps<DropZonesProps>();
const emit = defineEmits<DropZonesEmits>();

const activeZone = ref<"top" | "bottom" | "left" | "right" | "center" | null>(
  null,
);
let hideActiveZoneTimeout: ReturnType<typeof setTimeout> | null = null;

/**
 * Set the active drop zone
 * @param {string} zone - The zone identifier
 */
const setActiveZone = (
  zone: "top" | "bottom" | "left" | "right" | "center",
): void => {
  if (hideActiveZoneTimeout) {
    clearTimeout(hideActiveZoneTimeout);
    hideActiveZoneTimeout = null;
  }

  activeZone.value = zone;
};

/**
 * Handle drag over event
 * @param {DragEvent} event - The drag event
 */
const onDragOver = (event: DragEvent): void => {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = "move";
  }
};

/**
 * Handle drag enter event
 * @param {DragEvent} event - The drag event
 */
const onDragEnter = (event: DragEvent): void => {
  event.preventDefault();
};

/**
 * Handle drag leave event
 * @param {DragEvent} event - The drag event
 */
const onDragLeave = (event: DragEvent): void => {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const isLeavingContainer =
    event.clientX < rect.left ||
    event.clientX > rect.right ||
    event.clientY < rect.top ||
    event.clientY > rect.bottom;

  if (isLeavingContainer) {
    hideActiveZoneTimeout = setTimeout(() => {
      activeZone.value = null;
    }, 50);
  }
};

/**
 * Parse drag data from event
 * @param {DragEvent} event - The drop event
 * @returns {object|null} Parsed drag data or null
 */
const parseDragData = (
  event: DragEvent,
): { tab: Tab; sourcePanelId: string } | null => {
  if (!event.dataTransfer) return null;

  const draggedTabData = event.dataTransfer.getData("application/json");
  if (!draggedTabData) return null;

  return safeJsonParse<{
    tab: Tab;
    sourcePanelId: string;
  } | null>(draggedTabData, null);
};

/**
 * Handle drop action based on active zone
 * @param {object} dragData - The parsed drag data
 */
const handleDropAction = (dragData: {
  tab: Tab;
  sourcePanelId: string;
}): void => {
  const { tab: draggedTab, sourcePanelId } = dragData;

  if (activeZone.value && activeZone.value !== "center") {
    emit("splitPanel", activeZone.value, draggedTab, sourcePanelId);
  } else if (activeZone.value === "center") {
    if (sourcePanelId !== props.panelId) {
      emit("moveTab", draggedTab, sourcePanelId);
    }
  }
};

/**
 * Handle drop event
 * @param {DragEvent} event - The drop event
 */
const onDrop = (event: DragEvent): void => {
  event.preventDefault();

  const dragData = parseDragData(event);
  if (dragData) {
    handleDropAction(dragData);
  }

  if (hideActiveZoneTimeout) {
    clearTimeout(hideActiveZoneTimeout);
    hideActiveZoneTimeout = null;
  }
  activeZone.value = null;
};
</script>
