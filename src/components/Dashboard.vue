<template>
  <div
    class="h-full bg-[#171717] text-white p-4 overflow-y-auto dashboard-container"
  >
    <div class="container mx-auto animate-fade-in">
      <div class="mt-4 mb-12">
        <img
          src="../assets/images/logo_500.png"
          alt="Kerminal Logo"
          class="w-32 h-32 mx-auto mb-4"
        />
        <h1
          class="text-center text-5xl md:text-6xl font-extrabold bg-gradient-to-r from-[#74c7ec] to-[#facc15] bg-clip-text text-transparent"
        >
          Kerminal
        </h1>
        <p class="text-center text-sm text-gray-400 mb-2">v{{ appVersion }}</p>
        <p class="text-center text-xl mt-1">
          Modern Terminal with SSH, Tunneling & Cross-Device Sync
        </p>
        <p class="text-center text-gray-400 mt-1">
          Made with ❤️ by
          <a
            href="https://klpod221.com"
            target="_blank"
            rel="noopener noreferrer"
            class="underline hover:text-white text-sm transition-colors duration-200"
          >
            klpod221
          </a>
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <Card
            title="Computer"
            :icon="Computer"
            icon-background="bg-blue-500/20"
            icon-color="text-blue-400"
            :hover="true"
          >
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">OS:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate">{{
                systemInfo.os_version
              }}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Kernel:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate">{{
                systemInfo.platform + ' ' + systemInfo.cpu_arch
              }}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Hostname:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate">{{
                systemInfo.hostname
              }}</span>
            </div>
          </Card>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import Card from './ui/Card.vue'
// import { formatRelativeTime } from '../utils/formatter'
import { Computer } from 'lucide-vue-next'

import { getSystemInfo } from "../services/dashboard";

declare const __APP_VERSION__: string;
const appVersion = __APP_VERSION__;

// interface definitions
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
  addresses: string;
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
  cpus?: CPUInfo[];
  disks?: DiskInfo[];
  components?: ComponentInfo[];
  networks?: NetworkInterface[];
}

// State
const loading = ref(false);
const systemInfo = ref<SystemInfo>({} as SystemInfo);

// Fetch system info
const fetchSystemInfo = async () => {
  loading.value = true;

  try {
    const info = await getSystemInfo();
    systemInfo.value = info as SystemInfo;
  } catch (error) {
    console.error("Error fetching system info:", error);
  } finally {
    loading.value = false;
  }
};

// Initial fetch
fetchSystemInfo();
</script>
