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
          placeholder="Enter password"
          rules="required"
        />
      </div>

      <!-- SSH Key Reference -->
      <div v-else-if="sshProfile.authMethod === 'KeyReference'" class="space-y-4">
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

      <!-- Private Key Authentication -->
      <div
        v-else-if="
          sshProfile.authMethod === 'PrivateKey' ||
          sshProfile.authMethod === 'PrivateKeyWithPassphrase'
        "
        class="space-y-4"
      >
        <Select
          id="profile-key-type"
          v-model="authKeyType"
          label="Key Type"
          placeholder="Select key type"
          :options="keyTypeOptions"
          rules="required"
        />

        <Textarea
          id="profile-private-key"
          v-model="authPrivateKey"
          label="Private Key"
          placeholder="-----BEGIN PRIVATE KEY-----"
          :rows="6"
          rules="required"
        />

        <Textarea
          id="profile-public-key"
          v-model="authPublicKey"
          label="Public Key (Optional)"
          placeholder="ssh-rsa AAAAB3NzaC1yc2E..."
          :rows="2"
        />

        <Input
          v-if="sshProfile.authMethod === 'PrivateKeyWithPassphrase'"
          id="profile-passphrase"
          v-model="authPassphrase"
          label="Passphrase"
          type="password"
          placeholder="Enter passphrase"
          rules="required"
        />
      </div>

      <!-- Advanced Settings -->
      <Collapsible
        title="Advanced Settings"
        subtitle="Optional configuration"
        :default-expanded="false"
      >
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <Input
            id="profile-timeout"
            v-model.number="sshProfile.timeout"
            label="Timeout (seconds)"
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
        </div>

        <div class="flex gap-4">
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
        </div>

        <Input
          id="profile-tags"
          v-model="tagsInput"
          label="Tags (Optional)"
          placeholder="server, production, database (separated by commas)"
        />
      </Collapsible>

      <!-- Proxy Configuration -->
      <Collapsible
        title="Proxy Configuration"
        subtitle="Route connection through proxy"
        :default-expanded="false"
      >
        <div class="flex items-center mb-4">
          <Checkbox
            id="enable-proxy"
            v-model="enableProxy"
            label="Enable Proxy"
          />
        </div>

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
import { getErrorMessage } from "../../utils/helpers";
import { Save } from "lucide-vue-next";
import { useSSHStore } from "../../stores/ssh";
import { useSshKeyStore } from "../../stores/sshKey";
import { useOverlay } from "../../composables/useOverlay";
import type {
  SSHProfile,
  AuthMethod,
  KeyType,
  AuthData,
} from "../../types/ssh";
import { invoke } from "@tauri-apps/api/core";

// Props (for direct usage)
const props = defineProps<{
  sshProfileId?: string | null;
  groupId?: string | null;
}>();

// Store and composables
const sshStore = useSSHStore();
const sshKeyStore = useSshKeyStore();
const { closeOverlay, getOverlayProp, openOverlay } = useOverlay();

// Use composable to automatically merge props
// Use overlay prop with fallback to direct prop
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

// State
const sshProfileForm = ref<InstanceType<typeof Form> | null>(null);
const isLoading = ref(false);
const isTesting = ref(false);

// Form data
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

// Auth-specific data
const authPassword = ref("");
const authPrivateKey = ref("");
const authPublicKey = ref("");
const authPassphrase = ref("");
const authKeyType = ref<KeyType>("RSA");
const authKeyId = ref("");
const tagsInput = ref("");

// Proxy-specific data
const enableProxy = ref(false);
const proxyConfig = ref({
  proxyType: "Http" as "Http" | "Socks4" | "Socks5",
  host: "",
  port: 8080,
  username: "",
  password: "",
});

// Options
const authMethodOptions = [
  { value: "Password", label: "Password" },
  { value: "KeyReference", label: "SSH Key" },
  { value: "PrivateKey", label: "Private Key" },
  { value: "PrivateKeyWithPassphrase", label: "Private Key with Passphrase" },
  { value: "Agent", label: "SSH Agent" },
];

const keyTypeOptions = [
  { value: "RSA", label: "RSA" },
  { value: "Ed25519", label: "Ed25519" },
  { value: "ECDSA", label: "ECDSA" },
  { value: "DSA", label: "DSA" },
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

// Functions
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

      // Parse auth data
      if (profile.authData) {
        if ("Password" in profile.authData) {
          authPassword.value = profile.authData.Password.password;
        } else if ("KeyReference" in profile.authData) {
          authKeyId.value = profile.authData.KeyReference.keyId;
        } else if ("PrivateKey" in profile.authData) {
          authPrivateKey.value = profile.authData.PrivateKey.privateKey;
          authPublicKey.value = profile.authData.PrivateKey.publicKey || "";
          authKeyType.value = profile.authData.PrivateKey.keyType;
        } else if ("PrivateKeyWithPassphrase" in profile.authData) {
          authPrivateKey.value =
            profile.authData.PrivateKeyWithPassphrase.privateKey;
          authPublicKey.value =
            profile.authData.PrivateKeyWithPassphrase.publicKey || "";
          authPassphrase.value =
            profile.authData.PrivateKeyWithPassphrase.passphrase;
          authKeyType.value = profile.authData.PrivateKeyWithPassphrase.keyType;
        }
      }

      // Parse tags
      tagsInput.value = profile.tags?.join(", ") || "";

      // Parse proxy config
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

const buildAuthData = (): AuthData => {
  switch (sshProfile.value.authMethod) {
    case "Password":
      return { Password: { password: authPassword.value } };

    case "KeyReference":
      return { KeyReference: { keyId: authKeyId.value } };

    case "PrivateKey":
      return {
        PrivateKey: {
          privateKey: authPrivateKey.value,
          keyType: authKeyType.value,
          publicKey: authPublicKey.value || undefined,
        },
      };

    case "PrivateKeyWithPassphrase":
      return {
        PrivateKeyWithPassphrase: {
          privateKey: authPrivateKey.value,
          passphrase: authPassphrase.value,
          keyType: authKeyType.value,
          publicKey: authPublicKey.value || undefined,
        },
      };

    case "Agent":
      return {
        Agent: {
          publicKey: authPublicKey.value || undefined,
        },
      };

    default:
      throw new Error("Unsupported authentication method");
  }
};

const testConnection = async () => {
  const isValid = await sshProfileForm.value?.validate();
  if (!isValid || !sshProfile.value) return;

  isTesting.value = true;
  try {
    // Build test request with only necessary fields
    const testRequest = {
      host: sshProfile.value.host,
      port: sshProfile.value.port,
      username: sshProfile.value.username,
      authMethod: sshProfile.value.authMethod,
      authData: buildAuthData(),
      timeout: sshProfile.value.timeout || 30,
      keepAlive: sshProfile.value.keepAlive ?? true,
      compression: sshProfile.value.compression ?? false,
      proxy: enableProxy.value ? {
        proxyType: proxyConfig.value.proxyType,
        host: proxyConfig.value.host,
        port: proxyConfig.value.port,
        username: proxyConfig.value.username || undefined,
        password: proxyConfig.value.password || undefined,
      } : undefined,
    };

    await invoke("test_ssh_connection", { request: testRequest });
    message.success("Connection test successful!");
  } catch (error) {
    console.error("Error testing connection:", error);
    message.error(getErrorMessage(error, "Connection test failed."));
  } finally {
    isTesting.value = false;
  }
};

const handleSubmit = async () => {
  console.log("Submitting form...");
  const isValid = await sshProfileForm.value?.validate();
  if (!isValid || !sshProfile.value) return;

  isLoading.value = true;
  try {
    const profileData = {
      ...sshProfile.value,
      authData: buildAuthData(),
      tags: tagsInput.value
        .split(",")
        .map((tag) => tag.trim())
        .filter(Boolean),
      groupId: sshProfile.value.groupId || undefined,
      proxy: enableProxy.value
        ? {
            proxyType: proxyConfig.value.proxyType,
            host: proxyConfig.value.host,
            port: proxyConfig.value.port,
            username: proxyConfig.value.username || undefined,
            password: proxyConfig.value.password || undefined,
          }
        : undefined,
    } as any; // Type assertion to handle the complexity

    if (sshProfileId.value) {
      await sshStore.updateProfile(sshProfileId.value, profileData);
      message.success("SSH profile updated successfully.");
    } else {
      await sshStore.createProfile(profileData);
      message.success("SSH profile created successfully.");
    }

    closeOverlay("ssh-profile-modal");
  } catch (error) {
    console.error("Error saving SSH profile:", error);
    message.error(getErrorMessage(error, "Failed to save SSH profile."));
  } finally {
    isLoading.value = false;
  }
};

// Watch for prop changes
watch(
  () => [sshProfileId.value, groupId.value],
  ([newId, newGroupId]) => {
    console.log("🔍 SSHProfileModal props changed:", {
      sshProfileId: newId,
      groupId: newGroupId,
      fromProps: { sshProfileId: props.sshProfileId, groupId: props.groupId },
    });

    if (newId) {
      loadProfile();
    } else {
      // Reset form for new profile
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
      authPrivateKey.value = "";
      authPublicKey.value = "";
      authPassphrase.value = "";
      authKeyType.value = "RSA";
      authKeyId.value = "";
      tagsInput.value = "";

      // Reset proxy config
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

// Load SSH keys on mount
onMounted(() => {
  sshKeyStore.loadKeys();
});
</script>
