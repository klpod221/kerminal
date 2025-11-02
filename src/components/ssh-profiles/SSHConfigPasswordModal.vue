<template>
  <Modal
    id="ssh-config-password-modal"
    title="SSH Authentication Required"
    size="sm"
    :icon="Lock"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
  >
    <div class="space-y-4">
      <!-- Host info -->
      <div class="bg-gray-800/50 rounded-lg p-4 space-y-2">
        <div class="flex items-center gap-2 text-sm">
          <component :is="Server" class="w-4 h-4 text-blue-400" />
          <span class="text-gray-400">Connecting to:</span>
        </div>
        <div class="font-mono text-sm text-white ml-6">
          {{ connectionString }}
        </div>
      </div>

      <!-- Password form -->
      <Form ref="passwordForm" @submit.prevent="handleConnect">
        <Input
          id="ssh-config-password"
          ref="passwordInput"
          v-model="password"
          type="password"
          label="Password"
          placeholder="Enter SSH password"
          :error="errorMessage"
          :helper="true"
          helper-text="Enter the password for SSH authentication"
          :left-icon="Lock"
          required
          autocomplete="off"
          @keyup.enter="handleConnect"
        />
      </Form>
    </div>

    <template #footer>
      <Button variant="ghost" :disabled="isConnecting" @click="handleCancel">
        Cancel
      </Button>
      <Button
        variant="primary"
        :disabled="!password || isConnecting"
        :loading="isConnecting"
        :icon="isConnecting ? undefined : PlugZap"
        @click="handleConnect"
      >
        Connect
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import { Server, Lock, PlugZap } from "lucide-vue-next";
import { useOverlay } from "../../composables/useOverlay";
import type { SSHConfigHost } from "../../types/ssh";

const props = defineProps<{
  host?: SSHConfigHost;
  onConnect?: (password: string) => Promise<void>;
}>();

const { overlayStore, closeOverlay, getOverlayProp } = useOverlay();

const password = ref("");
const errorMessage = ref("");
const isConnecting = ref(false);
const passwordForm = ref<InstanceType<typeof Form>>();
const passwordInput = ref<InstanceType<typeof Input>>();

const host = getOverlayProp<SSHConfigHost | undefined>(
  "ssh-config-password-modal",
  "host",
  props.host,
  undefined,
);

const onConnect = getOverlayProp<
  ((password: string) => Promise<void>) | undefined
>("ssh-config-password-modal", "onConnect", props.onConnect, undefined);

const connectionString = computed(() => {
  if (!host.value) return "";
  const user = host.value.user || "user";
  return `${user}@${host.value.hostname}:${host.value.port}`;
});

const handleConnect = async () => {
  if (!password.value || !onConnect.value) {
    errorMessage.value = "Password is required";
    return;
  }

  isConnecting.value = true;
  errorMessage.value = "";

  try {
    await onConnect.value(password.value);
    handleCancel();
  } catch (error) {
    // onConnect callback may throw error, display it
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isConnecting.value = false;
  }
};

const handleCancel = () => {
  password.value = "";
  errorMessage.value = "";
  isConnecting.value = false;
  closeOverlay("ssh-config-password-modal");
};

watch(
  () => overlayStore.isVisible("ssh-config-password-modal"),
  (isVisible) => {
    if (isVisible) {
      nextTick(() => {
        passwordInput.value?.focus();
      });
    } else {
      password.value = "";
      errorMessage.value = "";
      isConnecting.value = false;
    }
  },
);
</script>
