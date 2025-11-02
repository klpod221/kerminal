<template>
  <div class="space-y-4">
    <SkeletonList
      v-if="isLoading"
      :items="4"
      :show-avatar="false"
      :show-actions="true"
    />

    <template v-else>
      <!-- Current Device Section -->
      <Card>
        <template #header>
          <div class="flex items-center justify-between">
            <h3
              class="text-base font-semibold text-white flex items-center gap-2"
            >
              <Computer class="w-5 h-5" />
              Current Device
            </h3>
            <Badge :variant="currentDevice ? 'success' : 'warning'">
              {{ currentDevice ? "Registered" : "Not Registered" }}
            </Badge>
          </div>
        </template>

        <div v-if="currentDevice" class="flex items-start gap-4">
          <div class="text-4xl">
            {{ getDeviceIcon(currentDevice.deviceType) }}
          </div>
          <div class="flex-1 space-y-2">
            <div class="flex items-center gap-2">
              <h4 class="text-white font-medium">
                {{ currentDevice.name }}
              </h4>
              <Badge variant="primary" size="sm">Current</Badge>
            </div>
            <div class="text-sm text-gray-400 space-y-1">
              <p>Device ID: {{ currentDevice.id }}</p>
              <p>Type: {{ currentDevice.deviceType }}</p>
            </div>
          </div>
        </div>

        <div v-else class="text-center py-2">
          <Button :disabled="isLoading" @click="registerCurrentDevice">
            <PlusCircle class="w-4 h-4 mr-2" />
            Register This Device
          </Button>
        </div>
      </Card>

      <!-- Other Devices Section -->
      <Card v-if="otherDevices.length > 0">
        <template #header>
          <div class="flex items-center justify-between">
            <h3 class="text-base font-semibold text-white">Other Devices</h3>
            <Button
              variant="ghost"
              size="sm"
              :disabled="isRefreshing"
              @click="loadDevices"
            >
              <RefreshCw :class="['w-4 h-4', isRefreshing && 'animate-spin']" />
            </Button>
          </div>
        </template>

        <div class="space-y-2">
          <div
            v-for="device in otherDevices"
            :key="device.id"
            class="bg-gray-800 border border-gray-700 rounded-lg p-4"
          >
            <div class="flex items-start justify-between">
              <div class="flex items-start gap-3 flex-1">
                <div class="text-2xl">
                  {{ getDeviceIcon(device.deviceType) }}
                </div>
                <div class="flex-1">
                  <div class="flex items-center gap-2 mb-1">
                    <h4 class="text-sm font-medium text-gray-200">
                      {{ device.name }}
                    </h4>
                    <Badge
                      v-if="
                        device.lastSeenAt && isRecentlyActive(device.lastSeenAt)
                      "
                      variant="success"
                    >
                      Online
                    </Badge>
                    <Badge v-else variant="default"> Offline </Badge>
                  </div>
                  <div class="text-xs text-gray-400 space-y-1">
                    <div>
                      ID: <span class="font-mono">{{ device.id }}</span>
                    </div>
                    <div>Type: {{ device.deviceType }}</div>
                    <div>
                      Last Seen: {{ formatDateOrNever(device.lastSeenAt) }}
                    </div>
                    <div>
                      Registered: {{ formatDateOrNever(device.registeredAt) }}
                    </div>
                  </div>
                </div>
              </div>
              <Button variant="ghost" size="sm" @click="showDeviceInfo(device)">
                Details
              </Button>
            </div>
          </div>
        </div>
      </Card>

      <!-- Statistics Section -->
      <Card v-if="devices.length > 0">
        <template #header>
          <h3 class="text-base font-semibold text-white">Statistics</h3>
        </template>

        <div class="grid grid-cols-3 gap-4">
          <div
            class="bg-gray-800 rounded-lg p-4 border border-gray-700 text-center"
          >
            <div class="text-2xl font-semibold text-gray-100">
              {{ devices.length }}
            </div>
            <div class="text-xs text-gray-400 mt-1">Total Devices</div>
          </div>
          <div
            class="bg-gray-800 rounded-lg p-4 border border-gray-700 text-center"
          >
            <div class="text-2xl font-semibold text-green-400">
              {{ onlineDevicesCount }}
            </div>
            <div class="text-xs text-gray-400 mt-1">Online</div>
          </div>
          <div
            class="bg-gray-800 rounded-lg p-4 border border-gray-700 text-center"
          >
            <div class="text-2xl font-semibold text-gray-400">
              {{ devices.length - onlineDevicesCount }}
            </div>
            <div class="text-xs text-gray-400 mt-1">Offline</div>
          </div>
        </div>
      </Card>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { Computer, PlusCircle, RefreshCw } from "lucide-vue-next";
import Card from "../ui/Card.vue";
import SkeletonList from "../ui/SkeletonList.vue";
import Badge from "../ui/Badge.vue";
import Button from "../ui/Button.vue";
import { message } from "../../utils/message";
import { isRecentlyActive } from "../../utils/helpers";
import { formatDateOrNever } from "../../utils/formatter";
import { useSyncStore } from "../../stores/sync";

const syncStore = useSyncStore();
const isLoading = ref(false);
const isRefreshing = ref(false);

type StoreDevice = {
  id: string;
  name: string;
  deviceType: string;
  registeredAt: string;
  lastSeenAt: string;
};

const devices = ref<StoreDevice[]>([]);
const currentDevice = ref<StoreDevice | null>(null);

const otherDevices = computed(() => {
  if (!currentDevice.value) return devices.value;
  return devices.value.filter((d) => d.id !== currentDevice.value?.id);
});

const onlineDevicesCount = computed(() => {
  return devices.value.filter(
    (d) => d.lastSeenAt && isRecentlyActive(d.lastSeenAt),
  ).length;
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

/**
 * Load all devices
 */
const loadDevices = async () => {
  isRefreshing.value = true;
  const allDevices = await syncStore.getAllDevices();
  devices.value = allDevices;
  isRefreshing.value = false;
};

/**
 * Load current device
 */
const loadCurrentDevice = async () => {
  const device = await syncStore.getCurrentDevice();
  currentDevice.value = device;
};

/**
 * Register current device
 */
const registerCurrentDevice = async () => {
  isLoading.value = true;
  const device = await syncStore.registerDevice("Current Device", "Desktop");
  currentDevice.value = device;
  await loadDevices();
  message.success("Device registered successfully");
  isLoading.value = false;
};

const showDeviceInfo = (device: StoreDevice) => {
  const info = `
Device Information:
- Name: ${device.name}
- ID: ${device.id}
- Type: ${device.deviceType}
- Last Seen: ${formatDateOrNever(device.lastSeenAt)}
- Registered: ${formatDateOrNever(device.registeredAt)}
  `.trim();

  message.info(info);
};

onMounted(async () => {
  isLoading.value = true;
  await Promise.all([loadCurrentDevice(), loadDevices()]);
  isLoading.value = false;
});
</script>
