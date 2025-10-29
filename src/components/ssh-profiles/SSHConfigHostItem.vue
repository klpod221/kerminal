<template>
  <div
    class="group relative flex items-center gap-3 p-3.5 bg-[#1e2936] hover:bg-[#24303f] border border-blue-800/30 hover:border-blue-600/50 rounded-lg cursor-pointer transition-all duration-200"
    @click="$emit('connect', host)"
  >
    <!-- Icon indicator -->
    <div class="shrink-0">
      <div
        class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center"
      >
        <component :is="FileCode" class="w-5 h-5 text-blue-400" />
      </div>
    </div>

    <!-- Host info -->
    <div class="flex-1 min-w-0 space-y-1">
      <div class="flex items-center gap-2">
        <h4
          class="text-sm font-semibold text-blue-300 group-hover:text-blue-200 transition-colors truncate"
        >
          {{ host.name }}
        </h4>
      </div>
      <div class="flex items-center gap-2 text-xs text-gray-400">
        <code class="font-mono truncate">
          {{ displayAddress }}
        </code>
      </div>
      <div
        v-if="host.identityFile"
        class="flex items-center gap-1.5 text-[11px] text-gray-500"
      >
        <component :is="Key" class="w-3 h-3" />
        <span class="truncate">{{
          formatIdentityFile(host.identityFile)
        }}</span>
      </div>
    </div>

    <!-- Connect button (hover) -->
    <div
      class="flex items-center transition-opacity duration-200 shrink-0"
      :class="isTouch ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'"
      @click.stop
    >
      <Button
        title="Connect to this host"
        variant="ghost"
        size="sm"
        :icon="PlugZap"
        class="text-blue-400 hover:text-blue-300"
        @click="$emit('connect', host)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { SSHConfigHost } from "../../types/ssh";
import Button from "../ui/Button.vue";
import { FileCode, Key, PlugZap } from "lucide-vue-next";
import { useWindowSize } from "../../composables/useWindowSize";

const { isTouch } = useWindowSize();

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
