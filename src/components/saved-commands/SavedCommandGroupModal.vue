<template>
  <Modal
    id="saved-command-group-modal"
    :title="isEditing ? 'Edit Command Group' : 'New Command Group'"
    :show-close-button="true"
  >
    <Form ref="groupForm" @submit="handleSubmit">
      <div class="space-y-4">
        <!-- Group Name -->
        <Input
          id="group-name"
          v-model="formData.name"
          label="Group Name"
          placeholder="e.g., System Administration"
          rules="required|min:3|max:50"
        />

        <!-- Description -->
        <Input
          id="group-description"
          v-model="formData.description"
          label="Description"
          placeholder="Brief description of this group"
          rules="max:200"
        />

        <!-- Color Picker -->
        <ColorPicker
          id="group-color"
          v-model="formData.color"
          label="Color"
          :preset-colors="colorOptions"
        />

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
              :class="
                formData.icon === icon.name
                  ? 'border-white bg-blue-900/30'
                  : 'border-gray-600 hover:border-gray-400'
              "
              :title="icon.name"
              @click="
                formData.icon = formData.icon === icon.name ? '' : icon.name
              "
            >
              <component
                :is="icon.component"
                :size="16"
                class="text-gray-300"
              />
            </button>
          </div>
        </div>

        <!-- Preview -->
        <div class="p-3 bg-[#1a1a1a] rounded-lg border border-gray-600">
          <h4 class="text-sm font-medium text-gray-300 mb-2">Preview</h4>
          <div class="flex items-center space-x-2">
            <component
              v-if="selectedIconComponent"
              :is="selectedIconComponent"
              :size="14"
              class="text-gray-400"
              :style="{ color: formData.color || '#6b7280' }"
            />
            <div
              v-else
              class="w-3 h-3 rounded-full"
              :style="{ backgroundColor: formData.color || '#6b7280' }"
            />
            <span class="text-white font-medium">
              {{ formData.name || "Group Name" }}
            </span>
          </div>
          <p v-if="formData.description" class="text-xs text-gray-400 mt-1">
            {{ formData.description }}
          </p>
        </div>
      </div>
    </Form>

    <!-- Actions -->
    <template #footer>
      <div class="flex justify-end space-x-3">
        <Button type="button" variant="outline" @click="closeModal">
          Cancel
        </Button>
        <Button type="submit" :loading="loading" @click="handleSubmit">
          {{ isEditing ? "Update Group" : "Create Group" }}
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
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
  Monitor,
} from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import ColorPicker from "../ui/ColorPicker.vue";
import { useOverlay } from "../../composables/useOverlay";
import { useSavedCommandStore } from "../../stores/savedCommand";
import { message } from "../../utils/message";

interface Props {
  groupId?: string | null;
}

const props = defineProps<Props>();

const { closeOverlay, getOverlayProp } = useOverlay();
const savedCommandStore = useSavedCommandStore();

const groupId = getOverlayProp(
  "saved-command-group-modal",
  "groupId",
  props.groupId,
  null,
);

const groupForm = ref<InstanceType<typeof Form> | null>(null);
const loading = ref(false);

const formData = ref({
  name: "",
  description: "",
  color: "#6b7280",
  icon: "",
});

const colorOptions = [
  "#6b7280", // Gray
  "#ef4444", // Red
  "#f97316", // Orange
  "#eab308", // Yellow
  "#22c55e", // Green
  "#3b82f6", // Blue
  "#6366f1", // Indigo
  "#a855f7", // Purple
  "#ec4899", // Pink
  "#14b8a6", // Teal
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

const isEditing = computed(() => !!groupId.value);

const selectedIconComponent = computed(() => {
  const icon = iconOptions.find((opt) => opt.name === formData.value.icon);
  return icon?.component;
});

const loadGroup = async () => {
  if (!groupId.value) return;

  loading.value = true;
  const group = await savedCommandStore.findGroupById(groupId.value);
  if (group) {
    formData.value = {
      name: group.name,
      description: group.description || "",
      color: group.color || "#6b7280",
      icon: group.icon || "",
    };
  }
  loading.value = false;
};

const handleSubmit = async () => {
  const isValid = await groupForm.value?.validate();
  if (!isValid) return;

  loading.value = true;
  const groupData = {
    name: formData.value.name,
    description: formData.value.description || undefined,
    color: formData.value.color || undefined,
    icon: formData.value.icon || undefined,
  };

  if (isEditing.value && groupId.value) {
    await savedCommandStore.updateGroup(groupId.value, groupData);
    message.success("Group updated successfully.");
  } else {
    await savedCommandStore.createGroup(groupData);
    message.success("Group created successfully.");
  }

  closeModal();
  loading.value = false;
};

const closeModal = () => {
  formData.value = {
    name: "",
    description: "",
    color: "#6b7280",
    icon: "",
  };
  closeOverlay("saved-command-group-modal");
};

watch(
  () => groupId.value,
  (newGroupId) => {
    if (newGroupId) {
      loadGroup();
    } else {
      formData.value = {
        name: "",
        description: "",
        color: "#6b7280",
        icon: "",
      };
    }
  },
  { immediate: true },
);
</script>
