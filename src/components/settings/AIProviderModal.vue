<template>
  <Modal
    id="ai-provider-modal"
    :title="editingProvider ? 'Edit Provider' : 'Add AI Provider'"
    :icon="editingProvider ? Pencil : Plus"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    size="md"
  >
    <div class="space-y-4">
      <!-- Provider Type -->
      <div class="space-y-2">
        <label class="text-sm font-medium text-gray-300">Provider Type</label>
        <div class="grid grid-cols-5 gap-2">
          <button
            v-for="type in providerTypes"
            :key="type.value"
            class="flex flex-col items-center gap-1 p-3 rounded-lg border transition-all"
            :class="
              providerForm.type === type.value
                ? 'border-blue-500 bg-blue-500/20'
                : 'border-gray-600 bg-gray-800 hover:border-gray-500'
            "
            @click="selectProviderType(type.value)"
          >
            <component :is="type.icon" class="w-5 h-5" :class="type.color" />
            <span class="text-xs text-gray-300">{{ type.label }}</span>
          </button>
        </div>
      </div>

      <!-- Name -->
      <Input
        id="provider-name"
        v-model="providerForm.name"
        label="Name"
        placeholder="My OpenAI Provider"
        :left-icon="Tag"
        :helper="false"
      />

      <!-- API Key (not for Ollama) -->
      <Input
        v-if="providerForm.type !== 'ollama'"
        id="provider-api-key"
        v-model="providerForm.apiKey"
        label="API Key"
        placeholder="sk-..."
        type="password"
        :left-icon="Key"
        :helper="false"
      />

      <!-- Base URL -->
      <Input
        id="provider-base-url"
        v-model="providerForm.baseUrl"
        label="Base URL"
        :placeholder="getDefaultBaseUrl(providerForm.type)"
        :left-icon="Globe"
        :helper="false"
      />

      <!-- Model -->
      <Select
        id="provider-model"
        v-model="providerForm.model"
        label="Model"
        :options="modelOptions"
        :left-icon="Cpu"
        :helper="false"
      />

      <!-- Custom Model Input for Ollama/Custom -->
      <Input
        v-if="providerForm.type === 'custom' || providerForm.type === 'ollama'"
        id="custom-model"
        v-model="providerForm.model"
        label="Custom Model Name"
        placeholder="llama3.2, mistral, etc."
        :helper="false"
      />

      <!-- Test Connection -->
      <div class="flex items-center gap-3">
        <Button
          variant="outline"
          :icon="
            testResult?.success
              ? CheckCircle
              : testResult === null
                ? Wifi
                : XCircle
          "
          :text="isTestingConnection ? 'Testing...' : 'Test Connection'"
          :disabled="isTestingConnection || !canTestConnection"
          @click="testConnection"
        />
        <span
          v-if="testResult"
          class="text-xs"
          :class="testResult.success ? 'text-green-400' : 'text-red-400'"
        >
          {{ testResult.message }}
        </span>
      </div>
    </div>

    <template #footer>
      <Button variant="outline" @click="closeProviderModal">Cancel</Button>
      <Button
        variant="primary"
        :icon="editingProvider ? Save : Plus"
        :text="editingProvider ? 'Save Changes' : 'Add Provider'"
        :disabled="!isFormValid"
        @click="saveProvider"
      />
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import {
  Plus,
  Pencil,
  Tag,
  Key,
  Globe,
  Wifi,
  CheckCircle,
  XCircle,
  Save,
  Bot,
  Brain,
  Sparkles,
  Cpu,
  Cog,
} from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import Input from "../ui/Input.vue";
import Select from "../ui/Select.vue";
import { useAIStore } from "../../stores/ai";
import { useOverlay } from "../../composables/useOverlay";
import type {
  AIProviderConfig,
  AIProviderType,
  AIConnectionTestResult,
} from "../../types/ai";
import { DEFAULT_BASE_URLS, DEFAULT_MODELS } from "../../types/ai";
import { message } from "../../utils/message";

const aiStore = useAIStore();
const { closeOverlay, getOverlayProp, isOverlayVisible } = useOverlay();

// Provider Modal State
const editingProvider = ref<AIProviderConfig | null>(null);
const isTestingConnection = ref(false);
const testResult = ref<AIConnectionTestResult | null>(null);

const providerForm = ref({
  type: "openai" as AIProviderType,
  name: "",
  apiKey: "",
  baseUrl: "",
  model: "",
});

const providerTypes = [
  { value: "openai", label: "OpenAI", icon: Bot, color: "text-green-400" },
  {
    value: "anthropic",
    label: "Claude",
    icon: Brain,
    color: "text-orange-400",
  },
  { value: "gemini", label: "Gemini", icon: Sparkles, color: "text-blue-400" },
  { value: "ollama", label: "Ollama", icon: Cpu, color: "text-purple-400" },
  { value: "custom", label: "Custom", icon: Cog, color: "text-gray-400" },
] as const;

const availableModels = computed(() => {
  return aiStore.getAvailableModels(providerForm.value.type);
});

const modelOptions = computed(() => {
  return availableModels.value.map((model) => ({
    value: model,
    label: model,
  }));
});

const canTestConnection = computed(() => {
  if (!providerForm.value.name || !providerForm.value.model) return false;
  if (providerForm.value.type !== "ollama" && !providerForm.value.apiKey)
    return false;
  return true;
});

const isFormValid = computed(() => canTestConnection.value);

// Watch for overlay opening/props
const providerProp = getOverlayProp<AIProviderConfig | null>(
  "ai-provider-modal",
  "provider",
  null,
);
const isVisible = computed(() => isOverlayVisible("ai-provider-modal"));

watch(isVisible, (visible) => {
  if (visible) {
    if (providerProp.value) {
      editProvider(providerProp.value);
    } else {
      resetForm();
    }
  }
});

watch(
  () => providerForm.value.type,
  (newType) => {
    // Only reset if type changed by user, not during init
    if (editingProvider.value && editingProvider.value.type === newType) {
      // If editing and same type, don't reset
      return;
    }

    // If user switched type manually, perform reset defaults
    // But we need to distinguish manual switch vs programmatic set
    // Simple logic: if providerForm.model is empty or valid for other type, reset
    // For now, let's keep it simple: always reset model if type changes unless it matches editingProvider's type?
    // Actually the original logic was:
    // providerForm.value.model = DEFAULT_MODELS[newType] || "";
    // providerForm.value.baseUrl = "";

    // We'll update model default if it's empty or invalid
    if (!editingProvider.value || editingProvider.value.type !== newType) {
      providerForm.value.model = DEFAULT_MODELS[newType] || "";
      providerForm.value.baseUrl = "";
      testResult.value = null;
    }
  },
);

// Methods
function getDefaultBaseUrl(type: AIProviderType) {
  return DEFAULT_BASE_URLS[type] || "";
}

function selectProviderType(type: AIProviderType) {
  providerForm.value.type = type;
  // Trigger watch?
}

function resetForm() {
  providerForm.value = {
    type: "openai",
    name: "",
    apiKey: "",
    baseUrl: "",
    model: DEFAULT_MODELS.openai,
  };
  editingProvider.value = null;
  testResult.value = null;
}

function editProvider(provider: AIProviderConfig) {
  editingProvider.value = provider;
  providerForm.value = {
    type: provider.type,
    name: provider.name,
    apiKey: provider.apiKey || "",
    baseUrl: provider.baseUrl || "",
    model: provider.model,
  };
  testResult.value = null;
}

async function testConnection() {
  isTestingConnection.value = true;
  testResult.value = null;

  try {
    const tempProvider: AIProviderConfig = {
      id: "test",
      type: providerForm.value.type,
      name: providerForm.value.name,
      apiKey: providerForm.value.apiKey,
      baseUrl: providerForm.value.baseUrl || undefined,
      model: providerForm.value.model,
      isEnabled: true,
      isDefault: false,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    testResult.value = await aiStore.testConnection(tempProvider);
  } catch (error) {
    testResult.value = {
      success: false,
      message: error instanceof Error ? error.message : "Connection failed",
    };
  } finally {
    isTestingConnection.value = false;
  }
}

async function saveProvider() {
  try {
    if (editingProvider.value) {
      await aiStore.updateProvider(editingProvider.value.id, {
        name: providerForm.value.name,
        apiKey: providerForm.value.apiKey || undefined,
        baseUrl: providerForm.value.baseUrl || undefined,
        model: providerForm.value.model,
      });
    } else {
      await aiStore.addProvider({
        type: providerForm.value.type,
        name: providerForm.value.name,
        apiKey: providerForm.value.apiKey || undefined,
        baseUrl: providerForm.value.baseUrl || undefined,
        model: providerForm.value.model,
      });
    }
    closeProviderModal();
  } catch (error) {
    message.error("Failed to save provider");
  }
}

function closeProviderModal() {
  closeOverlay("ai-provider-modal");
  resetForm();
}
</script>
