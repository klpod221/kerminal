<template>
  <div class="space-y-4">
    <!-- Enable Proxy -->
    <Checkbox
      :model-value="enabled"
      label="Use Proxy"
      :helper="false"
      @update:model-value="handleToggleProxy"
    />

    <!-- Proxy Configuration -->
    <div v-if="enabled" class="space-y-4 pl-6 border-l-2 border-gray-600">
      <!-- Proxy Type -->
      <Select
        v-model="proxyData.proxy_type"
        label="Proxy Type"
        @change="handleTypeChange"
      >
        <option value="http">HTTP Proxy</option>
        <option value="socks4">SOCKS4</option>
        <option value="socks5">SOCKS5</option>
        <option value="jump">Jump Host</option>
      </Select>

      <!-- Basic Proxy Settings (for HTTP/SOCKS) -->
      <div class="space-y-3">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Input
            v-model="proxyData.host"
            label="Proxy Host"
            placeholder="proxy.example.com"
            :rules="['required']"
          />
          <Input
            v-model.number="proxyData.port"
            label="Proxy Port"
            type="number"
            placeholder="8080"
            min="1"
            max="65535"
            :rules="['required']"
          />
        </div>

        <!-- Proxy Authentication (optional) -->
        <div v-if="proxyData.proxy_type !== 'socks4'" class="space-y-3">
          <h4 class="text-sm font-medium text-gray-300">
            Proxy Authentication (Optional)
          </h4>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Input
              v-model="proxyData.username"
              label="Username"
              placeholder="proxy username"
            />
            <Input
              v-model="proxyData.password"
              label="Password"
              :type="showProxyPassword ? 'text' : 'password'"
              placeholder="proxy password"
              :right-icon="showProxyPassword ? EyeOff : Eye"
              @right-icon-click="showProxyPassword = !showProxyPassword"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { Eye, EyeOff } from "lucide-vue-next";
import Input from "./Input.vue";
import Select from "./Select.vue";
import Checkbox from "./Checkbox.vue";
import type { SSHProxy } from "../types/ssh";

interface ProxySettingsProps {
  proxy: SSHProxy | null;
  disabled?: boolean;
}

const props = withDefaults(defineProps<ProxySettingsProps>(), {
  proxy: null,
  disabled: false,
});

const emit = defineEmits<{
  "update:proxy": [proxy: SSHProxy | null];
}>();

// State
const enabled = ref(false);
const showProxyPassword = ref(false);

// Proxy data
const proxyData = ref<SSHProxy>({
  proxy_type: "http",
  host: "",
  port: 8080,
  username: "",
  password: "",
});

// Methods
const handleToggleProxy = (value: boolean): void => {
  enabled.value = value;
  emitProxy();
};

const handleTypeChange = (): void => {
  // Reset proxy settings when type changes
  proxyData.value.host = "";
  proxyData.value.port = proxyData.value.proxy_type === "http" ? 8080 : 1080;
  proxyData.value.username = "";
  proxyData.value.password = "";

  emitProxy();
};

const emitProxy = (): void => {
  if (!enabled.value) {
    emit("update:proxy", null);
    return;
  }

  // Build clean proxy object
  const cleanProxy: SSHProxy = {
    proxy_type: proxyData.value.proxy_type,
    host: proxyData.value.host || "",
    port: proxyData.value.port || 8080,
  };

  // Only add username and password if they have actual values
  if (proxyData.value.username?.trim()) {
    cleanProxy.username = proxyData.value.username.trim();
  }
  if (proxyData.value.password?.trim()) {
    cleanProxy.password = proxyData.value.password.trim();
  }

  // Create a new clean object to break any potential references
  emit("update:proxy", JSON.parse(JSON.stringify(cleanProxy)));
};

// Watch for prop changes
watch(
  () => props.proxy,
  (newProxy) => {
    if (newProxy) {
      enabled.value = true;
      proxyData.value = {
        proxy_type: newProxy.proxy_type || "http",
        host: newProxy.host || "",
        port: newProxy.port || 8080,
        username: newProxy.username || "",
        password: newProxy.password || "",
      };
    } else {
      enabled.value = false;
    }
  },
  { immediate: true },
);

// Watch for proxy data changes
watch(
  [proxyData, enabled],
  () => {
    emitProxy();
  },
  { deep: true },
);
</script>
