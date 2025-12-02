<template>
  <Card
    :hover="true"
    no-padding
    custom-class="p-2 cursor-pointer"
    @click="$emit('execute', command)"
  >
    <div class="flex items-start gap-3">
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
          size="sm"
          :title="
            command.isFavorite ? 'Remove from favorites' : 'Add to favorites'
          "
          class="p-1.5!"
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
          <h4 class="text-sm font-medium text-white truncate">
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
        <CommandPreview :command="command.command" max-height="150px" />

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

      <!-- Actions -->
      <div class="shrink-0 flex flex-col items-center">
        <Button
          title="Run command"
          variant="ghost"
          size="sm"
          :icon="Play"
          class="p-1.5! text-gray-400 hover:text-blue-400 hover:bg-blue-600/20"
          @click.stop="$emit('execute', command)"
        />

        <!-- Copy button -->
        <Button
          title="Copy command"
          variant="ghost"
          size="sm"
          :icon="Copy"
          class="p-1.5! text-gray-400 hover:text-blue-400 hover:bg-blue-600/20"
          @click.stop="$emit('copy', command)"
        />

        <!-- Edit button -->
        <Button
          title="Edit command"
          variant="ghost"
          size="sm"
          :icon="Edit3"
          class="p-1.5! text-gray-400 hover:text-blue-400 hover:bg-blue-600/20"
          @click.stop="$emit('edit', command)"
        />

        <!-- Delete button -->
        <Button
          title="Delete command"
          variant="ghost"
          size="sm"
          :icon="Trash2"
          class="p-1.5! text-gray-400 hover:text-red-400 hover:bg-red-600/20"
          @click.stop="handleDelete"
        />
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { SavedCommand } from "../../types/savedCommand";
import { Copy, Star, Edit3, Trash2, Clock, Play } from "lucide-vue-next";
import Card from "../ui/Card.vue";
import Badge from "../ui/Badge.vue";
import Button from "../ui/Button.vue";
import CommandPreview from "../ui/CommandPreview.vue";
import { safeJsonParse } from "../../utils/helpers";
import { formatRelativeTime as formatTime } from "../../utils/formatter";
import { showConfirm } from "../../utils/message";

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
