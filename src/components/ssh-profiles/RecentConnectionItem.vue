<template>
  <Card
    :hover="true"
    no-padding
    custom-class="p-2 cursor-pointer"
    @click="$emit('connect')"
  >
    <div class="flex items-center gap-3">
      <!-- Color indicator -->
      <div class="shrink-0">
        <div
          class="w-1 h-10 rounded-full transition-all duration-200"
          :style="{
            backgroundColor:
              entry.color || (entry.type === 'profile' ? '#6b7280' : '#3b82f6'),
          }"
        />
      </div>

      <!-- Info -->
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <h4 class="text-sm font-medium text-white truncate">
            {{ entry.name }}
          </h4>
          <span
            v-if="entry.type === 'config-host'"
            class="text-[10px] px-1.5 py-0.5 rounded-full border border-blue-500/30 text-blue-400 bg-blue-500/10"
          >
            Config
          </span>
        </div>
        <div class="text-xs text-gray-500 mt-0.5">
          <code class="font-mono truncate block">
            {{ entry.username }}@{{ entry.host }}
          </code>
          <span class="flex items-center text-gray-400 mt-1">
            <Clock class="w-3 h-3 mr-1" />
            {{ formatTimeAgo(entry.lastConnected) }}
          </span>
        </div>
      </div>

      <!-- Actions -->
      <div
        class="shrink-0 flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity"
      >
        <Button
          title="Connect"
          variant="ghost"
          size="sm"
          :icon="Power"
          class="p-1.5! text-green-400 hover:text-green-300 hover:bg-green-400/10"
          @click.stop="$emit('connect')"
        />
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { Clock, Power } from "lucide-vue-next";
import Card from "../ui/Card.vue";
import Button from "../ui/Button.vue";
import type { ConnectionHistoryEntry } from "../../types/ssh";
import { formatDistanceToNow } from "date-fns";

defineProps<{
  entry: ConnectionHistoryEntry;
}>();

defineEmits(["connect"]);

const formatTimeAgo = (timestamp: number) => {
  try {
    return formatDistanceToNow(timestamp, { addSuffix: true });
  } catch (e) {
    console.error("Failed to format timestamp:", e);
    return "Unknown time";
  }
};
</script>
