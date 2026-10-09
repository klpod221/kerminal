<template>
  <div class="p-3">
    <div class="flex justify-between items-center mb-3">
      <h2 class="text-sm font-semibold flex items-center gap-1.5 bg-linear-to-r from-emerald-400 to-cyan-400 bg-clip-text text-transparent">
        <ActivityIcon class="w-4 h-4 text-emerald-400" />
        System Status
      </h2>
      <div class="flex items-center gap-1.5 cursor-pointer hover:opacity-80 transition-opacity" @click="monitoring = !monitoring" title="Click to toggle live polling">
        <span class="relative flex h-1.5 w-1.5">
          <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" v-if="monitoring"></span>
          <span class="relative inline-flex rounded-full h-1.5 w-1.5" :class="monitoring ? 'bg-emerald-500' : 'bg-gray-500'"></span>
        </span>
        <span class="text-[9px] text-gray-400 font-medium tracking-wider uppercase">
          {{ monitoring ? 'Live' : 'Paused' }}
        </span>
      </div>
    </div>

    <!-- Metrics Stack -->
    <div class="flex flex-col gap-2">
      <!-- CPU -->
      <div class="relative overflow-hidden bg-white/2 border border-white/5 p-2.5 rounded-lg hover:bg-white/4 transition-all duration-300 group shadow-sm">
        <div class="absolute top-0 left-0 w-full h-0.5 bg-gray-800/50">
          <div class="h-full bg-linear-to-r from-blue-500 to-cyan-400 transition-all duration-1000 ease-out" :style="{ width: monitoring ? `${metrics.cpu}%` : '0%' }"></div>
        </div>
        <div class="flex justify-between items-start mt-1">
          <div class="flex items-center gap-2">
            <div class="p-1.5 bg-blue-500/10 rounded-md group-hover:scale-110 transition-transform">
              <CpuIcon class="w-3.5 h-3.5 text-blue-400" />
            </div>
            <div>
              <div class="text-[10px] text-gray-400 font-medium mb-0.5 pr-2 leading-tight" :title="metrics.cpuName">{{ metrics.cpuName }}</div>
              <div class="text-xl font-light text-white leading-none">
                {{ monitoring ? metrics.cpu : 'N/A' }}<span v-if="monitoring" class="text-xs font-medium text-gray-500 ml-0.5">%</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- RAM -->
      <div class="relative overflow-hidden bg-white/2 border border-white/5 p-2.5 rounded-lg hover:bg-white/4 transition-all duration-300 group shadow-sm">
        <div class="absolute top-0 left-0 w-full h-0.5 bg-gray-800/50">
          <div class="h-full bg-linear-to-r from-purple-500 to-pink-500 transition-all duration-1000 ease-out" :style="{ width: monitoring ? `${metrics.ram}%` : '0%' }"></div>
        </div>
        <div class="flex justify-between items-start mt-1">
          <div class="flex items-center gap-2">
            <div class="p-1.5 bg-purple-500/10 rounded-md group-hover:scale-110 transition-transform">
              <MemoryStickIcon class="w-3.5 h-3.5 text-purple-400" />
            </div>
            <div>
              <div class="text-[10px] text-gray-400 font-semibold uppercase tracking-wider mb-0.5">Memory</div>
              <div class="text-xl font-light text-white leading-none">
                {{ monitoring ? metrics.ram : 'N/A' }}<span v-if="monitoring" class="text-xs font-medium text-gray-500 ml-0.5">%</span>
              </div>
            </div>
          </div>
          <div class="text-right mt-1">
            <div class="text-xs text-white font-medium">
              {{ monitoring ? metrics.ramUsed : 'N/A' }}<span v-if="monitoring" class="text-gray-500 text-[10px]">GB</span>
            </div>
            <div class="text-[9px] text-gray-500 mt-0.5 border-t border-gray-700/50 pt-0.5">
              {{ monitoring ? metrics.ramTotal : 'N/A' }}<span v-if="monitoring">GB Total</span>
            </div>
          </div>
        </div>
      </div>

      <!-- GPU -->
      <div class="relative overflow-hidden bg-white/2 border border-white/5 p-2.5 rounded-lg hover:bg-white/4 transition-all duration-300 group shadow-sm">
        <div class="absolute top-0 left-0 w-full h-0.5 bg-gray-800/50" v-if="metrics.hasGpu">
          <div class="h-full bg-linear-to-r from-emerald-500 to-green-400 transition-all duration-1000 ease-out" :style="{ width: monitoring ? `${metrics.gpu}%` : '0%' }"></div>
        </div>
        <div class="flex justify-between items-start mt-1">
          <div class="flex items-center gap-2">
            <div class="p-1.5 bg-emerald-500/10 rounded-md group-hover:scale-110 transition-transform">
              <ZapIcon class="w-3.5 h-3.5 text-emerald-400" />
            </div>
            <div>
              <div class="text-[10px] text-gray-400 font-medium mb-0.5 pr-2 leading-tight" :title="metrics.gpuName">{{ metrics.gpuName }}</div>
              <div class="text-xl font-light text-white leading-none" v-if="metrics.hasGpu">
                {{ monitoring ? metrics.gpu : 'N/A' }}<span v-if="monitoring" class="text-xs font-medium text-gray-500 ml-0.5">%</span>
              </div>
              <div class="text-xs text-gray-500 mt-0.5" v-else>N/A</div>
            </div>
          </div>
          <div class="text-right mt-1" v-if="metrics.hasGpu">
            <div class="text-xs text-white font-medium">
              {{ monitoring ? metrics.vramUsed : 'N/A' }}<span v-if="monitoring" class="text-gray-500 text-[10px]">GB</span>
            </div>
            <div class="text-[9px] text-gray-500 mt-0.5 border-t border-gray-700/50 pt-0.5">
              {{ monitoring ? metrics.vramTotal : 'N/A' }}<span v-if="monitoring">GB VRAM</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Disk -->
      <div class="relative overflow-hidden bg-white/2 border border-white/5 p-2.5 rounded-lg hover:bg-white/4 transition-all duration-300 group shadow-sm">
        <div class="absolute top-0 left-0 w-full h-0.5 bg-gray-800/50">
          <div class="h-full bg-linear-to-r from-orange-500 to-amber-400 transition-all duration-1000 ease-out" :style="{ width: monitoring ? `${metrics.disk}%` : '0%' }"></div>
        </div>
        <div class="flex justify-between items-start mt-1">
          <div class="flex items-center gap-2">
            <div class="p-1.5 bg-orange-500/10 rounded-md group-hover:scale-110 transition-transform">
              <HardDriveIcon class="w-3.5 h-3.5 text-orange-400" />
            </div>
            <div>
              <div class="text-[10px] text-gray-400 font-semibold uppercase tracking-wider mb-0.5">Root Disk (/)</div>
              <div class="text-xl font-light text-white leading-none">
                {{ monitoring ? metrics.disk : 'N/A' }}<span v-if="monitoring" class="text-xs font-medium text-gray-500 ml-0.5">%</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { ActivityIcon, CpuIcon, MemoryStickIcon, ZapIcon, HardDriveIcon } from 'lucide-vue-next';
import { getRemoteServerMetrics } from '../../services/monitor';

const props = defineProps<{
  profileId: string;
  isActive: boolean;
}>();

const monitoring = ref(true);
let pollInterval: any = null;

const metrics = ref({
  cpu: 0,
  cpuName: 'CPU Usage',
  ram: 0,
  ramUsed: '0',
  ramTotal: '0',
  hasGpu: true,
  gpu: 0,
  gpuName: 'GPU Compute',
  vramUsed: '0',
  vramTotal: '0',
  disk: 0
});

const fetchMetrics = async () => {
  if (!monitoring.value || !props.profileId || !props.isActive) return;
  try {
    const data: any = await getRemoteServerMetrics(props.profileId);
    metrics.value = {
      cpu: data.cpu,
      cpuName: data.cpu_name || 'CPU Usage',
      ram: data.ram,
      ramUsed: data.ram_used,
      ramTotal: data.ram_total,
      hasGpu: data.has_gpu,
      gpu: data.gpu,
      gpuName: data.gpu_name || 'GPU Compute',
      vramUsed: data.vram_used,
      vramTotal: data.vram_total,
      disk: data.disk
    };
  } catch (error) {
    console.error("Failed to fetch remote metrics:", error);
  }
};

watch(() => props.profileId, () => {
  metrics.value = {
    cpu: 0, cpuName: 'CPU Usage', ram: 0, ramUsed: '0', ramTotal: '0', hasGpu: true, gpu: 0, gpuName: 'GPU Compute', vramUsed: '0', vramTotal: '0', disk: 0
  };
  fetchMetrics();
});

onMounted(() => {
  fetchMetrics();
  pollInterval = setInterval(fetchMetrics, 2000); // Poll every 2 seconds
});

onUnmounted(() => {
  if (pollInterval) clearInterval(pollInterval);
});
</script>
