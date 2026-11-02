<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <Modal
    id="container-logs-modal"
    :title="`Container Logs: ${container?.name || container?.id || ''}`"
    :icon="FileText"
    size="2xl"
    :show-close-button="true"
  >
    <div class="space-y-3">
      <!-- Toolbar -->
      <div class="flex flex-wrap items-center justify-between gap-2 pb-2 border-b border-gray-800">
        <div class="flex items-center gap-2 flex-1 max-w-xs">
          <Input
            id="logs-search"
            v-model="searchQuery"
            placeholder="Search logs..."
            :left-icon="Search"
            size="sm"
            :helper="false"
          />
        </div>

        <div class="flex items-center gap-2">
          <Select
            id="tail-select"
            v-model="tailLines"
            :options="[
              { value: '100', label: 'Last 100 lines' },
              { value: '200', label: 'Last 200 lines' },
              { value: '500', label: 'Last 500 lines' },
              { value: '1000', label: 'Last 1000 lines' },
            ]"
            size="sm"
            class="w-36"
            @change="fetchLogs"
          />
          <Button
            variant="secondary"
            size="sm"
            :icon="RefreshCw"
            :loading="loading"
            title="Refresh logs"
            @click="fetchLogs"
          />
          <Button
            variant="secondary"
            size="sm"
            :icon="Copy"
            title="Copy logs to clipboard"
            @click="copyLogs"
          />
        </div>
      </div>

      <!-- Log Output Box -->
      <div class="relative bg-black/70 rounded-lg border border-gray-800 font-mono text-xs overflow-hidden">
        <div
          v-if="loading && !logs"
          class="flex items-center justify-center p-12 text-gray-400 gap-2"
        >
          <RefreshCw class="w-4 h-4 animate-spin text-blue-400" />
          <span>Fetching container logs...</span>
        </div>

        <div
          v-else-if="filteredLines.length === 0"
          class="p-8 text-center text-gray-500"
        >
          {{ searchQuery ? "No log lines matching search query." : "No logs available for this container." }}
        </div>

        <div
          v-else
          ref="logContainerRef"
          class="p-3 max-h-[460px] overflow-y-auto space-y-0.5 leading-relaxed selection:bg-blue-600/40"
        >
          <div
            v-for="(line, idx) in filteredLines"
            :key="idx"
            class="hover:bg-white/5 px-1 rounded flex gap-3 text-gray-300 whitespace-pre-wrap break-all"
          >
            <span class="text-gray-600 select-none text-[10px] w-8 text-right shrink-0">
              {{ idx + 1 }}
            </span>
            <span>{{ line }}</span>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-between items-center w-full text-xs text-gray-400">
        <span>Image: <code class="text-blue-400">{{ container?.image }}</code></span>
        <Button variant="ghost" size="sm" text="Close" @click="closeLogs" />
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import { FileText, Search, RefreshCw, Copy } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Input from "../ui/Input.vue";
import Select from "../ui/Select.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import { getContainerLogs, type ContainerInfo } from "../../services/docker";
import { message } from "../../utils/message";

const props = defineProps<{
  container: ContainerInfo | null;
  profileId?: string;
}>();

const { closeOverlay } = useOverlay();

const logs = ref("");
const loading = ref(false);
const searchQuery = ref("");
const tailLines = ref("200");
const logContainerRef = ref<HTMLDivElement | null>(null);

const filteredLines = computed(() => {
  if (!logs.value) return [];
  const lines = logs.value.split("\n");
  if (!searchQuery.value.trim()) return lines;
  const q = searchQuery.value.toLowerCase();
  return lines.filter((line) => line.toLowerCase().includes(q));
});

const fetchLogs = async () => {
  if (!props.container) return;
  loading.value = true;
  try {
    const raw = await getContainerLogs(
      props.profileId,
      props.container.id,
      parseInt(tailLines.value, 10) || 200,
      props.container.engine,
    );
    logs.value = raw.trim();
    await nextTick();
    if (logContainerRef.value) {
      logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight;
    }
  } catch (err: any) {
    message.error("Failed to load logs: " + (err.message || err));
  } finally {
    loading.value = false;
  }
};

const copyLogs = async () => {
  if (!logs.value) return;
  try {
    await navigator.clipboard.writeText(logs.value);
    message.success("Logs copied to clipboard!");
  } catch {
    message.error("Could not copy logs to clipboard.");
  }
};

const closeLogs = () => {
  closeOverlay("container-logs-modal");
};

watch(
  () => props.container,
  (c) => {
    if (c) {
      logs.value = "";
      searchQuery.value = "";
      fetchLogs();
    }
  },
  { immediate: true },
);
</script>
