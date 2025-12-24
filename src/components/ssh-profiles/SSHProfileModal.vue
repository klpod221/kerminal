<template>
  <Modal
    id="ssh-profile-modal"
    :title="sshProfileId ? 'Edit SSH Profile' : 'Create SSH Profile'"
    size="2xl"
  >
    <Form ref="sshProfileForm" @submit="handleSubmit">
      <!-- Tab Navigation -->
      <NavigationTabs v-model="activeTab" :tabs="tabs" class="mb-4" />

      <!-- Tab: Basic -->
      <div v-show="activeTab === 'basic'">
        <!-- Profile Info -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
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

        <!-- Connection -->
        <div class="grid grid-cols-1 md:grid-cols-4 gap-x-4">
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

          <Input
            id="profile-username"
            v-model="sshProfile.username"
            label="Username"
            placeholder="root"
            rules="required|min:1|max:32"
          />
        </div>

        <!-- Authentication -->
        <Select
          id="profile-auth-method"
          v-model="sshProfile.authMethod"
          label="Authentication Method"
          placeholder="Select authentication method"
          :options="authMethodOptions"
          rules="required"
        />

        <!-- Password -->
        <Input
          v-if="sshProfile.authMethod === 'Password'"
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
          helper-text="Leave empty to keep the current password"
        />

        <!-- SSH Key -->
        <div v-if="sshProfile.authMethod === 'KeyReference'">
          <Select
            id="profile-key-reference"
            v-model="authKeyId"
            label="SSH Key"
            placeholder="Choose a saved SSH key"
            :options="sshKeyOptions"
            rules="required"
          />
          <p class="text-xs text-gray-500 -mt-4">
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

      <!-- Tab: Terminal -->
      <div v-show="activeTab === 'terminal'">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
          <Input
            id="profile-working-dir"
            v-model="sshProfile.workingDir"
            label="Working Directory"
            placeholder="/home/user/projects"
            helper-text="Directory to start in after connecting"
          />

          <ColorPicker
            id="profile-color"
            v-model="sshProfile.color"
            label="Tab Color"
            placeholder="Pick a color"
          />
        </div>

        <SimpleCodeEditor
          id="profile-command"
          v-model="sshProfile.command"
          label="Startup Command"
          language="shell"
          height="80px"
          helper-text="Commands to run after connecting"
        />

        <div>
          <div class="block text-sm font-medium text-gray-300 mb-2">
            Environment Variables
          </div>
          <EnvVarEditor v-model="sshProfile.env" />
        </div>

        <div class="flex flex-wrap items-start gap-4">
          <div class="w-48">
            <Input
              id="profile-timeout"
              v-model.number="sshProfile.timeout"
              label="Connection Timeout (s)"
              type="number"
              placeholder="30"
              :min="1"
              :max="300"
              :helper="false"
            />
          </div>

          <div class="flex gap-4 pt-8">
            <Checkbox
              id="profile-keep-alive"
              v-model="sshProfile.keepAlive"
              label="Keep Alive"
            />

            <Checkbox
              id="profile-compression"
              v-model="sshProfile.compression"
              label="Compression"
            />
          </div>
        </div>
      </div>

      <!-- Tab: Network -->
      <div v-show="activeTab === 'network'" class="space-y-4">
        <!-- Proxy -->
        <div class="rounded-lg border border-gray-700 p-4">
          <div class="flex items-start gap-3">
            <Checkbox id="enable-proxy" v-model="enableProxy" />
            <div class="flex-1">
              <label
                for="enable-proxy"
                class="block text-sm font-medium text-gray-200 cursor-pointer"
              >
                Use Proxy
              </label>
              <p class="text-xs text-gray-500 mt-0.5">
                Route SSH connection through a proxy server (SOCKS5, HTTP, etc.)
              </p>
            </div>
          </div>

          <div v-if="enableProxy" class="mt-4 pt-4 border-t border-gray-700">
            <Select
              id="proxy-type"
              v-model="proxyConfig.proxyType"
              label="Proxy Type"
              placeholder="Select proxy type"
              :options="proxyTypeOptions"
              rules="required"
            />

            <div class="grid grid-cols-1 md:grid-cols-4 gap-x-4">
              <div class="md:col-span-2">
                <Input
                  id="proxy-host"
                  v-model="proxyConfig.host"
                  label="Host"
                  placeholder="proxy.example.com"
                  rules="required"
                />
              </div>

              <Input
                id="proxy-port"
                v-model.number="proxyConfig.port"
                label="Port"
                type="number"
                placeholder="8080"
                :min="1"
                :max="65535"
                rules="required|min_value:1|max_value:65535"
              />
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
              <Input
                id="proxy-username"
                v-model="proxyConfig.username"
                label="Username (Optional)"
                placeholder="username"
              />

              <Input
                id="proxy-password"
                v-model="proxyConfig.password"
                label="Password (Optional)"
                type="password"
                placeholder="password"
              />
            </div>
          </div>
        </div>

        <!-- Jump Host -->
        <div class="rounded-lg border border-gray-700 p-4">
          <div class="flex items-start gap-3">
            <Checkbox id="enable-jump-host" v-model="enableJumpHost" />
            <div class="flex-1">
              <label
                for="enable-jump-host"
                class="block text-sm font-medium text-gray-200 cursor-pointer"
              >
                Use Jump Host
              </label>
              <p class="text-xs text-gray-500 mt-0.5">
                Connect through an intermediate SSH server (bastion host)
              </p>
            </div>
          </div>

          <div v-if="enableJumpHost" class="mt-4 pt-4 border-t border-gray-700">
            <Select
              id="jump-host-profile"
              v-model="selectedJumpHostId"
              label="Jump Host Profile"
              placeholder="Select a profile"
              :options="jumpHostOptions"
              rules="required"
            />

            <p class="text-xs text-gray-500 -mt-4">
              Tip: Chain multiple jump hosts by configuring the selected profile
              with its own jump host.
            </p>
          </div>
        </div>
      </div>
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
            :loading="sshStore.isLoading"
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
import NavigationTabs from "../ui/NavigationTabs.vue";
import SimpleCodeEditor from "../ui/SimpleCodeEditor.vue";
import EnvVarEditor from "../ui/EnvVarEditor.vue";
import { message } from "../../utils/message";
import { Save, Server, Terminal, Network } from "lucide-vue-next";
import { useSSHStore } from "../../stores/ssh";
import { useSSHKeyStore } from "../../stores/sshKey";
import { useOverlay } from "../../composables/useOverlay";
import * as sshService from "../../services/sshProfile";
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
const sshKeyStore = useSSHKeyStore();
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
const isTesting = ref(false);
const isLoadingProfile = ref(false);
const activeTab = ref("basic");

const tabs = [
  { id: "basic", label: "Basic", icon: Server },
  { id: "terminal", label: "Terminal", icon: Terminal },
  { id: "network", label: "Network", icon: Network },
];

const sshProfile = ref<{
  name: string;
  host: string;
  port: number;
  username: string;
  groupId: string;
  authMethod: AuthMethod;
  authData?: AuthData;
  timeout: number;
  keepAlive: boolean;
  compression: boolean;
  color: string;
  description: string;
  command: string;
  workingDir: string;
  env: Record<string, string>;
  proxy?: SSHProfile["proxy"];
}>({
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
  command: "",
  workingDir: "",
  env: {},
});

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

// Jump Host state
const enableJumpHost = ref(false);
const selectedJumpHostId = ref("");

// Jump host options (other profiles that can be used as jump hosts)
const jumpHostOptions = computed(() => {
  // Filter out the current profile being edited
  return sshStore.profiles
    .filter((p) => p.id !== sshProfileId.value)
    .map((p) => ({
      value: p.id,
      label: `${p.name} (${p.username}@${p.host})`,
    }));
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

// Helper: Load auth data from profile
const loadAuthDataFromProfile = (profile: SSHProfile) => {
  if (!profile.authData) return;
  if ("Password" in profile.authData) {
    authPassword.value = "";
  } else if ("KeyReference" in profile.authData) {
    authKeyId.value = profile.authData.KeyReference.keyId;
  }
};

// Helper: Load proxy config from profile
const loadProxyFromProfile = (profile: SSHProfile) => {
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
};

// Helper: Load jump host config from profile
const loadJumpHostFromProfile = (profile: SSHProfile) => {
  if (
    profile.jumpHosts &&
    profile.jumpHosts.length > 0 &&
    profile.jumpHosts[0].profileId
  ) {
    enableJumpHost.value = true;
    selectedJumpHostId.value = profile.jumpHosts[0].profileId;
  } else {
    enableJumpHost.value = false;
    selectedJumpHostId.value = "";
  }
};

const loadProfile = async () => {
  if (!sshProfileId.value) return;

  isLoadingProfile.value = true;
  try {
    const profile = await sshStore.findProfileById(sshProfileId.value);
    if (!profile) return;

    sshProfile.value = {
      name: profile.name,
      host: profile.host,
      port: profile.port,
      username: profile.username,
      groupId: profile.groupId || "",
      authMethod: profile.authMethod,
      authData: profile.authData,
      timeout: profile.timeout || 30,
      keepAlive: profile.keepAlive,
      compression: profile.compression,
      color: profile.color || "#3b82f6",
      description: profile.description || "",
      command: profile.command || "",
      workingDir: profile.workingDir || "",
      env: profile.env || {},
      proxy: profile.proxy,
    };

    loadAuthDataFromProfile(profile);
    loadProxyFromProfile(profile);
    loadJumpHostFromProfile(profile);
  } catch (error) {
    console.error("Error loading SSH profile:", error);
  } finally {
    isLoadingProfile.value = false;
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

// Helper: Build proxy config for request
const buildProxyConfig = () => {
  if (!enableProxy.value) return null;
  return {
    proxyType: proxyConfig.value.proxyType,
    host: proxyConfig.value.host,
    port: proxyConfig.value.port,
    username: proxyConfig.value.username || null,
    password: proxyConfig.value.password || null,
  };
};

// Helper: Build jump hosts config for request
const buildJumpHostsConfig = () => {
  if (!enableJumpHost.value || !selectedJumpHostId.value) return null;
  return [{ profileId: selectedJumpHostId.value }];
};

const buildTestRequest = async () => {
  const authData = sshProfileId.value
    ? buildAuthData() ||
      (await sshService.getSSHProfile(sshProfileId.value))?.authData
    : buildAuthData();

  if (!authData) {
    throw new Error("Cannot test connection without authentication data");
  }

  return {
    host: sshProfile.value.host,
    port: sshProfile.value.port,
    username: sshProfile.value.username,
    authMethod: sshProfile.value.authMethod,
    authData,
    timeout: sshProfile.value.timeout || (sshProfileId.value ? 30 : 5),
    keepAlive: sshProfile.value.keepAlive ?? true,
    compression: sshProfile.value.compression ?? false,
    proxy: buildProxyConfig(),
    jumpHosts: buildJumpHostsConfig(),
  };
};

const testConnection = async () => {
  const isValid = await sshProfileForm.value?.validate();
  if (!isValid || !sshProfile.value) return;

  isTesting.value = true;
  try {
    const testRequest = await buildTestRequest();

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
      jumpHosts: testRequest.jumpHosts,
    });
    message.success("SSH connection test successful!");
  } finally {
    isTesting.value = false;
  }
};

const handleSubmit = async () => {
  const isValid = await sshProfileForm.value?.validate();
  if (!isValid || !sshProfile.value) return;

  try {
    const authData = buildAuthData();
    // Destructure to exclude authData from spread - we only want to send it when explicitly set
    const {
      authData: _existingAuthData,
      proxy: _existingProxy,
      ...restProfile
    } = sshProfile.value;

    const profileData = {
      ...restProfile,
      ...(authData && { authData }), // Only include authData if provided
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
      command: sshProfile.value.command,
      workingDir: sshProfile.value.workingDir,
      env: sshProfile.value.env,
      jumpHosts:
        enableJumpHost.value && selectedJumpHostId.value
          ? [{ profileId: selectedJumpHostId.value }]
          : [], // Send empty array to clear, not null
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
  } catch (error) {
    console.error("Failed to save SSH profile:", error);
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
    command: "",
    workingDir: "",
    env: {},
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

  activeTab.value = "basic";
  closeOverlay("ssh-profile-modal");
};

// Reset jump host state in closeModal function is handled by the watch

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
        command: "",
        workingDir: "",
        env: {},
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
      enableJumpHost.value = false;
      selectedJumpHostId.value = "";
    }
  },
  { immediate: true },
);

onMounted(() => {
  sshKeyStore.loadKeys();
});
</script>
