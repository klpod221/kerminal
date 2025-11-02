<template>
  <Modal
    id="ssh-profile-modal"
    :title="sshProfileId ? 'Edit SSH Profile' : 'Create SSH Profile'"
    size="lg"
  >
    <Form ref="sshProfileForm" @submit="handleSubmit">
      <!-- Basic Information -->
      <h4
        class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2"
      >
        Basic Information
      </h4>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Input
          id="profile-name"
          v-model="sshProfile.name"
          label="Profile Name"
          placeholder="My Server"
          rules="required|min:3|max:50"
          :autofocus="true"
        />

        <Select
          id="profile-group"
          v-model="sshProfile.groupId"
          label="Group (Optional)"
          placeholder="Select a group"
          :options="groupOptions"
        />
      </div>

      <Textarea
        id="profile-description"
        v-model="sshProfile.description"
        label="Description (Optional)"
        placeholder="A brief description of the server"
        :rows="2"
      />

      <!-- Connection Details -->
      <h4
        class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2"
      >
        Connection Details
      </h4>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="md:col-span-2">
          <Input
            id="profile-host"
            v-model="sshProfile.host"
            label="Host"
            placeholder="example.com or 192.168.1.100"
            rules="required"
          />
        </div>

        <Input
          id="profile-port"
          v-model.number="sshProfile.port"
          label="Port"
          type="number"
          placeholder="22"
          :min="1"
          :max="65535"
          rules="required|min_value:1|max_value:65535"
        />
      </div>

      <Input
        id="profile-username"
        v-model="sshProfile.username"
        label="Username"
        placeholder="root"
        rules="required|min:1|max:32"
      />

      <!-- Authentication -->
      <h4
        class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2"
      >
        Authentication
      </h4>

      <Select
        id="profile-auth-method"
        v-model="sshProfile.authMethod"
        label="Authentication Method"
        placeholder="Select authentication method"
        :options="authMethodOptions"
        rules="required"
      />

      <!-- Password Authentication -->
      <div v-if="sshProfile.authMethod === 'Password'" class="space-y-4">
        <Input
          id="profile-password"
          v-model="authPassword"
          label="Password"
          type="password"
          :placeholder="
            sshProfileId
              ? 'Leave empty to keep current password'
              : 'Enter password'
          "
          :rules="sshProfileId ? '' : 'required'"
        />
        <div v-if="sshProfileId" class="text-xs text-gray-400">
          Leave empty to keep the current password. Enter a new password to
          change it.
        </div>
      </div>

      <!-- SSH Key Reference -->
      <div v-else-if="sshProfile.authMethod === 'KeyReference'" class="mb-2">
        <Select
          id="profile-key-reference"
          v-model="authKeyId"
          label="Select SSH Key"
          placeholder="Choose a saved SSH key"
          :options="sshKeyOptions"
          rules="required"
        />
        <div class="text-sm text-gray-400">
          <p>
            Don't have any keys?
            <button
              type="button"
              class="text-blue-400 hover:text-blue-300 underline"
              @click="openKeyManager"
            >
              Manage SSH Keys
            </button>
          </p>
        </div>
      </div>

      <!-- Advanced Settings -->
      <Collapsible
        title="Advanced Settings"
        subtitle="Optional configuration"
        :default-expanded="false"
      >
        <Input
          id="profile-timeout"
          v-model.number="sshProfile.timeout"
          label="Timeout (s)"
          type="number"
          placeholder="30"
          :min="1"
          :max="300"
        />

        <ColorPicker
          id="profile-color"
          v-model="sshProfile.color"
          label="Profile Color"
          placeholder="Pick a color for the profile"
        />

        <Checkbox
          id="profile-keep-alive"
          v-model="sshProfile.keepAlive"
          label="Keep Alive"
        />

        <Checkbox
          id="profile-compression"
          v-model="sshProfile.compression"
          label="Enable Compression"
        />
      </Collapsible>

      <!-- Proxy Configuration -->
      <Collapsible
        title="Proxy Configuration"
        subtitle="Route connection through proxy"
        :default-expanded="false"
      >
        <Checkbox
          id="enable-proxy"
          v-model="enableProxy"
          label="Enable Proxy"
        />

        <div v-if="enableProxy">
          <Select
            id="proxy-type"
            v-model="proxyConfig.proxyType"
            label="Proxy Type"
            placeholder="Select proxy type"
            :options="proxyTypeOptions"
            rules="required"
          />

          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div class="md:col-span-2">
              <Input
                id="proxy-host"
                v-model="proxyConfig.host"
                label="Proxy Host"
                placeholder="proxy.example.com"
                rules="required"
              />
            </div>

            <Input
              id="proxy-port"
              v-model.number="proxyConfig.port"
              label="Proxy Port"
              type="number"
              placeholder="8080"
              :min="1"
              :max="65535"
              rules="required|min_value:1|max_value:65535"
            />
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Input
              id="proxy-username"
              v-model="proxyConfig.username"
              label="Proxy Username (Optional)"
              placeholder="username"
            />

            <Input
              id="proxy-password"
              v-model="proxyConfig.password"
              label="Proxy Password (Optional)"
              type="password"
              placeholder="password"
            />
          </div>
        </div>
      </Collapsible>
    </Form>

    <template #footer>
      <div class="flex justify-between w-full">
        <Button
          type="button"
          variant="secondary"
          @click="closeOverlay('ssh-profile-modal')"
        >
          Cancel
        </Button>
        <div class="flex gap-2">
          <Button
            type="button"
            variant="outline"
            @click="testConnection"
            :loading="isTesting"
          >
            Test Connection
          </Button>
          <Button
            type="submit"
            variant="primary"
            :loading="isLoading"
            :icon="Save"
            @click="handleSubmit"
          >
            {{ sshProfileId ? "Update Profile" : "Create Profile" }}
          </Button>
        </div>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted } from "vue";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Textarea from "../ui/Textarea.vue";
import Select from "../ui/Select.vue";
import ColorPicker from "../ui/ColorPicker.vue";
import Checkbox from "../ui/Checkbox.vue";
import Button from "../ui/Button.vue";
import Collapsible from "../ui/Collapsible.vue";
import { message } from "../../utils/message";
import { Save } from "lucide-vue-next";
import { useSSHStore } from "../../stores/ssh";
import { useSshKeyStore } from "../../stores/sshKey";
import { useOverlay } from "../../composables/useOverlay";
import type {
  SSHProfile,
  AuthMethod,
  AuthData,
  CreateSSHProfileRequest,
  UpdateSSHProfileRequest,
} from "../../types/ssh";

const props = defineProps<{
  sshProfileId?: string | null;
  groupId?: string | null;
}>();

const sshStore = useSSHStore();
const sshKeyStore = useSshKeyStore();
const { closeOverlay, getOverlayProp, openOverlay } = useOverlay();

const sshProfileId = getOverlayProp(
  "ssh-profile-modal",
  "sshProfileId",
  props.sshProfileId,
  null,
);
const groupId = getOverlayProp(
  "ssh-profile-modal",
  "groupId",
  props.groupId,
  null,
);

const sshProfileForm = ref<InstanceType<typeof Form> | null>(null);
const isLoading = ref(false);
const isTesting = ref(false);

const sshProfile = ref({
  name: "",
  host: "",
  port: 22,
  username: "",
  groupId: "",
  authMethod: "Password" as AuthMethod,
  timeout: 30,
  keepAlive: true,
  compression: false,
  color: "#3b82f6",
  description: "",
} as Partial<SSHProfile>);

const authPassword = ref("");
const authKeyId = ref("");

const enableProxy = ref(false);
const proxyConfig = ref({
  proxyType: "Http" as "Http" | "Socks4" | "Socks5",
  host: "",
  port: 8080,
  username: "",
  password: "",
});

const authMethodOptions = [
  { value: "Password", label: "Password" },
  { value: "KeyReference", label: "SSH Key" },
];

const proxyTypeOptions = [
  { value: "Http", label: "HTTP" },
  { value: "Socks4", label: "SOCKS4" },
  { value: "Socks5", label: "SOCKS5" },
];

const groupOptions = computed(() => [
  { value: "", label: "No Group" },
  ...sshStore.groups.map((group) => ({
    value: group.id,
    label: group.name,
  })),
]);

const sshKeyOptions = computed(() =>
  sshKeyStore.keys.map((key) => ({
    value: key.id,
    label: `${key.name} (${key.keyType})`,
  })),
);

const openKeyManager = () => {
  openOverlay("ssh-key-manager-modal");
};

const loadProfile = async () => {
  if (!sshProfileId.value) return;

  isLoading.value = true;
  try {
    const profile = await sshStore.findProfileById(sshProfileId.value);
    if (profile) {
      sshProfile.value = { ...profile };

      if (profile.authData) {
        if ("Password" in profile.authData) {
          authPassword.value = "";
        } else if ("KeyReference" in profile.authData) {
          authKeyId.value = profile.authData.KeyReference.keyId;
        }
      }

      if (profile.proxy) {
        enableProxy.value = true;
        proxyConfig.value = {
          proxyType: profile.proxy.proxyType,
          host: profile.proxy.host,
          port: profile.proxy.port,
          username: profile.proxy.username || "",
          password: profile.proxy.password || "",
        };
      } else {
        enableProxy.value = false;
        proxyConfig.value = {
          proxyType: "Http",
          host: "",
          port: 8080,
          username: "",
          password: "",
        };
      }
    }
  } catch (error) {
    console.error("Error loading SSH profile:", error);
  } finally {
    isLoading.value = false;
  }
};

const buildAuthData = (): AuthData | null => {
  switch (sshProfile.value.authMethod) {
    case "Password":
      if (sshProfileId.value && !authPassword.value.trim()) {
        return null;
      }
      return { Password: { password: authPassword.value } };

    case "KeyReference":
      return { KeyReference: { keyId: authKeyId.value } };

    default:
      throw new Error("Unsupported authentication method");
  }
};

const testConnection = async () => {
  const isValid = await sshProfileForm.value?.validate();
  if (!isValid || !sshProfile.value) return;

  isTesting.value = true;
  try {
    let testRequest;

    if (sshProfileId.value) {
      let authData;

      const formAuthData = buildAuthData();
      if (formAuthData) {
        authData = formAuthData;
      } else {
        const existingProfile = await sshStore.findProfileById(
          sshProfileId.value,
        );
        if (!existingProfile) {
          throw new Error("Profile not found");
        }
        authData = existingProfile.authData;
      }

      testRequest = {
        host: sshProfile.value.host,
        port: sshProfile.value.port,
        username: sshProfile.value.username,
        authMethod: sshProfile.value.authMethod,
        authData: authData,
        timeout: sshProfile.value.timeout || 30,
        keepAlive: sshProfile.value.keepAlive ?? true,
        compression: sshProfile.value.compression ?? false,
        proxy: enableProxy.value
          ? {
              proxyType: proxyConfig.value.proxyType,
              host: proxyConfig.value.host,
              port: proxyConfig.value.port,
              username: proxyConfig.value.username || null,
              password: proxyConfig.value.password || null,
            }
          : null,
      };
    } else {
      const authData = buildAuthData();
      if (!authData) {
        throw new Error("Cannot test connection without authentication data");
      }

      testRequest = {
        host: sshProfile.value.host,
        port: sshProfile.value.port,
        username: sshProfile.value.username,
        authMethod: sshProfile.value.authMethod,
        authData: authData,
        timeout: sshProfile.value.timeout || 5,
        keepAlive: sshProfile.value.keepAlive ?? true,
        compression: sshProfile.value.compression ?? false,
        proxy: enableProxy.value
          ? {
              proxyType: proxyConfig.value.proxyType,
              host: proxyConfig.value.host,
              port: proxyConfig.value.port,
              username: proxyConfig.value.username || null,
              password: proxyConfig.value.password || null,
            }
          : null,
      };
    }

    if (!testRequest.host || !testRequest.username || !testRequest.authMethod) {
      throw new Error("Missing required fields for connection test");
    }

    await sshStore.testConnection({
      host: testRequest.host,
      port: testRequest.port || 22,
      username: testRequest.username,
      authMethod: testRequest.authMethod,
      authData: testRequest.authData,
      timeout: testRequest.timeout,
      keepAlive: testRequest.keepAlive,
      compression: testRequest.compression,
      proxy: testRequest.proxy,
    });
    message.success("SSH connection test successful!");
  } finally {
    isTesting.value = false;
  }
};

const handleSubmit = async () => {
  const isValid = await sshProfileForm.value?.validate();
  if (!isValid || !sshProfile.value) return;

  isLoading.value = true;

  try {
    const authData = buildAuthData();
    const profileData = {
      ...sshProfile.value,
      ...(authData && { authData }), // Only include authData if not null
      groupId: sshProfile.value.groupId || undefined,
      proxy: enableProxy.value
        ? {
            proxyType: proxyConfig.value.proxyType,
            host: proxyConfig.value.host,
            port: proxyConfig.value.port,
            username: proxyConfig.value.username || null,
            password: proxyConfig.value.password || null,
          }
        : null,
    };

    if (sshProfileId.value) {
      await sshStore.updateProfile(
        sshProfileId.value,
        profileData as UpdateSSHProfileRequest,
      );
      message.success("SSH profile updated successfully.");
    } else {
      await sshStore.createProfile(profileData as CreateSSHProfileRequest);
      message.success("SSH profile created successfully.");
    }

    closeModal();
  } finally {
    isLoading.value = false;
  }
};

const closeModal = () => {
  sshProfile.value = {
    name: "",
    host: "",
    port: 22,
    username: "",
    groupId: "",
    authMethod: "Password" as AuthMethod,
    timeout: 30,
    keepAlive: true,
    compression: false,
    color: "#3b82f6",
    description: "",
  } as Partial<SSHProfile>;
  authPassword.value = "";
  authKeyId.value = "";

  enableProxy.value = false;
  proxyConfig.value = {
    proxyType: "Http",
    host: "",
    port: 8080,
    username: "",
    password: "",
  };

  closeOverlay("ssh-profile-modal");
};

watch(
  () => [sshProfileId.value, groupId.value],
  ([newId, newGroupId]) => {
    if (newId) {
      loadProfile();
    } else {
      sshProfile.value = {
        name: "",
        host: "",
        port: 22,
        username: "",
        groupId: newGroupId || "",
        authMethod: "Password",
        timeout: 30,
        keepAlive: true,
        compression: false,
        color: "#3b82f6",
        description: "",
      };
      authPassword.value = "";
      authKeyId.value = "";

      enableProxy.value = false;
      proxyConfig.value = {
        proxyType: "Http",
        host: "",
        port: 8080,
        username: "",
        password: "",
      };
    }
  },
  { immediate: true },
);

onMounted(() => {
  sshKeyStore.loadKeys();
});
</script>
