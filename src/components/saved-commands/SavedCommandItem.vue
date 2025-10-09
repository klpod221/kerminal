<template>
  <div
    class="group flex items-center justify-between p-3 bg-[#2a2a2a] hover:bg-[#333333] hover:border-gray-500 border border-transparent rounded-lg cursor-pointer transition-all duration-300 transform hover:scale-[1.02] hover:shadow-lg"
    @click="$emit('execute', command)"
  >
    <div class="flex items-center space-x-3 flex-1 min-w-0">
      <div class="flex-shrink-0 flex items-center space-x-2">
        <!-- Favorite indicator -->
        <div
          v-if="command.isFavorite"
          class="text-yellow-400 transition-all duration-300"
        >
          <Star :size="12" class="fill-current" />
        </div>

        <!-- Color indicator -->
        <div
          class="w-2 h-2 rounded-full transition-all duration-300 group-hover:w-3 group-hover:h-3"
          :style="{ backgroundColor: fallbackColor || '#6b7280' }"
        ></div>
      </div>

      <div class="flex-1 min-w-0">
        <div class="flex items-center space-x-2">
          <p
            class="text-sm font-medium text-white group-hover:text-blue-300 truncate transition-colors duration-300"
          >
            {{ command.name }}
          </p>

          <!-- Usage count badge -->
          <span
            v-if="command.usageCount > 0"
            class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-gray-700 text-gray-300"
          >
            {{ command.usageCount }}x
          </span>

          <!-- Tags -->
          <div
            v-if="parsedTags.length > 0"
            class="flex items-center space-x-1"
          >
            <span
              v-for="tag in parsedTags.slice(0, 2)"
              :key="tag"
              class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-blue-900/30 text-blue-300"
            >
              {{ tag }}
            </span>
            <span
              v-if="parsedTags.length > 2"
              class="text-xs text-gray-400"
            >
              +{{ parsedTags.length - 2 }}
            </span>
          </div>
        </div>

        <!-- Command preview -->
        <p
          class="text-xs text-gray-400 group-hover:text-gray-300 truncate transition-colors duration-300 font-mono"
        >
          {{ command.command }}
        </p>

        <!-- Description -->
        <p
          v-if="command.description"
          class="text-xs text-gray-500 group-hover:text-gray-400 truncate transition-colors duration-300 mt-1"
        >
          {{ command.description }}
        </p>

        <!-- Last used info -->
        <div
          v-if="command.lastUsedAt"
          class="flex items-center space-x-2 mt-1"
        >
          <Clock :size="10" class="text-gray-500" />
          <span class="text-xs text-gray-500">
            Last used {{ formatRelativeTime(command.lastUsedAt) }}
          </span>
        </div>
      </div>
    </div>

    <!-- Action buttons -->
    <div
      class="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-all duration-300"
      @click.stop
    >
      <!-- Copy button -->
      <Button
        title="Copy command to clipboard"
        variant="ghost"
        size="sm"
        :icon="Copy"
        @click="$emit('copy', command)"
      />

      <!-- Favorite toggle -->
      <Button
        :title="command.isFavorite ? 'Remove from favorites' : 'Add to favorites'"
        variant="ghost"
        size="sm"
        :icon="command.isFavorite ? StarOff : Star"
        :class="command.isFavorite ? 'text-yellow-400 hover:text-yellow-300' : ''"
        @click="$emit('toggleFavorite', command)"
      />

      <!-- Edit button -->
      <Button
        title="Edit command"
        variant="ghost"
        size="sm"
        :icon="Edit3"
        @click="$emit('edit', command)"
      />

      <!-- Delete button -->
      <PopConfirm
        :title="`Delete command '${command.name}'?`"
        content="This action cannot be undone."
        placement="bottom"
        @confirm="$emit('delete', command)"
      >
        <Button
          title="Delete command"
          variant="ghost"
          size="sm"
          :icon="Trash2"
          class="text-red-400 hover:text-red-300"
        />
      </PopConfirm>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { Star, StarOff, Copy, Edit3, Trash2, Clock } from "lucide-vue-next";
import Button from "../ui/Button.vue";
import PopConfirm from "../ui/PopConfirm.vue";
import type { SavedCommand } from "../../types/savedCommand";

interface Props {
  command: SavedCommand;
  fallbackColor?: string;
}

const props = defineProps<Props>();

defineEmits<{
  execute: [command: SavedCommand];
  copy: [command: SavedCommand];
  toggleFavorite: [command: SavedCommand];
  edit: [command: SavedCommand];
  delete: [command: SavedCommand];
}>();

const parsedTags = computed(() => {
  if (!props.command.tags) return [];
  try {
    return JSON.parse(props.command.tags) as string[];
  } catch {
    return [];
  }
});

const formatRelativeTime = (dateString: string): string => {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMinutes = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMinutes < 1) return "just now";
  if (diffMinutes < 60) return `${diffMinutes}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
};
</script>
