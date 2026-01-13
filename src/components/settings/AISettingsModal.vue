<template>
  <Modal
    id="ai-settings-modal"
    title="AI Assistant Settings"
    :icon="Sparkles"
    icon-background="bg-purple-500/20"
    icon-color="text-purple-400"
    size="lg"
    :show-close-button="true"
  >
    <div class="space-y-6">
      <!-- Enable/Disable AI -->
      <div
        class="flex items-center justify-between p-4 bg-bg-quaternary rounded-lg border border-gray-700"
      >
        <div class="flex items-center gap-3">
          <div class="p-2 rounded-lg bg-purple-500/20">
            <Sparkles class="w-5 h-5 text-purple-400" />
          </div>
          <div>
            <div class="text-sm font-medium text-white">
              AI Command Suggestions
            </div>
            <div class="text-xs text-gray-400">
              Get intelligent command completions
            </div>
          </div>
        </div>
        <Checkbox
          id="ai-enabled-checkbox"
          :model-value="aiStore.settings.isEnabled"
          :helper="false"
          @update:model-value="updateEnabled"
        />
      </div>

      <!-- Trigger Mode -->
      <div class="space-y-3" v-if="aiStore.settings.isEnabled">
        <div class="text-sm font-medium text-gray-300 flex items-center gap-2">
          <Zap class="w-4 h-4 text-yellow-400" />
          Trigger Mode
        </div>
        <div class="grid grid-cols-3 gap-2">
          <button
            v-for="mode in triggerModes"
            :key="mode.value"
            class="px-3 py-2 rounded-lg border text-sm transition-all"
            :class="
              aiStore.settings.triggerMode === mode.value
                ? 'border-purple-500 bg-purple-500/20 text-purple-400'
                : 'border-gray-600 bg-gray-800 text-gray-400 hover:border-gray-500'
            "
            @click="setTriggerMode(mode.value)"
          >
            {{ mode.label }}
          </button>
        </div>
        <p class="text-xs text-gray-500">
          {{ triggerModeDescription }}
        </p>
      </div>

      <div class="border-t border-gray-700" v-if="aiStore.settings.isEnabled" />

      <!-- AI Providers -->
      <div class="space-y-3" v-if="aiStore.settings.isEnabled">
        <div class="flex items-center justify-between">
          <div
            class="text-sm font-medium text-gray-300 flex items-center gap-2"
          >
            <Server class="w-4 h-4 text-blue-400" />
            AI Providers
          </div>
          <Button
            variant="ghost"
            size="sm"
            :icon="Plus"
            text="Add Provider"
            @click="openAddProvider"
          />
        </div>

        <!-- Provider List -->
        <div class="space-y-2">
          <Card
            v-for="provider in aiStore.providers"
            :key="provider.id"
            no-padding
            :custom-class="
              provider.isDefault
                ? 'p-3 !border-purple-500 !bg-purple-600/10'
                : 'p-3'
            "
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="p-2 rounded-lg"
                  :class="getProviderIconBg(provider.type)"
                >
                  <component
                    :is="getProviderIcon(provider.type)"
                    class="w-4 h-4"
                    :class="getProviderIconColor(provider.type)"
                  />
                </div>
                <div>
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-medium text-white">{{
                      provider.name
                    }}</span>
                    <span
                      v-if="provider.isDefault"
                      class="px-1.5 py-0.5 text-[9px] bg-purple-500/30 text-purple-400 rounded font-medium uppercase"
                    >
                      Default
                    </span>
                  </div>
                  <div class="text-xs text-gray-400">{{ provider.model }}</div>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  :icon="provider.isDefault ? Star : StarOff"
                  :title="
                    provider.isDefault ? 'Default provider' : 'Set as default'
                  "
                  @click="setDefault(provider.id)"
                />
                <Button
                  variant="ghost"
                  size="sm"
                  :icon="Pencil"
                  title="Edit provider"
                  @click="editProvider(provider)"
                />
                <Button
                  variant="ghost"
                  size="sm"
                  :icon="Trash2"
                  title="Remove provider"
                  @click="removeProvider(provider.id)"
                />
              </div>
            </div>
          </Card>

          <div v-if="aiStore.providers.length === 0" class="text-center py-8">
            <Server class="w-12 h-12 text-gray-600 mx-auto mb-3" />
            <p class="text-sm text-gray-400">No AI providers configured</p>
            <p class="text-xs text-gray-500 mt-1">
              Add a provider to get started
            </p>
          </div>
        </div>
      </div>

      <div class="border-t border-gray-700" v-if="aiStore.settings.isEnabled" />

      <!-- Context Settings -->
      <div class="space-y-3" v-if="aiStore.settings.isEnabled">
        <div class="text-sm font-medium text-gray-300 flex items-center gap-2">
          <Settings class="w-4 h-4 text-green-400" />
          Context Settings
        </div>
        <div class="space-y-2">
          <Checkbox
            id="checkbox-history"
            :model-value="aiStore.settings.includeHistory"
            label="Include command history"
            :helper="false"
            @update:model-value="
              (v: boolean) => updateContextSetting('includeHistory', v)
            "
          />
          <Checkbox
            id="checkbox-saved-commands"
            :model-value="aiStore.settings.includeSavedCommands"
            label="Include saved commands"
            :helper="false"
            @update:model-value="
              (v: boolean) => updateContextSetting('includeSavedCommands', v)
            "
          />
          <Checkbox
            id="checkbox-cwd"
            :model-value="aiStore.settings.includeCwd"
            label="Include current directory"
            :helper="false"
            @update:model-value="
              (v: boolean) => updateContextSetting('includeCwd', v)
            "
          />
          <Checkbox
            id="checkbox-system-info"
            :model-value="aiStore.settings.includeSystemInfo"
            label="Include system info (OS, shell)"
            :helper="false"
            @update:model-value="
              (v: boolean) => updateContextSetting('includeSystemInfo', v)
            "
          />
        </div>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { computed } from "vue";
import {
  Sparkles,
  Zap,
  Server,
  Plus,
  Pencil,
  Trash2,
  Star,
  StarOff,
  Settings,
  Bot,
  Brain,
  Cpu,
  Cloud,
  Cog,
} from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import Card from "../ui/Card.vue";
import Checkbox from "../ui/Checkbox.vue";
import { useAIStore } from "../../stores/ai";
import { useOverlay } from "../../composables/useOverlay";
import type {
  AIProviderConfig,
  AIProviderType,
  AISettings,
} from "../../types/ai";
import { message } from "../../utils/message";

const aiStore = useAIStore();
const { openOverlay } = useOverlay();

const triggerModes = [
  { value: "auto", label: "Auto" },
  { value: "manual", label: "Manual" },
  { value: "both", label: "Both" },
] as const;

const triggerModeDescription = computed(() => {
  switch (aiStore.settings.triggerMode) {
    case "auto":
      return "Suggestions appear automatically when you pause typing";
    case "manual":
      return "Press Ctrl+Space to get suggestions";
    case "both":
      return "Suggestions appear automatically or when you press Ctrl+Space";
    default:
      return "";
  }
});

// Methods
function updateEnabled(enabled: boolean) {
  aiStore.updateSettings({ isEnabled: enabled });
}

function updateContextSetting(key: keyof AISettings, value: boolean) {
  aiStore.updateSettings({ [key]: value });
}

function setTriggerMode(mode: "auto" | "manual" | "both") {
  aiStore.updateSettings({ triggerMode: mode });
}

function getProviderIcon(type: AIProviderType) {
  const icons: Record<AIProviderType, any> = {
    openai: Bot,
    anthropic: Brain,
    gemini: Sparkles,
    ollama: Cpu,
    custom: Cog,
  };
  return icons[type] || Cloud;
}

function getProviderIconBg(type: AIProviderType) {
  const colors: Record<AIProviderType, string> = {
    openai: "bg-green-500/20",
    anthropic: "bg-orange-500/20",
    gemini: "bg-blue-500/20",
    ollama: "bg-purple-500/20",
    custom: "bg-gray-500/20",
  };
  return colors[type] || "bg-gray-500/20";
}

function getProviderIconColor(type: AIProviderType) {
  const colors: Record<AIProviderType, string> = {
    openai: "text-green-400",
    anthropic: "text-orange-400",
    gemini: "text-blue-400",
    ollama: "text-purple-400",
    custom: "text-gray-400",
  };
  return colors[type] || "text-gray-400";
}

function openAddProvider() {
  openOverlay("ai-provider-modal", { provider: null });
}

function editProvider(provider: AIProviderConfig) {
  openOverlay("ai-provider-modal", { provider });
}

async function removeProvider(id: string) {
  try {
    await aiStore.removeProvider(id);
  } catch (error) {
    message.error("Failed to remove provider");
  }
}

async function setDefault(id: string) {
  await aiStore.setDefaultProvider(id);
}
</script>
