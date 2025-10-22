<template>
  <Modal
    id="tunnel-modal"
    :title="isEditing ? 'Edit Tunnel' : 'Create Tunnel'"
    size="lg"
  >
    <Form ref="tunnelForm" @submit="handleSubmit">
      <!-- Basic Information -->
      <h4
        class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2"
      >
        Basic Information
      </h4>

      <Input
        id="tunnel-name"
        v-model="form.name"
        label="Tunnel Name"
        placeholder="Enter tunnel name"
        rules="required"
      />

      <Textarea
        id="tunnel-description"
        v-model="form.description"
        label="Description (Optional)"
        placeholder="Optional description for this tunnel"
        :rows="2"
      />

      <Select
        id="tunnel-profile"
        v-model="form.profileId"
        label="SSH Profile"
        placeholder="Select SSH Profile"
        :options="profileOptions"
        rules="required"
      />

      <!-- Tunnel Configuration -->
      <h4
        class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2"
      >
        Tunnel Configuration
      </h4>

      <div class="space-y-1">
        <label class="block text-sm font-medium text-gray-300 mb-2">
          Tunnel Type *
        </label>
        <div class="grid grid-cols-3 gap-2">
          <Card
            v-for="type in tunnelTypes"
            :key="type.value"
            class="cursor-pointer transition-all duration-200 mb-2"
            :class="
              form.tunnelType === type.value
                ? 'ring-2 ring-blue-500 bg-blue-500/10'
                : 'hover:ring-1 hover:ring-gray-600'
            "
            @click="form.tunnelType = type.value"
          >
            <div class="text-center">
              <div
                class="font-semibold text-sm"
                :class="
                  form.tunnelType === type.value
                    ? 'text-blue-400'
                    : 'text-white'
                "
              >
                {{ type.label }}
              </div>
              <div class="text-xs mt-1 opacity-80 text-gray-400">
                {{ type.description }}
              </div>
            </div>
          </Card>
        </div>
      </div>

      <!-- Local Forwarding -->
      <div v-if="form.tunnelType === 'Local'" class="space-y-1">
        <div class="grid grid-cols-2 gap-4">
          <Input
            id="local-host-local"
            v-model="form.localHost"
            label="Local Host"
            placeholder="localhost"
            rules="required"
          />
          <Input
            id="local-port"
            v-model.number="form.localPort"
            label="Local Port"
            type="number"
            placeholder="8080"
            min="1"
            max="65535"
            rules="required|min_value:1|max_value:65535"
          />
        </div>
        <div class="grid grid-cols-2 gap-4">
          <Input
            id="remote-host-local"
            v-model="form.remoteHost"
            label="Remote Host"
            placeholder="localhost"
            rules="required"
          />
          <Input
            id="remote-port-local"
            v-model.number="form.remotePort"
            label="Remote Port"
            type="number"
            placeholder="80"
            min="1"
            max="65535"
            rules="required|min_value:1|max_value:65535"
          />
        </div>
        <div class="text-sm text-gray-400 bg-gray-800 p-3 rounded">
          <strong>Local Forwarding:</strong> Traffic to {{ form.localHost }}:{{
            form.localPort
          }}
          will be forwarded through SSH to {{ form.remoteHost }}:{{
            form.remotePort
          }}
        </div>
      </div>

      <!-- Remote Forwarding -->
      <div v-if="form.tunnelType === 'Remote'" class="space-y-1">
        <div class="grid grid-cols-2 gap-4">
          <Input
            id="remote-host-remote"
            v-model="form.remoteHost"
            label="Remote Host"
            placeholder="localhost"
            rules="required"
          />
          <Input
            id="remote-port-remote"
            v-model.number="form.remotePort"
            label="Remote Port"
            type="number"
            placeholder="8080"
            min="1"
            max="65535"
            rules="required|min_value:1|max_value:65535"
          />
        </div>
        <div class="grid grid-cols-2 gap-4">
          <Input
            id="local-host-remote"
            v-model="form.localHost"
            label="Local Host"
            placeholder="localhost"
            rules="required"
          />
          <Input
            id="local-port-remote"
            v-model.number="form.localPort"
            label="Local Port"
            type="number"
            placeholder="80"
            min="1"
            max="65535"
            rules="required|min_value:1|max_value:65535"
          />
        </div>
        <div class="text-sm text-gray-400 bg-gray-800 p-3 rounded mb-2">
          <strong>Remote Forwarding:</strong> Traffic to
          {{ form.remoteHost }}:{{ form.remotePort }} on the SSH server will be
          forwarded to {{ form.localHost }}:{{ form.localPort }}
        </div>
      </div>
      <!-- Dynamic Forwarding (SOCKS) -->
      <div v-if="form.tunnelType === 'Dynamic'" class="space-y-1">
        <div class="grid grid-cols-2 gap-4">
          <Input
            id="local-host-dynamic"
            v-model="form.localHost"
            label="Local Host"
            placeholder="localhost"
            rules="required"
          />
          <Input
            id="local-port-dynamic"
            v-model.number="form.localPort"
            label="Local Port"
            type="number"
            placeholder="1080"
            min="1"
            max="65535"
            rules="required|min_value:1|max_value:65535"
          />
        </div>
        <div class="text-sm text-gray-400 bg-gray-800 p-3 rounded mb-2">
          <strong>Dynamic Forwarding:</strong> Creates a SOCKS proxy on
          {{ form.localHost }}:{{ form.localPort }}
          that routes traffic through the SSH connection
        </div>
      </div>

      <!-- Options -->
      <h4
        class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2"
      >
        Options
      </h4>

      <Checkbox
        id="auto-start"
        v-model="form.autoStart"
        label="Auto-start tunnel"
        description="Automatically start this tunnel when the application launches"
      />
    </Form>

    <template #footer>
      <div class="flex justify-between w-full">
        <Button
          type="button"
          variant="ghost"
          @click="closeOverlay('tunnel-modal')"
        >
          Cancel
        </Button>
        <Button
          type="submit"
          variant="primary"
          :icon="isEditing ? Edit3 : Plus"
          :loading="submitting"
          @click="handleSubmit"
        >
          {{ isEditing ? "Update Tunnel" : "Create Tunnel" }}
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, reactive, watch, onMounted, computed } from "vue";
import { useTunnelStore } from "../../stores/tunnel";
import { useSSHStore } from "../../stores/ssh";
import { useOverlay } from "../../composables/useOverlay";
import { message } from "../../utils/message";
import type { TunnelType } from "../../types/tunnel";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import Card from "../ui/Card.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Select from "../ui/Select.vue";
import Textarea from "../ui/Textarea.vue";
import Checkbox from "../ui/Checkbox.vue";
import { Plus, Edit3 } from "lucide-vue-next";

const props = defineProps<{
  tunnelId?: string | null;
}>();

const tunnelStore = useTunnelStore();
const sshStore = useSSHStore();
const { closeOverlay, getOverlayProp } = useOverlay();

const tunnelId = getOverlayProp(
  "tunnel-modal",
  "tunnelId",
  props.tunnelId,
  null,
);
const isEditing = computed(() => !!tunnelId.value);

const tunnel = computed(() => {
  if (!tunnelId.value) return null;
  const tunnelWithStatus = tunnelStore.tunnels.find(
    (t) => t.id === tunnelId.value,
  );
  return tunnelWithStatus || null;
});

const form = reactive({
  name: "",
  description: "",
  profileId: "",
  tunnelType: "Remote" as TunnelType,
  localHost: "localhost",
  localPort: 8080,
  remoteHost: "localhost",
  remotePort: 80,
  autoStart: false,
});

const tunnelForm = ref<InstanceType<typeof Form> | null>(null);
const submitting = ref(false);

const tunnelTypes = [
  {
    value: "Remote" as TunnelType,
    label: "Remote",
    description: "Forward remote port to local",
  },
  {
    value: "Local" as TunnelType,
    label: "Local",
    description: "Forward local port to remote",
  },
  {
    value: "Dynamic" as TunnelType,
    label: "Dynamic",
    description: "SOCKS proxy",
  },
];

const profileOptions = computed(() =>
  sshStore.profiles.map((profile) => ({
    value: profile.id,
    label: `${profile.name} (${profile.username}@${profile.host}:${profile.port})`,
  })),
);

const handleSubmit = async () => {
  const isValid = await tunnelForm.value?.validate();
  if (!isValid) return;

  submitting.value = true;

  try {
    const tunnelData = {
      name: form.name,
      description: form.description,
      profileId: form.profileId,
      tunnelType: form.tunnelType,
      localHost: form.localHost,
      localPort: form.localPort,
      remoteHost: form.remoteHost,
      remotePort: form.remotePort,
      autoStart: form.autoStart,
    };

    if (isEditing.value && tunnel.value) {
      await tunnelStore.updateTunnel(tunnel.value.id, tunnelData);
      message.success("Tunnel updated successfully");
    } else {
      await tunnelStore.createTunnel(tunnelData);
      message.success("Tunnel created successfully");
    }

    closeOverlay("tunnel-modal");
  } catch (error) {
    console.error("Failed to save tunnel:", error);
    message.error("Failed to save tunnel. Please try again.");
  } finally {
    submitting.value = false;
  }
};

watch(
  () => form.tunnelType,
  (newType) => {
    if (newType === "Dynamic") {
      form.localPort = 1080;
      form.remoteHost = "";
      form.remotePort = 0;
    } else if (newType === "Local") {
      form.localPort = 8080;
      form.remotePort = 80;
    } else if (newType === "Remote") {
      form.localPort = 80;
      form.remotePort = 8080;
    }
  },
);

const initializeForm = () => {
  if (tunnel.value) {
    form.name = tunnel.value.name;
    form.description = tunnel.value.description || "";
    form.profileId = tunnel.value.profileId;
    form.tunnelType = tunnel.value.tunnelType;
    form.localHost = tunnel.value.localHost;
    form.localPort = tunnel.value.localPort;
    form.remoteHost = tunnel.value.remoteHost || "";
    form.remotePort = tunnel.value.remotePort || 0;
    form.autoStart = tunnel.value.autoStart;
  } else {
    form.name = "";
    form.description = "";
    form.profileId = "";
    form.tunnelType = "Local";
    form.localHost = "localhost";
    form.localPort = 8080;
    form.remoteHost = "localhost";
    form.remotePort = 80;
    form.autoStart = false;
  }
};

watch(
  () => tunnel.value,
  (newTunnel, oldTunnel) => {
    if (newTunnel !== oldTunnel) {
      initializeForm();
    }
  },
  { immediate: true },
);

watch(
  tunnelId,
  async (newId) => {
    if (newId) {
      await tunnelStore.loadTunnels();
    }

    setTimeout(() => {
      initializeForm();
    }, 100);
  },
  { immediate: true },
);

onMounted(async () => {
  await Promise.all([sshStore.loadProfiles(), tunnelStore.loadTunnels()]);

  initializeForm();
});
</script>
