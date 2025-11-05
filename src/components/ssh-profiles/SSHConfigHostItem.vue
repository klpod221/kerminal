<template>
  <Card
    :hover="true"
    no-padding
    custom-class="p-3 cursor-pointer !border-blue-800/30"
    @click="$emit('connect', host)"
  >
    <div class="flex items-center gap-3">
      <!-- Icon indicator -->
      <div class="shrink-0">
        <div
          class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center"
        >
          <component :is="FileCode" class="w-5 h-5 text-blue-400" />
        </div>
      </div>

      <!-- Host info -->
      <div class="flex-1 min-w-0">
        <div class="font-medium text-blue-300 text-sm truncate">
          {{ host.name }}
        </div>
        <div class="text-xs text-gray-500 mt-0.5">
          <code class="font-mono truncate">
            {{ displayAddress }}
          </code>
        </div>
        <div
          v-if="host.identityFile"
          class="flex items-center gap-1.5 text-[11px] text-gray-500 mt-1"
        >
          <component :is="Key" class="w-3 h-3" />
          <span class="truncate">{{
            formatIdentityFile(host.identityFile)
          }}</span>
        </div>
      </div>

      <!-- Connect button -->
      <div class="shrink-0">
        <Button
          title="Connect to this host"
          variant="ghost"
          size="sm"
          :icon="PlugZap"
          class="p-1.5! text-blue-400 hover:text-blue-300 hover:bg-blue-600/20"
          @click.stop="$emit('connect', host)"
        />
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { SSHConfigHost } from "../../types/ssh";
import Card from "../ui/Card.vue";
import Button from "../ui/Button.vue";
import { FileCode, Key, PlugZap } from "lucide-vue-next";

interface Props {
  host: SSHConfigHost;
}

interface Emits {
  connect: [host: SSHConfigHost];
}

const props = defineProps<Props>();
defineEmits<Emits>();

const displayAddress = computed(() => {
  if (props.host.user) {
    return `${props.host.user}@${props.host.hostname}:${props.host.port}`;
  }
  return `${props.host.hostname}:${props.host.port}`;
});

const formatIdentityFile = (path: string): string => {
  if (path.includes("/")) {
    const parts = path.split("/");
    return parts[parts.length - 1];
  }
  return path;
};
</script>
