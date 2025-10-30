<template>
  <div
    class="group relative flex items-start gap-3 p-3.5 bg-[#2a2a2a] hover:bg-[#303030] border border-gray-700 hover:border-gray-600 rounded-lg cursor-pointer transition-all duration-200"
    @click="$emit('execute', command)"
  >
    <!-- Left indicator & favorite -->
    <div class="flex flex-col items-center gap-2 shrink-0 pt-0.5">
      <!-- Color indicator -->
      <div
        class="w-1 h-8 rounded-full transition-all duration-200"
        :style="{ backgroundColor: fallbackColor || '#6b7280' }"
      />

      <!-- Favorite button - always visible -->
      <Button
        variant="ghost"
        :title="
          command.isFavorite ? 'Remove from favorites' : 'Add to favorites'
        "
        @click.stop="$emit('toggleFavorite', command)"
      >
        <Star
          :size="14"
          :class="{
            'fill-current': command.isFavorite,
            'text-yellow-400 hover:text-yellow-300': command.isFavorite,
            'text-gray-600 hover:text-yellow-400': !command.isFavorite,
          }"
        />
      </Button>
    </div>

    <!-- Main content -->
    <div class="flex-1 min-w-0 space-y-1.5">
      <!-- Title row -->
      <div class="flex items-center gap-2 flex-wrap">
        <h4
          class="text-sm font-semibold text-white group-hover:text-blue-300 transition-colors"
        >
          {{ command.name }}
        </h4>

        <!-- Usage count badge -->
        <Badge v-if="command.usageCount > 0" variant="gray" size="xs">
          {{ command.usageCount }}× used
        </Badge>

        <!-- Tags -->
        <div v-if="parsedTags.length > 0" class="flex items-center gap-1">
          <Badge
            v-for="tag in parsedTags.slice(0, 3)"
            :key="tag"
            variant="primary"
            size="xs"
          >
            {{ tag }}
          </Badge>
          <Badge v-if="parsedTags.length > 3" variant="outline" size="xs">
            +{{ parsedTags.length - 3 }}
          </Badge>
        </div>
      </div>

      <!-- Command preview -->
      <div class="command-preview-container" @click.stop>
        <SyntaxHighlight
          :code="command.command"
          language="shell"
          class="command-preview-code"
        />
      </div>

      <!-- Description -->
      <p
        v-if="command.description"
        class="text-xs text-gray-400 leading-relaxed line-clamp-2"
      >
        {{ command.description }}
      </p>

      <!-- Metadata row -->
      <div class="flex items-center gap-3 text-xs text-gray-500">
        <div v-if="command.lastUsedAt" class="flex items-center gap-1">
          <Clock :size="12" />
          <span>{{ formatTime(new Date(command.lastUsedAt)) }}</span>
        </div>
      </div>
    </div>

    <!-- Action buttons (hover) -->
    <div
      class="flex flex-col items-center gap-1 transition-opacity duration-200 shrink-0"
      :class="isTouch ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'"
      @click.stop
    >
      <Button
        title="Copy command"
        variant="ghost"
        size="sm"
        :icon="Copy"
        @click="$emit('copy', command)"
      />

      <Button
        title="Edit command"
        variant="ghost"
        size="sm"
        :icon="Edit3"
        @click="$emit('edit', command)"
      />

      <Button
        title="Delete command"
        variant="ghost"
        size="sm"
        :icon="Trash2"
        class="text-red-400 hover:text-red-300"
        @click="handleDelete"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { SavedCommand } from "../../types/savedCommand";
import { Copy, Star, Edit3, Trash2, Clock } from "lucide-vue-next";
import Badge from "../ui/Badge.vue";
import Button from "../ui/Button.vue";
import SyntaxHighlight from "../ui/SyntaxHighlight.vue";
import { safeJsonParse } from "../../utils/helpers";
import { formatRelativeTime as formatTime } from "../../utils/formatter";
import { useWindowSize } from "../../composables/useWindowSize";
import { showConfirm } from "../../utils/message";

const { isTouch } = useWindowSize();

interface Props {
  command: SavedCommand;
  fallbackColor?: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  execute: [command: SavedCommand];
  copy: [command: SavedCommand];
  toggleFavorite: [command: SavedCommand];
  edit: [command: SavedCommand];
  delete: [command: SavedCommand];
}>();

const handleDelete = async () => {
  const confirmed = await showConfirm(
    "Delete Command",
    `Delete '${props.command.name}'? This action cannot be undone.`,
  );
  if (confirmed) {
    emit("delete", props.command);
  }
};

const parsedTags = computed(() => {
  return safeJsonParse<string[]>(props.command.tags, []);
});
</script>

<style scoped>
.command-preview-container {
  position: relative;
  border-radius: 0.375rem;
  overflow: hidden;
  border: 1px solid rgba(75, 85, 99, 0.5);
  background-color: rgba(0, 0, 0, 0.3);
  max-height: 150px;
  overflow-y: auto;
}

.command-preview-container::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.command-preview-container::-webkit-scrollbar-track {
  background: transparent;
}

.command-preview-container::-webkit-scrollbar-thumb {
  background-color: rgba(156, 163, 175, 0.3);
  border-radius: 3px;
}

.command-preview-container::-webkit-scrollbar-thumb:hover {
  background-color: rgba(156, 163, 175, 0.5);
}

.command-preview-code {
  margin: 0;
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  line-height: 1.5;
}
</style>
