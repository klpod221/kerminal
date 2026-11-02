<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <div class="p-3">
    <!-- Header with Engine Badge and Refresh -->
    <div class="flex justify-between items-center mb-3">
      <div class="flex items-center gap-2">
        <h2 class="text-sm font-semibold flex items-center gap-1.5 bg-gradient-to-r from-blue-400 to-indigo-400 bg-clip-text text-transparent">
          <Box class="w-4 h-4 text-blue-400" />
          Container Manager
        </h2>
        <span
          v-if="detectedEngine"
          class="px-1.5 py-0.5 rounded text-xs font-mono uppercase font-semibold"
          :class="detectedEngine === 'docker' ? 'bg-blue-500/20 text-blue-300 border border-blue-500/30' : 'bg-purple-500/20 text-purple-300 border border-purple-500/30'"
        >
          {{ detectedEngine }}
        </span>
      </div>

      <div class="flex items-center gap-1">
        <button
          @click="fetchContainers"
          class="text-gray-400 hover:text-white p-1 rounded-md hover:bg-white/5 transition-colors"
          :class="loading ? 'animate-spin' : ''"
          title="Refresh containers"
        >
          <RefreshCw class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Error state -->
    <div v-if="error" class="bg-red-500/10 border border-red-500/20 text-red-400 p-2.5 rounded-lg mb-3 text-xs leading-relaxed">
      {{ error }}
    </div>

    <!-- Filter & Search Toolbar -->
    <div v-if="containers.length > 0 || searchQuery" class="space-y-2 mb-3">
      <div class="relative">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Filter containers, images, compose..."
          class="w-full bg-black/40 border border-gray-700/60 rounded px-2.5 py-1.5 text-xs text-gray-200 placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors"
        />
      </div>

      <div class="flex items-center justify-between text-xs">
        <div class="flex items-center gap-1.5">
          <button
            v-for="st in statusFilters"
            :key="st.id"
            @click="activeStatusFilter = st.id"
            class="px-2.5 py-0.5 rounded text-xs transition-colors"
            :class="activeStatusFilter === st.id ? 'bg-blue-600/30 text-blue-300 font-medium border border-blue-500/40' : 'text-gray-400 hover:text-white bg-gray-800/40'"
          >
            {{ st.label }}
          </button>
        </div>

        <button
          v-if="hasComposeProjects"
          @click="groupByCompose = !groupByCompose"
          class="text-xs text-gray-400 hover:text-blue-300 flex items-center gap-1"
        >
          <Layers class="w-3.5 h-3.5" />
          <span>{{ groupByCompose ? "Grouped" : "Flat" }}</span>
        </button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading && containers.length === 0" class="flex flex-col items-center justify-center p-8 text-gray-500">
      <RefreshCw class="w-6 h-6 animate-spin mb-2 opacity-50 text-blue-400" />
      <span class="text-xs font-medium tracking-wider uppercase">Loading containers...</span>
    </div>

    <!-- Empty State -->
    <div v-else-if="filteredContainers.length === 0" class="flex flex-col items-center justify-center p-6 text-gray-500 border border-dashed border-gray-700/50 rounded-lg bg-white/1">
      <Box class="w-6 h-6 mb-2 opacity-30" />
      <span class="text-xs">{{ searchQuery ? "No matching containers" : "No containers found" }}</span>
    </div>

    <!-- Containers List -->
    <div v-else class="space-y-3">
      <!-- Grouped by Compose project -->
      <template v-if="groupByCompose && hasComposeProjects">
        <div
          v-for="(group, groupName) in groupedContainers"
          :key="groupName"
          class="space-y-1.5"
        >
          <div class="flex items-center gap-1.5 text-xs font-semibold text-gray-300 px-1 pt-1">
            <Layers class="w-3.5 h-3.5 text-indigo-400" />
            <span>{{ groupName }}</span>
            <span class="text-xs text-gray-500 font-normal">({{ group.length }})</span>
          </div>

          <div class="space-y-1.5 pl-1.5 border-l border-gray-800">
            <div
              v-for="container in group"
              :key="container.id"
              class="relative bg-white/2 border border-white/5 p-2 rounded-lg hover:bg-white/4 transition-all duration-200 group"
            >
              <ContainerCard
                :container="container"
                :stats="containerStats[container.id]"
                :action-loading="actionLoadingId === container.id"
                @start="handleAction('start', container)"
                @stop="handleAction('stop', container)"
                @restart="handleAction('restart', container)"
                @logs="openLogs(container)"
                @remove="handleRemove(container)"
              />
            </div>
          </div>
        </div>
      </template>

      <!-- Flat view -->
      <template v-else>
        <div
          v-for="container in filteredContainers"
          :key="container.id"
          class="relative bg-white/2 border border-white/5 p-2.5 rounded-lg hover:bg-white/4 transition-all duration-200 group shadow-xs"
        >
          <ContainerCard
            :container="container"
            :stats="containerStats[container.id]"
            :action-loading="actionLoadingId === container.id"
            @start="handleAction('start', container)"
            @stop="handleAction('stop', container)"
            @restart="handleAction('restart', container)"
            @logs="openLogs(container)"
            @remove="handleRemove(container)"
          />
        </div>
      </template>
    </div>

    <!-- Logs Modal -->
    <ContainerLogsModal
      :container="activeLogsContainer"
      :profile-id="props.profileId"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import {
  Box,
  RefreshCw,
  Layers,
} from "lucide-vue-next";
import {
  getContainerList,
  executeContainerAction,
  getContainerStats,
  type ContainerInfo,
  type ContainerStatInfo,
  type ContainerActionType,
} from "../../services/docker";
import ContainerCard from "./ContainerCard.vue";
import ContainerLogsModal from "./ContainerLogsModal.vue";
import { useOverlay } from "../../composables/useOverlay";
import { message, showConfirm } from "../../utils/message";

const props = defineProps<{
  profileId: string;
  isActive: boolean;
}>();

const { openOverlay } = useOverlay();

const containers = ref<ContainerInfo[]>([]);
const containerStats = ref<Record<string, ContainerStatInfo>>({});
const loading = ref(false);
const error = ref<string | null>(null);
const actionLoadingId = ref<string | null>(null);
const searchQuery = ref("");
const activeStatusFilter = ref<"all" | "running" | "stopped">("all");
const groupByCompose = ref(true);
const activeLogsContainer = ref<ContainerInfo | null>(null);

const statusFilters = [
  { id: "all" as const, label: "All" },
  { id: "running" as const, label: "Running" },
  { id: "stopped" as const, label: "Stopped" },
];

const detectedEngine = computed(() => {
  return containers.value[0]?.engine || null;
});

const hasComposeProjects = computed(() => {
  return containers.value.some((c) => !!c.compose_project);
});

const filteredContainers = computed(() => {
  let list = containers.value;

  if (activeStatusFilter.value === "running") {
    list = list.filter((c) => c.state.toLowerCase() === "running");
  } else if (activeStatusFilter.value === "stopped") {
    list = list.filter((c) => c.state.toLowerCase() !== "running");
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase();
    list = list.filter(
      (c) =>
        c.name.toLowerCase().includes(q) ||
        c.image.toLowerCase().includes(q) ||
        c.id.toLowerCase().includes(q) ||
        (c.compose_project && c.compose_project.toLowerCase().includes(q)),
    );
  }

  return list;
});

const groupedContainers = computed(() => {
  const groups: Record<string, ContainerInfo[]> = {};
  for (const c of filteredContainers.value) {
    const key = c.compose_project
      ? `Compose: ${c.compose_project}`
      : "Standalone Containers";
    if (!groups[key]) groups[key] = [];
    groups[key].push(c);
  }
  return groups;
});

const fetchContainers = async () => {
  if (!props.isActive) return;
  loading.value = true;
  error.value = null;

  try {
    const data = await getContainerList(props.profileId);
    containers.value = data;

    // Fetch stats in background for running containers
    if (data.some((c) => c.state.toLowerCase() === "running")) {
      getContainerStats(props.profileId)
        .then((stats) => {
          containerStats.value = stats;
        })
        .catch(() => {
          // Stats failure is non-fatal
        });
    }
  } catch (err: any) {
    error.value = err.message || String(err);
  } finally {
    loading.value = false;
  }
};

const handleAction = async (action: ContainerActionType, container: ContainerInfo) => {
  actionLoadingId.value = container.id;
  try {
    await executeContainerAction(props.profileId, action, container.id, container.engine);
    message.success(`Container ${container.name} ${action}ed`);
    await fetchContainers();
  } catch (err: any) {
    message.error(`Action ${action} failed: ` + (err.message || err));
  } finally {
    actionLoadingId.value = null;
  }
};

const handleRemove = async (container: ContainerInfo) => {
  const confirmed = await showConfirm(
    "Remove Container",
    `Are you sure you want to remove container "${container.name}" (${container.id.slice(0, 10)})?\nThis action cannot be undone.`,
  );

  if (!confirmed) return;

  actionLoadingId.value = container.id;
  try {
    await executeContainerAction(props.profileId, "rm", container.id, container.engine);
    message.success(`Container ${container.name} removed`);
    await fetchContainers();
  } catch (err: any) {
    message.error("Failed to remove container: " + (err.message || err));
  } finally {
    actionLoadingId.value = null;
  }
};

const openLogs = (container: ContainerInfo) => {
  activeLogsContainer.value = container;
  openOverlay("container-logs-modal");
};

watch(
  () => props.profileId,
  () => {
    fetchContainers();
  },
);

watch(
  () => props.isActive,
  (active) => {
    if (active) fetchContainers();
  },
);

onMounted(() => {
  fetchContainers();
});
</script>
