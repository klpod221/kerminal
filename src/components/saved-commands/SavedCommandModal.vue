<template>
  <Modal
    :id="modalId"
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
      <Textarea
        id="command-textarea"
        v-model="formData.command"
        label="Command"
        placeholder="e.g., sudo apt update && sudo apt upgrade -y"
        :rows="3"
        rules="required|min:1"
        size="md"
      />

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
        :options="[
          { value: '', label: 'No Group (Ungrouped)' },
          ...groups.map((g) => ({ value: g.id, label: g.name })),
        ]"
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
import Textarea from "../ui/Textarea.vue";
import Select from "../ui/Select.vue";
import TagInput from "../ui/TagInput.vue";
import { useOverlay } from "../../composables/useOverlay";
import { useSavedCommandStore } from "../../stores/savedCommand";
import { safeJsonParse, safeJsonStringify } from "../../utils/helpers";
import type { SavedCommand, SavedCommandGroup } from "../../types/savedCommand";

interface Props {
  modalId: string;
  groups: SavedCommandGroup[];
  commandId?: string | null;
  defaultGroupId?: string | null;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  success: [command: SavedCommand];
  error: [error: string];
}>();

const { closeOverlay, getOverlayProp } = useOverlay();
const savedCommandStore = useSavedCommandStore();

const commandId = getOverlayProp(
  props.modalId,
  "commandId",
  props.commandId,
  null,
);
const defaultGroupId = getOverlayProp(
  props.modalId,
  "defaultGroupId",
  props.defaultGroupId,
  null,
);

const commandForm = ref<InstanceType<typeof Form> | null>(null);
const loading = ref(false);

const formData = ref({
  name: "",
  command: "",
  description: "",
  groupId: "",
  isFavorite: false,
});

const parsedTags = ref<string[]>([]);

const isEditing = computed(() => !!commandId.value);

const loadCommand = async () => {
  if (!commandId.value) return;

  loading.value = true;
  try {
    const command = await savedCommandStore.findCommandById(commandId.value);
    if (command) {
      formData.value = {
        name: command.name,
        command: command.command,
        description: command.description || "",
        groupId: command.groupId || "",
        isFavorite: command.isFavorite,
      };

      parsedTags.value = safeJsonParse<string[]>(command.tags, []);
    }
  } catch (error) {
    console.error("Error loading command:", error);
    emit("error", "Failed to load command");
  } finally {
    loading.value = false;
  }
};

const handleSubmit = async () => {
  const isValid = await commandForm.value?.validate();
  if (!isValid) return;

  loading.value = true;
  try {
    const tagsJson = safeJsonStringify(parsedTags.value) || undefined;

    const commandData = {
      name: formData.value.name,
      command: formData.value.command,
      description: formData.value.description || undefined,
      groupId: formData.value.groupId || undefined,
      tags: tagsJson,
      isFavorite: formData.value.isFavorite,
    };

    if (isEditing.value && commandId.value) {
      const updatedCommand = await savedCommandStore.updateCommand(
        commandId.value,
        commandData,
      );
      emit("success", updatedCommand);
    } else {
      const newCommand = await savedCommandStore.createCommand(commandData);
      emit("success", newCommand);
    }

    closeModal();
  } catch (error) {
    console.error("Failed to save command:", error);
    emit(
      "error",
      error instanceof Error ? error.message : "Failed to save command",
    );
  } finally {
    loading.value = false;
  }
};

const closeModal = () => {
  formData.value = {
    name: "",
    command: "",
    description: "",
    groupId: "",
    isFavorite: false,
  };
  closeOverlay(props.modalId);
};

watch(
  () => [commandId.value, defaultGroupId.value],
  ([newCommandId, newDefaultGroupId]) => {
    console.log("🔍 SavedCommandModal props changed:", {
      commandId: newCommandId,
      defaultGroupId: newDefaultGroupId,
    });

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
    }
  },
  { immediate: true },
);
</script>
