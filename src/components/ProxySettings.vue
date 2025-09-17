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
        v-model="proxyData.type"
        label="Proxy Type"
        @change="handleTypeChange"
      >
        <option value="http">HTTP Proxy</option>
        <option value="socks4">SOCKS4</option>
        <option value="socks5">SOCKS5</option>
        <option value="jump">Jump Host</option>
      </Select>

      <!-- Basic Proxy Settings (for HTTP/SOCKS) -->
      <div v-if="proxyData.type !== 'jump'" class="space-y-3">
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
        <div v-if="proxyData.type !== 'socks4'" class="space-y-3">
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

      <!-- Jump Host Settings -->
      <div v-if="proxyData.type === 'jump'" class="space-y-3">
        <h4 class="text-sm font-medium text-gray-300">
          Jump Host Configuration
        </h4>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Input
            v-model="proxyData.jumpHost"
            label="Jump Host"
            placeholder="jumphost.example.com"
            :rules="['required']"
          />
          <Input
            v-model.number="proxyData.jumpPort"
            label="Jump Port"
            type="number"
            placeholder="22"
            min="1"
            max="65535"
          />
        </div>

        <Input
          v-model="proxyData.jumpUser"
          label="Jump User"
          placeholder="jump username"
          :rules="['required']"
        />

        <!-- Jump Host Authentication -->
        <div class="space-y-3">
          <Select v-model="jumpAuthType" label="Jump Host Authentication">
            <option value="password">Password</option>
            <option value="key">SSH Key</option>
          </Select>

          <div v-if="jumpAuthType === 'password'">
            <Input
              v-model="proxyData.jumpPassword"
              label="Jump Password"
              :type="showJumpPassword ? 'text' : 'password'"
              placeholder="jump host password"
              :right-icon="showJumpPassword ? EyeOff : Eye"
              @right-icon-click="showJumpPassword = !showJumpPassword"
            />
          </div>

          <div v-if="jumpAuthType === 'key'">
            <Input
              v-model="proxyData.jumpKeyPath"
              label="Jump Host SSH Key"
              placeholder="~/.ssh/jumphost_key"
              :right-icon="Folder"
              @right-icon-click="selectJumpKeyFile"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { Eye, EyeOff, Folder } from "lucide-vue-next";
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
const showJumpPassword = ref(false);
const jumpAuthType = ref<"password" | "key">("password");

// Proxy data
const proxyData = ref<SSHProxy>({
  type: "http",
  host: "",
  port: 8080,
  username: "",
  password: "",
  jumpHost: "",
  jumpPort: 22,
  jumpUser: "",
  jumpKeyPath: "",
  jumpPassword: "",
});

// Methods
const handleToggleProxy = (value: boolean): void => {
  enabled.value = value;
  emitProxy();
};

const handleTypeChange = (): void => {
  // Reset type-specific fields
  if (proxyData.value.type === "jump") {
    proxyData.value.host = "";
    proxyData.value.port = 22;
    proxyData.value.username = "";
    proxyData.value.password = "";
  } else {
    proxyData.value.jumpHost = "";
    proxyData.value.jumpPort = 22;
    proxyData.value.jumpUser = "";
    proxyData.value.jumpKeyPath = "";
    proxyData.value.jumpPassword = "";
  }

  emitProxy();
};

const selectJumpKeyFile = async (): Promise<void> => {
  try {
    // const selectedPath = (await window.api.invoke('dialog.selectFile')) as string | null
    // if (selectedPath) {
    //   proxyData.value.jumpKeyPath = selectedPath
    //   emitProxy()
    // }
  } catch (error) {
    console.error("Failed to select jump key file:", error);
  }
};

const emitProxy = (): void => {
  if (!enabled.value) {
    emit("update:proxy", null);
    return;
  }

  // Build clean proxy object with only necessary properties
  const cleanProxy: SSHProxy = {
    type: proxyData.value.type,
    host: proxyData.value.host || "",
    port: proxyData.value.port || (proxyData.value.type === "jump" ? 22 : 8080),
  };

  if (proxyData.value.type !== "jump") {
    // Only add username and password if they have actual values
    if (proxyData.value.username?.trim()) {
      cleanProxy.username = proxyData.value.username.trim();
    }
    if (proxyData.value.password?.trim()) {
      cleanProxy.password = proxyData.value.password.trim();
    }
  } else {
    // Jump host specific properties
    cleanProxy.jumpHost = proxyData.value.jumpHost?.trim() || "";
    cleanProxy.jumpPort = proxyData.value.jumpPort || 22;
    cleanProxy.jumpUser = proxyData.value.jumpUser?.trim() || "";

    // Only add authentication method being used
    if (jumpAuthType.value === "key" && proxyData.value.jumpKeyPath?.trim()) {
      cleanProxy.jumpKeyPath = proxyData.value.jumpKeyPath.trim();
    } else if (
      jumpAuthType.value === "password" &&
      proxyData.value.jumpPassword?.trim()
    ) {
      cleanProxy.jumpPassword = proxyData.value.jumpPassword.trim();
    }
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
        type: newProxy.type || "http",
        host: newProxy.host || "",
        port: newProxy.port || (newProxy.type === "jump" ? 22 : 8080),
        username: newProxy.username || "",
        password: newProxy.password || "",
        jumpHost: newProxy.jumpHost || "",
        jumpPort: newProxy.jumpPort || 22,
        jumpUser: newProxy.jumpUser || "",
        jumpKeyPath: newProxy.jumpKeyPath || "",
        jumpPassword: newProxy.jumpPassword || "",
      };

      // Set jump auth type based on available data
      if (newProxy.type === "jump") {
        jumpAuthType.value = newProxy.jumpKeyPath ? "key" : "password";
      }
    } else {
      enabled.value = false;
    }
  },
  { immediate: true },
);

// Watch for proxy data changes
watch(
  [proxyData, enabled, jumpAuthType],
  () => {
    emitProxy();
  },
  { deep: true },
);
</script>
