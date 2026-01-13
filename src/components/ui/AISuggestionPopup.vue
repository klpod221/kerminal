<template>
  <Transition
    enter-active-class="transition-all duration-150 ease-out"
    enter-from-class="opacity-0 translate-y-1 scale-95"
    enter-to-class="opacity-100 translate-y-0 scale-100"
    leave-active-class="transition-all duration-100 ease-in"
    leave-from-class="opacity-100 translate-y-0 scale-100"
    leave-to-class="opacity-0 translate-y-1 scale-95"
  >
    <div
      v-if="visible && suggestions.length > 0"
      class="absolute z-50 bg-bg-tertiary border border-gray-700 rounded-lg shadow-xl overflow-hidden"
      :style="popupStyle"
    >
      <!-- Header -->
      <div
        class="flex items-center justify-between px-3 py-1.5 bg-bg-quaternary border-b border-gray-700"
      >
        <div class="flex items-center gap-2">
          <Sparkles class="w-3.5 h-3.5 text-purple-400" />
          <span class="text-xs font-medium text-gray-400">AI Suggestions</span>
        </div>
        <div class="flex items-center gap-1">
          <span v-if="latencyMs" class="text-[10px] text-gray-500"
            >{{ latencyMs }}ms</span
          >
          <kbd
            class="px-1 py-0.5 text-[10px] bg-gray-700 border border-gray-600 rounded text-gray-400"
          >
            ↑↓
          </kbd>
          <kbd
            class="px-1 py-0.5 text-[10px] bg-gray-700 border border-gray-600 rounded text-gray-400"
          >
            ↵
          </kbd>
          <kbd
            class="px-1 py-0.5 text-[10px] bg-gray-700 border border-gray-600 rounded text-gray-400"
          >
            esc
          </kbd>
        </div>
      </div>

      <!-- Suggestion List -->
      <div class="max-h-48 overflow-y-auto">
        <div
          v-for="(suggestion, index) in suggestions"
          :key="suggestion.command"
          class="px-3 py-2 cursor-pointer transition-colors border-b border-gray-800 last:border-b-0"
          :class="
            index === selectedIndex
              ? 'bg-purple-600/20 border-l-2 border-l-purple-500'
              : 'hover:bg-gray-800/50 border-l-2 border-l-transparent'
          "
          @click="selectSuggestion(index)"
          @mouseenter="selectedIndex = index"
        >
          <div class="flex items-center justify-between gap-3">
            <div class="flex-1 min-w-0">
              <code class="text-sm font-mono text-white">{{
                suggestion.command
              }}</code>
              <p
                v-if="suggestion.description"
                class="text-xs text-gray-400 truncate mt-0.5"
              >
                {{ suggestion.description }}
              </p>
            </div>
            <div
              v-if="suggestion.confidence"
              class="shrink-0 w-6 h-1.5 bg-gray-700 rounded-full overflow-hidden"
              :title="`${Math.round(suggestion.confidence * 100)}% confidence`"
            >
              <div
                class="h-full rounded-full"
                :class="getConfidenceColor(suggestion.confidence)"
                :style="{ width: `${suggestion.confidence * 100}%` }"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Loading State -->
      <div v-if="isLoading" class="px-3 py-2 flex items-center gap-2">
        <div
          class="w-4 h-4 border-2 border-purple-400 border-t-transparent rounded-full animate-spin"
        />
        <span class="text-xs text-gray-400">Getting suggestions...</span>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { Sparkles } from "lucide-vue-next";
import type { AISuggestion } from "../../types/ai";

interface Props {
  visible: boolean;
  suggestions: AISuggestion[];
  isLoading?: boolean;
  latencyMs?: number;
  position?: { x: number; y: number };
}

const props = withDefaults(defineProps<Props>(), {
  isLoading: false,
  latencyMs: undefined,
  position: () => ({ x: 0, y: 0 }),
});

const emit = defineEmits<{
  select: [suggestion: AISuggestion];
  close: [];
}>();

const selectedIndex = ref(0);

const popupStyle = computed(() => ({
  left: `${props.position.x}px`,
  top: `${props.position.y}px`,
  minWidth: "280px",
  maxWidth: "450px",
}));

// Reset selection when suggestions change
watch(
  () => props.suggestions,
  () => {
    selectedIndex.value = 0;
  },
);

// Keyboard navigation
function handleKeydown(event: KeyboardEvent) {
  if (!props.visible || props.suggestions.length === 0) return;

  switch (event.key) {
    case "ArrowDown":
      event.preventDefault();
      event.stopPropagation();
      selectedIndex.value =
        (selectedIndex.value + 1) % props.suggestions.length;
      break;
    case "ArrowUp":
      event.preventDefault();
      event.stopPropagation();
      selectedIndex.value =
        (selectedIndex.value - 1 + props.suggestions.length) %
        props.suggestions.length;
      break;
    case "Enter":
    case "Tab":
      event.preventDefault();
      event.stopPropagation();
      selectSuggestion(selectedIndex.value);
      break;
    case "Escape":
      event.preventDefault();
      event.stopPropagation();
      emit("close");
      break;
  }
}

function selectSuggestion(index: number) {
  if (index >= 0 && index < props.suggestions.length) {
    emit("select", props.suggestions[index]);
  }
}

function getConfidenceColor(confidence: number): string {
  if (confidence >= 0.8) return "bg-green-500";
  if (confidence >= 0.5) return "bg-yellow-500";
  return "bg-gray-500";
}

onMounted(() => {
  document.addEventListener("keydown", handleKeydown, true);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown, true);
});
</script>
