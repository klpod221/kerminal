<template>
  <div
    class="group relative flex items-start gap-3 p-3.5 bg-[#2a2a2a] hover:bg-[#303030] border border-gray-700 hover:border-gray-600 rounded-lg cursor-pointer transition-all duration-200"
    @click="$emit('execute', command)"
  >
    <!-- Left indicator & favorite -->
    <div class="flex flex-col items-center gap-2 flex-shrink-0 pt-0.5">
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
      <div class="flex items-center gap-2">
        <code
          class="flex-1 text-xs text-gray-300 font-mono bg-black/30 px-2 py-1 rounded border border-gray-700/50 truncate"
        >
          {{ command.command }}
        </code>
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
      class="flex flex-col items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200 flex-shrink-0"
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
import { safeJsonParse } from "../../utils/helpers";
import { formatRelativeTime as formatTime } from "../../utils/formatter";

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

const handleDelete = () => {
  if (
    confirm(`Delete '${props.command.name}'? This action cannot be undone.`)
  ) {
    emit("delete", props.command);
  }
};

const parsedTags = computed(() => {
  return safeJsonParse<string[]>(props.command.tags, []);
});
</script>
