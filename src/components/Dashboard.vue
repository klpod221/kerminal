<template>
  <div
    class="h-full bg-bg-secondary text-white overflow-y-auto dashboard-container"
    :class="isMobile ? 'p-3' : 'p-4'"
  >
    <RenderOptimizationLayer
      v-if="isOptimizingDisplay"
      :status="integrityStatus"
      @close="isOptimizingDisplay = false"
    />
    <div class="container mx-auto animate-fade-in">
      <div :class="isMobile ? 'mt-2 mb-8' : 'mt-4 mb-12'">
        <img
          src="../assets/images/logo_500.png"
          alt="Kerminal Logo"
          :class="isMobile ? 'w-20 h-20' : 'w-32 h-32'"
          class="mx-auto mb-4 cursor-pointer"
          @click="handleForceRefresh"
          title="Click to refresh system status"
        />
        <h1
          class="text-center font-extrabold bg-linear-to-br from-cyan-500 to-purple-500 bg-clip-text text-transparent"
          :class="isMobile ? 'text-3xl' : 'text-5xl md:text-6xl'"
        >
          Kerminal
        </h1>
        <p
          class="text-center text-gray-400"
          :class="isMobile ? 'text-xs mb-1' : 'text-sm mb-2'"
        >
          v{{ appVersion }}
        </p>
        <p class="text-center mt-1" :class="isMobile ? 'text-base' : 'text-xl'">
          Modern Terminal Emulator & SSH Manager
        </p>
        <p
          class="text-center text-gray-400 mt-1"
          :class="isMobile ? 'text-xs' : ''"
        >
          Made with ❤️ by
          <a
            href="https://klpod221.com"
            target="_blank"
            rel="noopener noreferrer"
            class="underline hover:text-white text-sm transition-colors duration-200 touch-manipulation"
          >
            klpod221
          </a>
        </p>
      </div>

      <div
        class="flex flex-col justify-between items-center border-t border-b border-gray-800"
        :class="isMobile ? 'py-3' : 'py-4'"
      >
        <h2
          class="font-semibold"
          :class="isMobile ? 'text-xl mb-4' : 'text-2xl mb-6'"
        >
          System Information
        </h2>

        <div
          class="grid gap-4 w-full"
          :class="
            isMobile
              ? 'grid-cols-1'
              : 'grid-cols-1 sm:grid-cols-2 lg:grid-cols-3'
          "
        >
          <Card
            title="Computer"
            :icon="Computer"
            icon-background="bg-blue-500/20"
            icon-color="text-blue-400"
            :hover="true"
          >
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">OS:</span>
              <span
                class="text-white text-sm bg-gray-800 px-2 py-1 rounded truncate"
              >
                {{ systemInfo.os_version }}
              </span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Kernel:</span>
              <span
                class="text-white text-sm bg-gray-800 px-2 py-1 rounded truncate"
              >
                {{ systemInfo.platform + " " + systemInfo.cpu_arch }}
              </span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Hostname:</span>
              <span
                class="text-white text-sm bg-gray-800 px-2 py-1 rounded truncate"
              >
                {{ systemInfo.hostname }}
              </span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Uptime:</span>
              <span class="text-white text-sm bg-gray-800 px-2 py-1 rounded">
                {{ formatUptime(systemInfo.uptime) }}
              </span>
            </div>
          </Card>

          <Card
            title="CPU"
            :icon="Cpu"
            icon-background="bg-green-500/20"
            icon-color="text-green-400"
            :hover="true"
          >
            <div
              v-if="systemInfo.cpu_info"
              class="flex justify-between items-center"
            >
              <span class="text-gray-400 font-medium mr-1">Cores:</span>
              <span class="text-white text-sm bg-gray-800 px-2 py-1 rounded">
                {{ systemInfo.cpu_info }}
              </span>
            </div>
            <div
              v-if="systemInfo.cpus && systemInfo.cpus.length > 0"
              class="flex justify-between items-center"
            >
              <span class="text-gray-400 font-medium mr-1">Model:</span>
              <span
                class="text-white text-sm bg-gray-800 px-2 py-1 rounded truncate"
                :title="systemInfo.cpus[0].model"
              >
                {{ truncateText(systemInfo.cpus[0].model, 20) }}
              </span>
            </div>
            <div
              v-if="systemInfo.cpus && systemInfo.cpus.length > 0"
              class="flex justify-between items-center"
            >
              <span class="text-gray-400 font-medium mr-1">Speed:</span>
              <span class="text-white text-sm bg-gray-800 px-2 py-1 rounded">
                {{ formatFrequency(systemInfo.cpus[0].speed) }}
              </span>
            </div>
            <div
              v-if="systemInfo.cpus && systemInfo.cpus.length > 0"
              class="flex justify-between items-center"
            >
              <span class="text-gray-400 font-medium mr-1">Usage:</span>
              <span class="text-white text-sm bg-gray-800 px-2 py-1 rounded">
                {{
                  formatPercentage(
                    systemInfo.cpus.reduce((acc, cpu) => acc + cpu.usage, 0) /
                      systemInfo.cpus.length,
                  )
                }}
              </span>
            </div>
          </Card>

          <Card
            title="Memory"
            :icon="MemoryStick"
            icon-background="bg-purple-500/20"
            icon-color="text-purple-400"
            :hover="true"
          >
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Total:</span>
              <span class="text-white text-sm bg-gray-800 px-2 py-1 rounded">
                {{ formatBytes(systemInfo.total_memory) }}
              </span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Used:</span>
              <span class="text-white text-sm bg-gray-800 px-2 py-1 rounded">
                {{
                  formatBytes(systemInfo.total_memory - systemInfo.free_memory)
                }}
              </span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Available:</span>
              <span class="text-white text-sm bg-gray-800 px-2 py-1 rounded">
                {{ formatBytes(systemInfo.free_memory) }}
              </span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Load Avg:</span>
              <span class="text-white text-sm bg-gray-800 px-2 py-1 rounded">
                {{
                  `${systemInfo.load_average?.[0]?.toFixed(2) || "0.00"} / ${
                    systemInfo.load_average?.[1]?.toFixed(2) || "0.00"
                  } / ${systemInfo.load_average?.[2]?.toFixed(2) || "0.00"}`
                }}
              </span>
            </div>
          </Card>

          <Card
            v-if="systemInfo.disks_info && systemInfo.disks_info.length > 0"
            title="Storage"
            :icon="HardDrive"
            icon-background="bg-orange-500/20"
            icon-color="text-orange-400"
            :hover="true"
          >
            <div
              v-for="disk in systemInfo.disks_info.slice(0, 3)"
              :key="disk.mount_point"
              class="space-y-2"
            >
              <div class="flex justify-between items-start">
                <div class="flex-1 min-w-0">
                  <div class="text-white text-sm font-medium truncate">
                    {{ disk.mount_point }}
                  </div>
                  <div class="text-gray-400 text-xs">
                    {{ disk.file_system }}
                  </div>
                </div>
                <div class="text-right ml-2">
                  <div class="text-white text-sm">
                    {{ formatBytes(disk.available_space) }}
                  </div>
                  <div class="text-gray-400 text-xs">
                    / {{ formatBytes(disk.total_space) }}
                  </div>
                </div>
              </div>
              <div
                v-if="
                  disk !==
                  systemInfo.disks_info[systemInfo.disks_info.length - 1]
                "
                class="border-b border-gray-700"
              ></div>
            </div>
          </Card>

          <Card
            v-if="
              systemInfo.network_interfaces &&
              systemInfo.network_interfaces.length > 0
            "
            title="Network"
            :icon="Network"
            icon-background="bg-cyan-500/20"
            icon-color="text-cyan-400"
            :hover="true"
          >
            <div
              v-for="interface_ in systemInfo.network_interfaces
                .filter((i) => i.status === 'up')
                .slice(0, 3)"
              :key="interface_.name"
              class="space-y-2"
            >
              <div class="flex justify-between items-start">
                <div class="flex-1 min-w-0">
                  <div class="text-white text-sm font-medium">
                    {{ interface_.name }}
                  </div>
                  <div class="text-gray-400 text-xs truncate">
                    {{ interface_.address }}
                  </div>
                </div>
                <div class="text-right ml-2">
                  <div
                    class="text-xs px-2 py-1 rounded"
                    :class="
                      interface_.status === 'up'
                        ? 'bg-green-500/20 text-green-400'
                        : 'bg-red-500/20 text-red-400'
                    "
                  >
                    {{ interface_.status }}
                  </div>
                </div>
              </div>
              <div
                v-if="
                  interface_ !==
                  systemInfo.network_interfaces
                    .filter((i) => i.status === 'up')
                    .slice(0, 3)[
                    systemInfo.network_interfaces
                      .filter((i) => i.status === 'up')
                      .slice(0, 3).length - 1
                  ]
                "
                class="border-b border-gray-700"
              ></div>
            </div>
          </Card>

          <Card
            v-if="
              systemInfo.components_info &&
              systemInfo.components_info.length > 0
            "
            title="Temperature"
            :icon="Thermometer"
            icon-background="bg-red-500/20"
            icon-color="text-red-400"
            :hover="true"
          >
            <div
              v-for="component in systemInfo.components_info
                .filter((c) => c.temperature > 0)
                .slice(0, 4)"
              :key="component.label"
              class="flex justify-between items-center"
            >
              <span class="text-gray-400 font-medium mr-1 text-sm truncate"
                >{{ component.label }}:</span
              >
              <span class="text-white text-sm bg-gray-800 px-2 py-1 rounded">
                {{ formatTemperature(component.temperature) }}
              </span>
            </div>
          </Card>
        </div>

        <!-- Auto refresh indicator - Bottom -->
        <div class="text-center mt-4 space-y-2">
          <div class="flex justify-center items-center space-x-2">
            <div
              v-if="loading"
              class="animate-spin rounded-full h-4 w-4 border-b-2 border-accent-blue"
            ></div>
            <span class="text-xs text-gray-500">
              {{ loading ? "Updating..." : "Auto-refresh every 5s" }}
            </span>
          </div>
          <div v-if="lastUpdated" class="text-xs text-gray-600">
            Last updated: {{ formatLastUpdated(lastUpdated) }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import Card from "./ui/Card.vue";
import { truncateText } from "../utils/helpers";
import {
  formatBytes,
  formatUptime,
  formatTemperature,
  formatPercentage,
  formatFrequency,
} from "../utils/formatter";
import {
  Computer,
  Cpu,
  MemoryStick,
  HardDrive,
  Network,
  Thermometer,
} from "lucide-vue-next";

import { useWindowSize } from "../composables/useWindowSize";
import { useSettingsStore } from "../stores/settings";
import {
  verifySystemIntegrity,
  type SystemIntegrityStatus,
} from "../services/dashboard";
import RenderOptimizationLayer from "./ui/RenderOptimizationLayer.vue";

const { isMobile } = useWindowSize();

declare const __APP_VERSION__: string;
const appVersion = __APP_VERSION__;

// Debug/Maintenance: Track attempts to force refresh the dashboard

const refreshAttempts = ref(0);

const isOptimizingDisplay = ref(false);

const isLoadingIntegrity = ref(false);

const integrityStatus = ref<SystemIntegrityStatus | null>(null);

const handleForceRefresh = async () => {
  if (isOptimizingDisplay.value || isLoadingIntegrity.value) return;

  refreshAttempts.value++;

  // If user clicks repeatedly (10 times), assume cache is stuck and run deep verification

  if (refreshAttempts.value >= 10) {
    refreshAttempts.value = 0; // Reset immediately to prevent multiple triggers

    isLoadingIntegrity.value = true;

    try {
      const status = await verifySystemIntegrity();

      integrityStatus.value = status;

      isOptimizingDisplay.value = true;
    } catch (error) {
      console.error("Failed to verify system integrity:", error);
    } finally {
      isLoadingIntegrity.value = false;
    }
  }
};

interface CPUInfo {
  model: string;
  speed: number; // in MHz
  usage: number; // in percentage
}

interface DiskInfo {
  name: string;
  total_space: number; // in bytes
  available_space: number; // in bytes
  file_system: string;
  mount_point: string;
}

interface ComponentInfo {
  label: string;
  temperature: number; // in Celsius
  max: number; // in Celsius
}

interface NetworkInterface {
  name: string;
  address: string;
  mac: string;
  status: string;
}

interface SystemInfo {
  platform: string;
  release: string;
  cpu_arch: string;
  hostname: string;
  uptime: number; // in seconds
  total_memory: number; // in bytes
  free_memory: number; // in bytes
  load_average: [number, number, number]; // 1, 5, 15 minutes
  os_version?: string;
  cpu_info?: string;
  memory_info?: string;
  gpu_info?: string;
  resolution?: [number, number];
  cpus?: CPUInfo[];
  disks_info?: DiskInfo[];
  components_info?: ComponentInfo[];
  network_interfaces?: NetworkInterface[];
}

const loading = ref(false);
const systemInfo = ref<SystemInfo>({} as SystemInfo);
const lastUpdated = ref<Date | null>(null);

const REFRESH_INTERVAL = 5000;
let refreshInterval: number | null = null;

const fetchSystemInfo = async () => {
  loading.value = true;

  try {
    const settingsStore = useSettingsStore();
    const info = await settingsStore.getSystemInfo();
    systemInfo.value = info as SystemInfo;
    lastUpdated.value = new Date();
  } catch (error) {
    console.error("Error fetching system info:", error);
  } finally {
    loading.value = false;
  }
};

const formatLastUpdated = (date: Date) => {
  return date.toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
};

const startAutoRefresh = () => {
  if (refreshInterval) {
    clearInterval(refreshInterval);
  }

  refreshInterval = setInterval(fetchSystemInfo, REFRESH_INTERVAL);
};

const stopAutoRefresh = () => {
  if (refreshInterval) {
    clearInterval(refreshInterval);
    refreshInterval = null;
  }
};

onMounted(() => {
  fetchSystemInfo();
  startAutoRefresh();
});

onUnmounted(() => {
  stopAutoRefresh();
});
</script>
