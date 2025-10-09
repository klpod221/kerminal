<template>
  <Modal
    :id="modalId"
    :title="isEditing ? 'Edit Command Group' : 'New Command Group'"
    :show-close="true"
  >
    <Form @submit="handleSubmit">
      <div class="space-y-4">
        <!-- Group Name -->
        <Input
          id="group-name"
          v-model="formData.name"
          label="Group Name"
          placeholder="e.g., System Administration"
          required
          :error="errors.name"
        />

        <!-- Description -->
        <Input
          id="group-description"
          v-model="formData.description"
          label="Description"
          placeholder="Brief description of this group"
          :error="errors.description"
        />

        <!-- Color Picker -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Color
          </label>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="color in colorOptions"
              :key="color.value"
              type="button"
              class="w-8 h-8 rounded-full border-2 transition-all duration-200 hover:scale-110"
              :class="formData.color === color.value ? 'border-white ring-2 ring-blue-500' : 'border-gray-600'"
              :style="{ backgroundColor: color.value }"
              :title="color.name"
              @click="formData.color = color.value"
            />
          </div>
          <div class="mt-2 flex items-center space-x-2">
            <input
              v-model="formData.color"
              type="color"
              class="w-8 h-8 rounded border border-gray-600 bg-transparent"
            />
            <Input
              id="group-color-hex"
              v-model="formData.color"
              placeholder="#6b7280"
              class="flex-1"
            />
          </div>
        </div>

        <!-- Icon Selection -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Icon (optional)
          </label>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="icon in iconOptions"
              :key="icon.name"
              type="button"
              class="flex items-center justify-center w-10 h-10 rounded border-2 transition-all duration-200 hover:scale-110"
              :class="formData.icon === icon.name ? 'border-white bg-blue-900/30' : 'border-gray-600 hover:border-gray-400'"
              :title="icon.name"
              @click="formData.icon = formData.icon === icon.name ? '' : icon.name"
            >
              <component :is="icon.component" :size="16" class="text-gray-300" />
            </button>
          </div>
          <Input
            id="group-icon-custom"
            v-model="formData.icon"
            placeholder="Custom icon name"
            class="mt-2"
          />
        </div>

        <!-- Preview -->
        <div class="p-3 bg-[#1a1a1a] rounded-lg border border-gray-600">
          <h4 class="text-sm font-medium text-gray-300 mb-2">Preview</h4>
          <div class="flex items-center space-x-2">
            <div
              class="w-3 h-3 rounded-full"
              :style="{ backgroundColor: formData.color || '#6b7280' }"
            />
            <span class="text-white font-medium">
              {{ formData.name || 'Group Name' }}
            </span>
            <component
              v-if="selectedIconComponent"
              :is="selectedIconComponent"
              :size="14"
              class="text-gray-400"
            />
          </div>
          <p
            v-if="formData.description"
            class="text-xs text-gray-400 mt-1"
          >
            {{ formData.description }}
          </p>
        </div>
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
            {{ isEditing ? 'Update Group' : 'Create Group' }}
          </Button>
        </div>
      </template>
    </Form>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import {
  Folder,
  Terminal,
  Server,
  Database,
  Settings,
  Code,
  GitBranch,
  Package,
  Shield,
  Zap,
  Wrench,
  Monitor
} from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import { useSavedCommandStore } from "../../stores/savedCommand";
import type { SavedCommandGroup, CreateSavedCommandGroupRequest, UpdateSavedCommandGroupRequest } from "../../types/savedCommand";

interface Props {
  modalId: string;
  group?: SavedCommandGroup;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  success: [group: SavedCommandGroup];
  error: [error: string];
}>();

const { closeOverlay } = useOverlay();
const savedCommandStore = useSavedCommandStore();

const loading = ref(false);

const formData = ref({
  name: "",
  description: "",
  color: "#6b7280",
  icon: "",
});

const errors = ref({
  name: "",
  description: "",
});

const colorOptions = [
  { name: "Gray", value: "#6b7280" },
  { name: "Red", value: "#ef4444" },
  { name: "Orange", value: "#f97316" },
  { name: "Yellow", value: "#eab308" },
  { name: "Green", value: "#22c55e" },
  { name: "Blue", value: "#3b82f6" },
  { name: "Indigo", value: "#6366f1" },
  { name: "Purple", value: "#a855f7" },
  { name: "Pink", value: "#ec4899" },
  { name: "Teal", value: "#14b8a6" },
];

const iconOptions = [
  { name: "folder", component: Folder },
  { name: "terminal", component: Terminal },
  { name: "server", component: Server },
  { name: "database", component: Database },
  { name: "settings", component: Settings },
  { name: "code", component: Code },
  { name: "git-branch", component: GitBranch },
  { name: "package", component: Package },
  { name: "shield", component: Shield },
  { name: "zap", component: Zap },
  { name: "wrench", component: Wrench },
  { name: "monitor", component: Monitor },
];

const isEditing = computed(() => !!props.group);

const selectedIconComponent = computed(() => {
  const icon = iconOptions.find(opt => opt.name === formData.value.icon);
  return icon?.component;
});

// Initialize form data
const initializeForm = () => {
  if (props.group) {
    // Editing existing group
    formData.value = {
      name: props.group.name,
      description: props.group.description || "",
      color: props.group.color || "#6b7280",
      icon: props.group.icon || "",
    };
  } else {
    // Creating new group
    formData.value = {
      name: "",
      description: "",
      color: "#6b7280",
      icon: "",
    };
  }

  // Clear errors
  errors.value = {
    name: "",
    description: "",
  };
};

const validateForm = (): boolean => {
  let isValid = true;
  errors.value = { name: "", description: "" };

  if (!formData.value.name.trim()) {
    errors.value.name = "Group name is required";
    isValid = false;
  }

  return isValid;
};

const handleSubmit = async () => {
  if (!validateForm()) return;

  loading.value = true;
  try {
    if (isEditing.value && props.group) {
      // Update existing group
      const updateRequest: UpdateSavedCommandGroupRequest = {
        name: formData.value.name,
        description: formData.value.description || undefined,
        color: formData.value.color || undefined,
        icon: formData.value.icon || undefined,
      };

      const updatedGroup = await savedCommandStore.updateGroup(props.group.base.id, updateRequest);
      emit("success", updatedGroup);
    } else {
      // Create new group
      const createRequest: CreateSavedCommandGroupRequest = {
        name: formData.value.name,
        description: formData.value.description || undefined,
        color: formData.value.color || undefined,
        icon: formData.value.icon || undefined,
      };

      const newGroup = await savedCommandStore.createGroup(createRequest);
      emit("success", newGroup);
    }

    closeModal();
  } catch (error) {
    console.error("Failed to save group:", error);
    emit("error", error instanceof Error ? error.message : "Failed to save group");
  } finally {
    loading.value = false;
  }
};

const closeModal = () => {
  closeOverlay(props.modalId);
};

// Watch for prop changes to reinitialize form
watch(() => props.group, initializeForm, { immediate: true });

onMounted(() => {
  initializeForm();
});
</script>
