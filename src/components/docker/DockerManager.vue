<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <div class="p-3">
    <div class="flex justify-between items-center mb-3">
      <h2 class="text-sm font-semibold flex items-center gap-1.5 bg-linear-to-r from-blue-400 to-indigo-400 bg-clip-text text-transparent">
        <BoxIcon class="w-4 h-4 text-blue-400" />
        Docker Containers
      </h2>
      <button
        @click="fetchContainers"
        class="text-gray-400 hover:text-white p-1 rounded-md hover:bg-white/5 transition-colors"
        :class="loading ? 'animate-spin' : ''"
      >
        <RefreshCwIcon class="w-3.5 h-3.5" />
      </button>
    </div>

    <div v-if="error" class="bg-red-500/10 border border-red-500/20 text-red-400 p-2 rounded-lg mb-3 text-xs backdrop-blur-md">
      {{ error }}
    </div>

    <div v-if="loading && containers.length === 0" class="flex flex-col items-center justify-center p-6 text-gray-500">
      <RefreshCwIcon class="w-6 h-6 animate-spin mb-2 opacity-50" />
      <span class="text-[10px] font-medium tracking-wider uppercase">Loading containers...</span>
    </div>

    <div v-else-if="containers.length === 0" class="flex flex-col items-center justify-center p-6 text-gray-500 border border-dashed border-gray-700/50 rounded-lg bg-white/1">
      <BoxIcon class="w-6 h-6 mb-2 opacity-30" />
      <span class="text-xs">No containers found</span>
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="container in containers"
        :key="container.id"
        class="relative overflow-hidden bg-white/2 border border-white/5 p-2.5 rounded-lg hover:bg-white/4 transition-all duration-300 group shadow-sm"
      >
        <div class="flex items-start justify-between">
          <div class="flex items-start gap-2.5">
            <!-- Status Dot / Icon -->
            <div
              class="mt-1 w-2 h-2 rounded-full ring-2 shadow-[0_0_8px_rgba(0,0,0,0.5)] shrink-0"
              :class="container.state.toLowerCase() === 'running' ? 'bg-green-500 ring-green-500/20' : 'bg-gray-500 ring-gray-500/20'"
            ></div>

            <div class="min-w-0">
              <div class="font-medium text-gray-200 text-xs truncate">
                {{ container.name }}
              </div>
              <div class="text-[9px] text-gray-400 font-mono mt-0.5 bg-black/20 inline-block px-1 rounded truncate max-w-[200px]">
                {{ container.image }}
              </div>

              <div class="text-[10px] text-gray-500 mt-1.5 flex items-center gap-1">
                <FileTextIcon class="w-2.5 h-2.5 shrink-0" />
                <span class="truncate">{{ container.status }}</span>
              </div>

              <div v-if="container.ports" class="text-[9px] text-blue-400/70 mt-1 font-mono truncate">
                {{ container.ports }}
              </div>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex flex-col gap-1 opacity-10 md:opacity-0 group-hover:opacity-100 transition-opacity shrink-0 ml-2">
            <button
              v-if="container.state.toLowerCase() !== 'running'"
              @click="startContainer(container.id)"
              class="p-1 bg-green-500/10 text-green-400 hover:bg-green-500/20 rounded transition-colors"
              title="Start Container"
            >
              <PlayIcon class="w-3 h-3" />
            </button>
            <button
              v-if="container.state.toLowerCase() === 'running'"
              @click="stopContainer(container.id)"
              class="p-1 bg-orange-500/10 text-orange-400 hover:bg-orange-500/20 rounded transition-colors"
              title="Stop Container"
            >
              <SquareIcon class="w-3 h-3" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { executeRemoteDockerCommand } from '../../services/docker';
import { BoxIcon, RefreshCwIcon, PlayIcon, SquareIcon, FileTextIcon } from 'lucide-vue-next';

const props = defineProps<{
  profileId: string;
  isActive: boolean;
}>();

interface DockerContainer {
  id: string;
  name: string;
  image: string;
  state: string;
  status: string;
  ports: string;
}

const containers = ref<DockerContainer[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

const fetchContainers = async () => {
  if (!props.profileId || !props.isActive) return;
  loading.value = true;
  error.value = null;
  containers.value = [];
  try {
    const data: DockerContainer[] = await executeRemoteDockerCommand(
      props.profileId,
      "ps"
    );
    containers.value = data;
  } catch (err: any) {
    error.value = err.message || 'Failed to fetch containers';
  } finally {
    loading.value = false;
  }
};

const startContainer = async (id: string) => {
  if (!props.profileId) return;
  try {
    await executeRemoteDockerCommand(props.profileId, "start", id);
    await fetchContainers();
  } catch (err: any) {
    error.value = err.message || 'Failed to start container';
  }
};

const stopContainer = async (id: string) => {
  if (!props.profileId) return;
  try {
    await executeRemoteDockerCommand(props.profileId, "stop", id);
    await fetchContainers();
  } catch (err: any) {
    error.value = err.message || 'Failed to stop container';
  }
};

watch(() => props.profileId, () => {
  fetchContainers();
});

onMounted(() => {
  fetchContainers();
});
</script>
