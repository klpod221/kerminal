<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <Modal
    id="saved-command-modal"
    :title="isEditing ? 'Edit Saved Command' : 'New Saved Command'"
    :show-close-button="true"
  >
    <Form ref="commandForm" @submit="handleSubmit">
      <!-- Command Name -->
      <Input
        id="command-name"
        v-model="formData.name"
        label="Command Name"
        placeholder="e.g., Update System Packages"
        rules="required|min:3|max:100"
      />

      <!-- Command -->
      <SimpleCodeEditor
        id="command-editor"
        v-model="formData.command"
        label="Command"
        language="shell"
        height="150px"
        rules="required"
        :error-message="commandError"
        helper-text="Enter your shell command or script"
      />

      <!-- Secret Warning Box (if detected) -->
      <div
        v-if="secretFindings.length > 0"
        class="rounded-lg border border-amber-500/50 bg-amber-950/30 p-3 text-xs text-amber-300 space-y-2 animate-fade-in"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-1.5 font-semibold text-amber-400">
            <span>⚠️ Potential Secret Detected ({{ secretFindings.length }})</span>
          </div>
          <Button
            type="button"
            variant="outline"
            size="xs"
            class="text-amber-300 border-amber-500/50 hover:bg-amber-500/20"
            @click="applyMaskSecrets"
          >
            Mask Secrets
          </Button>
        </div>
        <p class="text-gray-300 text-xs">
          Found plain text credentials:
          <span class="font-mono text-amber-200">
            {{ secretFindings.map((f) => f.type).join(", ") }}
          </span>.
          We recommend using environment variables or placeholders.
        </p>
      </div>

      <!-- Scope Selection -->
      <div class="space-y-1">
        <label class="block text-xs font-medium text-gray-300">
          Target Scope
        </label>
        <div class="grid grid-cols-3 gap-2">
          <button
            type="button"
            class="px-3 py-2 text-xs font-medium rounded-lg border transition-all text-center"
            :class="
              selectedScope === 'any'
                ? 'border-blue-500 bg-blue-500/20 text-blue-300 ring-1 ring-blue-500/40'
                : 'border-gray-700 bg-gray-800/60 text-gray-400 hover:text-white'
            "
            @click="selectedScope = 'any'"
          >
            🌐 All Envs
          </button>
          <button
            type="button"
            class="px-3 py-2 text-xs font-medium rounded-lg border transition-all text-center"
            :class="
              selectedScope === 'local'
                ? 'border-emerald-500 bg-emerald-500/20 text-emerald-300 ring-1 ring-emerald-500/40'
                : 'border-gray-700 bg-gray-800/60 text-gray-400 hover:text-white'
            "
            @click="selectedScope = 'local'"
          >
            💻 Local Only
          </button>
          <button
            type="button"
            class="px-3 py-2 text-xs font-medium rounded-lg border transition-all text-center"
            :class="
              selectedScope === 'ssh'
                ? 'border-purple-500 bg-purple-500/20 text-purple-300 ring-1 ring-purple-500/40'
                : 'border-gray-700 bg-gray-800/60 text-gray-400 hover:text-white'
            "
            @click="selectedScope = 'ssh'"
          >
            ☁️ SSH Only
          </button>
        </div>
      </div>

      <!-- Description -->
      <Input
        id="command-description"
        v-model="formData.description"
        label="Description"
        placeholder="Brief description of what this command does"
        rules="max:500"
      />

      <!-- Group Selection -->
      <Select
        id="group-select"
        v-model="formData.groupId"
        label="Group"
        :options="groupOptions"
        size="md"
      />

      <!-- Tags -->
      <TagInput id="command-tags" v-model="parsedTags" label="Tags" size="sm" />

      <!-- Favorite Toggle -->
      <Checkbox
        id="is-favorite"
        v-model="formData.isFavorite"
        label="Mark as favorite"
      />

      <!-- Actions -->
    </Form>

    <template #footer>
      <div class="flex space-x-3">
        <Button type="button" variant="outline" @click="closeModal">
          Cancel
        </Button>
        <Button type="submit" :loading="loading" @click="handleSubmit">
          {{ isEditing ? "Update Command" : "Create Command" }}
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import Checkbox from "../ui/Checkbox.vue";
import Select from "../ui/Select.vue";
import TagInput from "../ui/TagInput.vue";
import SimpleCodeEditor from "../ui/SimpleCodeEditor.vue";
import { useOverlay } from "../../composables/useOverlay";
import { useSavedCommandStore } from "../../stores/savedCommand";
import { message } from "../../utils/message";
import {
  scanForSecrets,
  maskSecrets,
  type SecretFinding,
} from "../../services/security/secretScanner";
import {
  extractScopeFromTags,
  getDisplayTags,
  packTagsWithScope,
  type SavedCommandScope,
} from "../../types/savedCommand";

interface Props {
  commandId?: string | null;
  defaultGroupId?: string | null;
}

const props = defineProps<Props>();

const { closeOverlay, getOverlayProp } = useOverlay();
const savedCommandStore = useSavedCommandStore();

const commandId = getOverlayProp(
  "saved-command-modal",
  "commandId",
  props.commandId,
  null,
);
const defaultGroupId = getOverlayProp(
  "saved-command-modal",
  "defaultGroupId",
  props.defaultGroupId,
  null,
);

const commandForm = ref<InstanceType<typeof Form> | null>(null);
const loading = ref(false);
const commandError = ref<string>("");

const formData = ref({
  name: "",
  command: "",
  description: "",
  groupId: "",
  isFavorite: false,
});

const selectedScope = ref<SavedCommandScope>("any");
const secretFindings = ref<SecretFinding[]>([]);
const parsedTags = ref<string[]>([]);

const isEditing = computed(() => !!commandId.value);

const groupOptions = computed(() => [
  { value: "", label: "No Group (Ungrouped)" },
  ...savedCommandStore.groups.map((g) => ({
    value: g.id,
    label: g.name,
  })),
]);

// Real-time scan for secrets in command
watch(
  () => formData.value.command,
  (newCmd) => {
    if (commandError.value) {
      commandError.value = "";
    }
    secretFindings.value = scanForSecrets(newCmd);
  },
  { immediate: true },
);

const applyMaskSecrets = () => {
  formData.value.command = maskSecrets(formData.value.command);
  secretFindings.value = scanForSecrets(formData.value.command);
  message.success("Detected secrets replaced with variable placeholders.");
};

const loadCommand = async () => {
  if (!commandId.value) return;

  loading.value = true;
  const command = await savedCommandStore.findCommandById(commandId.value);
  if (command) {
    formData.value = {
      name: command.name,
      command: command.command,
      description: command.description || "",
      groupId: command.groupId || "",
      isFavorite: command.isFavorite,
    };

    selectedScope.value = extractScopeFromTags(command.tags);
    parsedTags.value = getDisplayTags(command.tags);
    secretFindings.value = scanForSecrets(command.command);
  }
  loading.value = false;
};

const handleSubmit = async () => {
  commandError.value = "";
  if (!formData.value.command || formData.value.command.trim().length === 0) {
    commandError.value = "Command is required";
    return;
  }

  const isValid = await commandForm.value?.validate();
  if (!isValid) return;

  loading.value = true;
  const packedTagsJson = packTagsWithScope(parsedTags.value, selectedScope.value);

  const commandData = {
    name: formData.value.name,
    command: formData.value.command,
    description: formData.value.description || undefined,
    groupId: formData.value.groupId || undefined,
    tags: packedTagsJson,
    isFavorite: formData.value.isFavorite,
  };

  if (isEditing.value && commandId.value) {
    await savedCommandStore.updateCommand(commandId.value, commandData);
    message.success("Command updated successfully.");
  } else {
    await savedCommandStore.createCommand(commandData);
    message.success("Command created successfully.");
  }

  closeModal();
  loading.value = false;
};

const closeModal = () => {
  formData.value = {
    name: "",
    command: "",
    description: "",
    groupId: "",
    isFavorite: false,
  };
  commandError.value = "";
  closeOverlay("saved-command-modal");
};

watch(
  () => [commandId.value, defaultGroupId.value],
  ([newCommandId, newDefaultGroupId]) => {
    if (newCommandId) {
      loadCommand();
    } else {
      formData.value = {
        name: "",
        command: "",
        description: "",
        groupId: newDefaultGroupId || "",
        isFavorite: false,
      };
      parsedTags.value = [];
      commandError.value = "";
    }
  },
  { immediate: true },
);
</script>
