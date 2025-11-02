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
      <CodeEditor
        id="command-editor"
        v-model="formData.command"
        label="Command"
        language="shell"
        height="150px"
        :error="commandError"
        helper-text="Enter your shell command or script"
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
import CodeEditor from "../ui/CodeEditor.vue";
import { useOverlay } from "../../composables/useOverlay";
import { useSavedCommandStore } from "../../stores/savedCommand";
import { safeJsonParse, safeJsonStringify } from "../../utils/helpers";
import { message } from "../../utils/message";

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

const parsedTags = ref<string[]>([]);

const isEditing = computed(() => !!commandId.value);

const groupOptions = computed(() => [
  { value: "", label: "No Group (Ungrouped)" },
  ...savedCommandStore.groups.map((g) => ({
    value: g.id,
    label: g.name,
  })),
]);

watch(
  () => formData.value.command,
  () => {
    if (commandError.value) {
      commandError.value = "";
    }
  },
);

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

    parsedTags.value = safeJsonParse<string[]>(command.tags, []);
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
