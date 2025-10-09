<template>
  <Modal
    :id="modalId"
    :title="isEditing ? 'Edit Saved Command' : 'New Saved Command'"
    :show-close="true"
  >
    <Form @submit="handleSubmit">
      <div class="space-y-4">
        <!-- Command Name -->
        <Input
          id="command-name"
          v-model="formData.name"
          label="Command Name"
          placeholder="e.g., Update System Packages"
          required
          :error="errors.name"
        />

        <!-- Command -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Command *
          </label>
          <textarea
            v-model="formData.command"
            class="w-full px-3 py-2 bg-[#1a1a1a] border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
            rows="3"
            placeholder="e.g., sudo apt update && sudo apt upgrade -y"
            required
          />
          <span v-if="errors.command" class="text-red-400 text-xs mt-1">
            {{ errors.command }}
          </span>
        </div>

        <!-- Description -->
        <Input
          id="command-description"
          v-model="formData.description"
          label="Description"
          placeholder="Brief description of what this command does"
          :error="errors.description"
        />

        <!-- Group Selection -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Group
          </label>
          <select
            v-model="formData.groupId"
            class="w-full px-3 py-2 bg-[#1a1a1a] border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            <option value="">No Group (Ungrouped)</option>
            <option
              v-for="group in groups"
              :key="group.base.id"
              :value="group.base.id"
            >
              {{ group.name }}
            </option>
          </select>
        </div>

        <!-- Tags -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Tags
          </label>
          <div class="flex flex-wrap gap-2 mb-2">
            <span
              v-for="(tag, index) in parsedTags"
              :key="index"
              class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-900/30 text-blue-300"
            >
              {{ tag }}
              <button
                type="button"
                class="ml-1 hover:text-blue-200"
                @click="removeTag(index)"
              >
                ×
              </button>
            </span>
          </div>
          <div class="flex gap-2">
            <input
              v-model="newTag"
              type="text"
              class="flex-1 px-3 py-2 bg-[#1a1a1a] border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm"
              placeholder="Add a tag..."
              @keyup.enter="addTag"
            />
            <Button
              type="button"
              variant="outline"
              size="sm"
              @click="addTag"
            >
              Add
            </Button>
          </div>
        </div>

        <!-- Favorite Toggle -->
        <Checkbox
          id="is-favorite"
          v-model="formData.isFavorite"
          label="Mark as favorite"
        />
      </div>

      <!-- Actions -->
      <template #footer>
        <div class="flex justify-end space-x-3">
          <Button
            type="button"
            variant="outline"
            @click="closeModal"
          >
            Cancel
          </Button>
          <Button
            type="submit"
            :loading="loading"
          >
            {{ isEditing ? 'Update Command' : 'Create Command' }}
          </Button>
        </div>
      </template>
    </Form>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import Checkbox from "../ui/Checkbox.vue";
import { useOverlay } from "../../composables/useOverlay";
import { useSavedCommandStore } from "../../stores/savedCommand";
import type { SavedCommand, SavedCommandGroup, CreateSavedCommandRequest, UpdateSavedCommandRequest } from "../../types/savedCommand";

interface Props {
  modalId: string;
  command?: SavedCommand;
  groups: SavedCommandGroup[];
  defaultGroupId?: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  success: [command: SavedCommand];
  error: [error: string];
}>();

const { closeOverlay } = useOverlay();
const savedCommandStore = useSavedCommandStore();

const loading = ref(false);
const newTag = ref("");

const formData = ref({
  name: "",
  command: "",
  description: "",
  groupId: "",
  isFavorite: false,
});

const parsedTags = ref<string[]>([]);

const errors = ref({
  name: "",
  command: "",
  description: "",
});

const isEditing = computed(() => !!props.command);

// Initialize form data
const initializeForm = () => {
  if (props.command) {
    // Editing existing command
    formData.value = {
      name: props.command.name,
      command: props.command.command,
      description: props.command.description || "",
      groupId: props.command.groupId || "",
      isFavorite: props.command.isFavorite,
    };

    // Parse tags
    try {
      parsedTags.value = props.command.tags ? JSON.parse(props.command.tags) : [];
    } catch {
      parsedTags.value = [];
    }
  } else {
    // Creating new command
    formData.value = {
      name: "",
      command: "",
      description: "",
      groupId: props.defaultGroupId || "",
      isFavorite: false,
    };
    parsedTags.value = [];
  }

  // Clear errors
  errors.value = {
    name: "",
    command: "",
    description: "",
  };
};

const validateForm = (): boolean => {
  let isValid = true;
  errors.value = { name: "", command: "", description: "" };

  if (!formData.value.name.trim()) {
    errors.value.name = "Command name is required";
    isValid = false;
  }

  if (!formData.value.command.trim()) {
    errors.value.command = "Command is required";
    isValid = false;
  }

  return isValid;
};

const addTag = () => {
  const tag = newTag.value.trim();
  if (tag && !parsedTags.value.includes(tag)) {
    parsedTags.value.push(tag);
    newTag.value = "";
  }
};

const removeTag = (index: number) => {
  parsedTags.value.splice(index, 1);
};

const handleSubmit = async () => {
  if (!validateForm()) return;

  loading.value = true;
  try {
    const tagsJson = parsedTags.value.length > 0 ? JSON.stringify(parsedTags.value) : undefined;

    if (isEditing.value && props.command) {
      // Update existing command
      const updateRequest: UpdateSavedCommandRequest = {
        name: formData.value.name,
        command: formData.value.command,
        description: formData.value.description || undefined,
        groupId: formData.value.groupId || undefined,
        tags: tagsJson,
        isFavorite: formData.value.isFavorite,
      };

      const updatedCommand = await savedCommandStore.updateCommand(props.command.base.id, updateRequest);
      emit("success", updatedCommand);
    } else {
      // Create new command
      const createRequest: CreateSavedCommandRequest = {
        name: formData.value.name,
        command: formData.value.command,
        description: formData.value.description || undefined,
        groupId: formData.value.groupId || undefined,
        tags: tagsJson,
        isFavorite: formData.value.isFavorite,
      };

      const newCommand = await savedCommandStore.createCommand(createRequest);
      emit("success", newCommand);
    }

    closeModal();
  } catch (error) {
    console.error("Failed to save command:", error);
    emit("error", error instanceof Error ? error.message : "Failed to save command");
  } finally {
    loading.value = false;
  }
};

const closeModal = () => {
  closeOverlay(props.modalId);
};

// Watch for prop changes to reinitialize form
watch(() => props.command, initializeForm, { immediate: true });
watch(() => props.defaultGroupId, () => {
  if (!isEditing.value) {
    formData.value.groupId = props.defaultGroupId || "";
  }
});

onMounted(() => {
  initializeForm();
});
</script>
