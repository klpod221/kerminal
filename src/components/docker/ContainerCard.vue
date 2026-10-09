<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <div class="flex items-start justify-between w-full">
    <div class="flex items-start gap-2.5 min-w-0 flex-1">
      <!-- State indicator dot -->
      <div
        class="mt-1.5 w-2.5 h-2.5 rounded-full shrink-0 ring-2"
        :class="
          container.state.toLowerCase() === 'running'
            ? 'bg-green-500 ring-green-500/25'
            : container.state.toLowerCase() === 'paused'
            ? 'bg-yellow-500 ring-yellow-500/25'
            : 'bg-gray-500 ring-gray-500/25'
        "
      ></div>

      <div class="min-w-0 flex-1 space-y-1">
        <!-- Name and short ID -->
        <div class="flex items-center gap-2 flex-wrap">
          <span
            class="font-semibold text-gray-100 text-sm truncate max-w-[200px]"
            :title="container.name"
          >
            {{ container.name }}
          </span>
          <span class="text-xs text-gray-400 font-mono">
            {{ container.id.slice(0, 10) }}
          </span>
        </div>

        <!-- Image tag -->
        <div
          class="text-xs text-gray-300 font-mono bg-black/40 border border-white/5 inline-block px-1.5 py-0.5 rounded truncate max-w-[260px]"
          :title="container.image"
        >
          {{ container.image }}
        </div>

        <!-- Compose service badge if available -->
        <div
          v-if="container.compose_service"
          class="flex items-center gap-1.5 text-xs text-indigo-400 font-medium"
        >
          <Layers class="w-3.5 h-3.5 shrink-0" />
          <span class="truncate">{{ container.compose_service }}</span>
        </div>

        <!-- Status & Resource stats -->
        <div class="text-xs text-gray-400 flex flex-wrap items-center gap-x-2.5 gap-y-1 pt-0.5">
          <span class="truncate">{{ container.status }}</span>
          <template v-if="stats">
            <span class="text-blue-400 font-mono text-xs font-medium">CPU: {{ stats.cpu }}</span>
            <span class="text-purple-400 font-mono text-xs font-medium">Mem: {{ stats.memory.split('/')[0].trim() }}</span>
          </template>
        </div>

        <!-- Ports -->
        <div
          v-if="container.ports"
          class="text-xs text-blue-400/90 font-mono truncate max-w-[280px]"
          :title="container.ports"
        >
          {{ container.ports }}
        </div>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-1 shrink-0 ml-2">
      <div v-if="actionLoading" class="p-1.5 text-gray-400">
        <RefreshCw class="w-4 h-4 animate-spin" />
      </div>
      <template v-else>
        <div class="flex items-center gap-1 opacity-80 group-hover:opacity-100 transition-opacity">
          <button
            v-if="container.state.toLowerCase() !== 'running'"
            @click="$emit('start')"
            class="p-1.5 bg-green-500/10 text-green-400 hover:bg-green-500/20 rounded transition-colors"
            title="Start Container"
          >
            <Play class="w-3.5 h-3.5" />
          </button>
          <button
            v-if="container.state.toLowerCase() === 'running'"
            @click="$emit('stop')"
            class="p-1.5 bg-orange-500/10 text-orange-400 hover:bg-orange-500/20 rounded transition-colors"
            title="Stop Container"
          >
            <Square class="w-3.5 h-3.5" />
          </button>
          <button
            @click="$emit('restart')"
            class="p-1.5 bg-blue-500/10 text-blue-400 hover:bg-blue-500/20 rounded transition-colors"
            title="Restart Container"
          >
            <RotateCw class="w-3.5 h-3.5" />
          </button>
          <button
            @click="$emit('logs')"
            class="p-1.5 bg-gray-500/10 text-gray-300 hover:bg-gray-500/20 rounded transition-colors"
            title="View Logs"
          >
            <FileText class="w-3.5 h-3.5" />
          </button>
          <button
            @click="$emit('remove')"
            class="p-1.5 bg-red-500/10 text-red-400 hover:bg-red-500/20 rounded transition-colors"
            title="Remove Container"
          >
            <Trash2 class="w-3.5 h-3.5" />
          </button>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Play, Square, RotateCw, FileText, Trash2, Layers, RefreshCw } from "lucide-vue-next";
import type { ContainerInfo, ContainerStatInfo } from "../../services/docker";

defineProps<{
  container: ContainerInfo;
  stats?: ContainerStatInfo;
  actionLoading?: boolean;
}>();

defineEmits<{
  (e: "start"): void;
  (e: "stop"): void;
  (e: "restart"): void;
  (e: "logs"): void;
  (e: "remove"): void;
}>();
</script>
