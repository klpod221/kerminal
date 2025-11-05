<template>
  <Card
    :hover="true"
    no-padding
    custom-class="p-3 cursor-pointer"
    @click="$emit('execute', entry)"
  >
    <div class="flex items-start gap-3">
      <!-- Command Index -->
      <div
        class="shrink-0 w-9 h-9 flex items-center justify-center rounded-md bg-gray-800/50 text-xs font-medium text-gray-400 transition-colors"
      >
        {{ entry.index + 1 }}
      </div>

      <!-- Main content -->
      <div class="flex-1 min-w-0 space-y-2">
        <!-- Command text with highlight -->
        <CommandPreview :command="entry.command" />

        <!-- Timestamp and Actions Row -->
        <div class="flex items-center justify-between">
          <!-- Timestamp -->
          <div
            v-if="entry.timestamp"
            class="flex items-center gap-1.5 text-xs text-gray-500"
          >
            <Clock :size="13" class="opacity-70" />
            <span>{{ formatTime(new Date(entry.timestamp)) }}</span>
          </div>
          <div v-else class="flex-1"></div>

          <!-- Action buttons -->
          <div class="flex items-center gap-1.5 shrink-0" @click.stop>
            <Button
              title="Copy command"
              variant="ghost"
              size="sm"
              :icon="Copy"
              class="p-1.5! text-gray-400 hover:text-blue-400 hover:bg-blue-600/20"
              @click="handleCopy"
            />
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { Clock, Copy } from "lucide-vue-next";
import Card from "../ui/Card.vue";
import Button from "../ui/Button.vue";
import CommandPreview from "../ui/CommandPreview.vue";
import type { CommandHistoryEntry } from "../../types/history";
import { formatRelativeTime as formatTime } from "../../utils/formatter";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { message } from "../../utils/message";

interface Props {
  entry: CommandHistoryEntry;
  highlight?: string;
}

const props = defineProps<Props>();

defineEmits<{
  execute: [entry: CommandHistoryEntry];
  copy: [entry: CommandHistoryEntry];
}>();

const handleCopy = async () => {
  try {
    await writeText(props.entry.command);
    message.success("Command copied to clipboard");
  } catch (error) {
    message.error("Failed to copy command");
  }
};
</script>

<style scoped>
/* Smooth animations */
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.history-item {
  animation: fadeIn 0.2s ease-out;
}
</style>
