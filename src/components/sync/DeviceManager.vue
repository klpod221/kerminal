<template>
  <div class="space-y-4">
    <div v-if="isLoading" class="text-center py-8 text-gray-400">
      Loading devices...
    </div>

    <template v-else>
      <!-- Current Device Section -->
      <Card>
        <template #header>
          <div class="flex items-center justify-between">
            <h3 class="text-base font-semibold text-white flex items-center gap-2">
              <Computer class="w-5 h-5" />
              Current Device
            </h3>
            <Badge :variant="currentDevice ? 'success' : 'warning'">
              {{ currentDevice ? 'Registered' : 'Not Registered' }}
            </Badge>
          </div>
        </template>

        <template #content>
          <div v-if="currentDevice" class="flex items-start gap-4">
            <div class="text-4xl">{{ getDeviceIcon(currentDevice.deviceType) }}</div>
            <div class="flex-1 space-y-2">
              <div class="flex items-center gap-2">
                <h4 class="text-white font-medium">{{ currentDevice.deviceName }}</h4>
                <Badge variant="primary" size="sm">Current</Badge>
              </div>
              <div class="text-sm text-gray-400 space-y-1">
                <p>Device ID: {{ currentDevice.deviceId }}</p>
                <p>OS: {{ currentDevice.osInfo.osType }} {{ currentDevice.osInfo.osVersion }}</p>
                <p>Type: {{ currentDevice.deviceType }}</p>
              </div>
            </div>
          </div>

          <div v-else class="text-center py-2">
            <Button
              :disabled="isLoading"
              @click="registerCurrentDevice"
            >
              <PlusCircle class="w-4 h-4 mr-2" />
              Register This Device
            </Button>
          </div>
        </template>
      </Card>

      <!-- Other Devices Section -->
      <Card v-if="otherDevices.length > 0">
        <template #header>
          <div class="flex items-center justify-between">
            <h3 class="text-base font-semibold text-white">Other Devices</h3>
            <Button variant="ghost" size="sm" :disabled="isRefreshing" @click="loadDevices">
              <RefreshCw :class="['w-4 h-4', isRefreshing && 'animate-spin']" />
            </Button>
          </div>
        </template>

        <template #content>
          <div class="space-y-2">
            <div
              v-for="device in otherDevices"
              :key="device.deviceId"
              class="bg-gray-800 border border-gray-700 rounded-lg p-4"
            >
              <div class="flex items-start justify-between">
                <div class="flex items-start gap-3 flex-1">
                  <div class="text-2xl">{{ getDeviceIcon(device.deviceType) }}</div>
                  <div class="flex-1">
                    <div class="flex items-center gap-2 mb-1">
                      <h4 class="text-sm font-medium text-gray-200">{{ device.deviceName }}</h4>
                      <Badge v-if="device.lastSeen && isOnline(device.lastSeen)" variant="success">
                        Online
                      </Badge>
                      <Badge v-else variant="default">
                        Offline
                      </Badge>
                    </div>
                    <div class="text-xs text-gray-400 space-y-1">
                      <div>ID: <span class="font-mono">{{ device.deviceId }}</span></div>
                      <div>Type: {{ device.deviceType }}</div>
                      <div>OS: {{ device.osInfo.osType }} {{ device.osInfo.osVersion }}</div>
                      <div>Last Seen: {{ formatDate(device.lastSeen) }}</div>
                      <div>Registered: {{ formatDate(device.createdAt) }}</div>
                    </div>
                  </div>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  @click="showDeviceInfo(device)"
                >
                  Details
                </Button>
              </div>
            </div>
          </div>
        </template>
      </Card>

      <!-- Statistics Section -->
      <Card v-if="devices.length > 0">
        <template #header>
          <h3 class="text-base font-semibold text-white">Statistics</h3>
        </template>

        <template #content>
          <div class="grid grid-cols-3 gap-4">
            <div class="bg-gray-800 rounded-lg p-4 border border-gray-700 text-center">
              <div class="text-2xl font-semibold text-gray-100">{{ devices.length }}</div>
              <div class="text-xs text-gray-400 mt-1">Total Devices</div>
            </div>
            <div class="bg-gray-800 rounded-lg p-4 border border-gray-700 text-center">
              <div class="text-2xl font-semibold text-green-400">{{ onlineDevices }}</div>
              <div class="text-xs text-gray-400 mt-1">Online</div>
            </div>
            <div class="bg-gray-800 rounded-lg p-4 border border-gray-700 text-center">
              <div class="text-2xl font-semibold text-gray-400">{{ devices.length - onlineDevices }}</div>
              <div class="text-xs text-gray-400 mt-1">Offline</div>
            </div>
          </div>
        </template>
      </Card>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { Computer, PlusCircle, RefreshCw } from "lucide-vue-next";
import Card from "../ui/Card.vue";
import Badge from "../ui/Badge.vue";
import Button from "../ui/Button.vue";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import { syncService } from "../../services/sync";
import type { Device } from "../../types/sync";

const isLoading = ref(false);
const isRefreshing = ref(false);
const devices = ref<Device[]>([]);
const currentDevice = ref<Device | null>(null);

const otherDevices = computed(() => {
  if (!currentDevice.value) return devices.value;
  return devices.value.filter((d) => d.deviceId !== currentDevice.value?.deviceId);
});

const onlineDevices = computed(() => {
  return devices.value.filter((d) => d.lastSeen && isOnline(d.lastSeen)).length;
});

const getDeviceIcon = (type: string): string => {
  const icons: Record<string, string> = {
    Desktop: "🖥️",
    Laptop: "💻",
    Mobile: "📱",
    Tablet: "📱",
    Server: "🖧",
  };
  return icons[type] || "💻";
};

const isOnline = (lastSeenAt: string): boolean => {
  const lastSeen = new Date(lastSeenAt).getTime();
  const now = Date.now();
  const fiveMinutes = 5 * 60 * 1000;
  return (now - lastSeen) < fiveMinutes;
};

const formatDate = (dateStr?: string): string => {
  if (!dateStr) return "Never";

  const date = new Date(dateStr);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (seconds < 60) return "Just now";
  if (minutes < 60) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
  if (hours < 24) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
  if (days < 7) return `${days} day${days > 1 ? 's' : ''} ago`;

  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
};

const loadDevices = async () => {
  isRefreshing.value = true;
  try {
    devices.value = await syncService.getAllDevices();
  } catch (error) {
    console.error("Failed to load devices:", error);
    message.error(getErrorMessage(error, "Failed to load devices"));
  } finally {
    isRefreshing.value = false;
  }
};

const loadCurrentDevice = async () => {
  try {
    currentDevice.value = await syncService.getCurrentDevice();
  } catch (error) {
    console.error("Failed to load current device:", error);
  }
};

const registerCurrentDevice = async () => {
  isLoading.value = true;
  try {
    const device = await syncService.registerDevice({
      deviceName: "Current Device",
      deviceType: "Desktop",
    });
    currentDevice.value = device;
    await loadDevices();
    message.success("Device registered successfully");
  } catch (error) {
    console.error("Failed to register device:", error);
    message.error(getErrorMessage(error, "Failed to register device"));
  } finally {
    isLoading.value = false;
  }
};

const showDeviceInfo = (device: Device) => {
  const info = `
Device Information:
- Name: ${device.deviceName}
- ID: ${device.deviceId}
- Type: ${device.deviceType}
- OS: ${device.osInfo.osType} ${device.osInfo.osVersion}
- Last Seen: ${formatDate(device.lastSeen)}
- Registered: ${formatDate(device.createdAt)}
  `.trim();

  message.info(info);
};

onMounted(async () => {
  isLoading.value = true;
  try {
    await Promise.all([
      loadCurrentDevice(),
      loadDevices(),
    ]);
  } finally {
    isLoading.value = false;
  }
});
</script>
