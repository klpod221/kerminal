<template>
  <Modal
    id="ssh-group-modal"
    :title="sshGroupId ? 'Edit SSH Group' : 'Create SSH Group'"
    size="md"
  >
    <Form ref="sshGroupForm" @submit="handleSubmit">
      <Input
        id="group-name"
        v-model="sshGroup.name"
        label="Group Name"
        placeholder="My Servers"
        rules="required|min:3|max:50"
        :autofocus="true"
      />

      <Textarea
        id="group-description"
        v-model="sshGroup.description"
        label="Group Description"
        placeholder="A brief description of the group"
        :rows="3"
      />

      <ColorPicker
        id="group-color"
        v-model="sshGroup.color"
        label="Group Color"
        placeholder="Pick a color for the group"
      />

      <!-- Advanced Settings -->
      <Collapsible
        title="Advanced Settings"
        subtitle="Optional group configuration"
        :default-expanded="false"
      >
        <Select
          id="group-default-auth"
          v-model="sshGroup.defaultAuthMethod"
          label="Default Auth Method (Optional)"
          placeholder="Select default authentication method"
          :options="authMethodOptions"
        />

        <Checkbox
          id="group-expanded"
          v-model="sshGroup.isExpanded"
          label="Expanded by default"
        />
      </Collapsible>
    </Form>

    <template #footer>
      <div class="flex justify-between w-full">
        <Button type="button" variant="ghost" @click="closeOverlay('ssh-group-modal')">
          Cancel
        </Button>
        <Button
          type="submit"
          variant="primary"
          :loading="isLoading"
          :icon="Save"
          @click="handleSubmit"
        >
          {{ sshGroupId ? "Update Group" : "Create Group" }}
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Textarea from "../ui/Textarea.vue";
import ColorPicker from "../ui/ColorPicker.vue";
import Select from "../ui/Select.vue";
import Checkbox from "../ui/Checkbox.vue";
import Button from "../ui/Button.vue";
import Collapsible from "../ui/Collapsible.vue";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import { Save } from "lucide-vue-next";
import { useSSHStore } from "../../stores/ssh";
import { useOverlay } from "../../composables/useOverlay";
import { SSHGroup } from "../../types/ssh";

// Props
const props = defineProps<{
  sshGroupId?: string | null;
}>();

// Store and composables
const sshStore = useSSHStore();
const { closeOverlay, getOverlayProp } = useOverlay();

// Use overlay prop with fallback to direct prop
const sshGroupId = getOverlayProp("ssh-group-modal", "sshGroupId", props.sshGroupId, null);

// State
const sshGroupForm = ref<InstanceType<typeof Form> | null>(null);
const isLoading = ref(false);
const sshGroup = ref({
  name: "",
  description: "",
  color: "#000000",
  isExpanded: true,
  defaultAuthMethod: "",
} as Partial<SSHGroup>);

// Options
const authMethodOptions = [
  { value: "", label: "No Default" },
  { value: "Password", label: "Password" },
  { value: "PrivateKey", label: "Private Key" },
  { value: "PrivateKeyWithPassphrase", label: "Private Key with Passphrase" },
  { value: "Agent", label: "SSH Agent" },
];

// Functions
const loadGroup = () => {
  if (!sshGroupId.value) return;

  try {
    const group = sshStore.findGroupById(sshGroupId.value);
    if (group) {
      sshGroup.value = { ...group };
    }
  } catch (error) {
    console.error("Error loading SSH group:", error);
  }
};

const handleSubmit = async () => {
  console.log("Submitting form...");
  const isValid = await sshGroupForm.value?.validate();
  if (!isValid || !sshGroup.value) return;

  isLoading.value = true;
  try {
    const groupData = {
      name: sshGroup.value.name!,
      description: sshGroup.value.description,
      color: sshGroup.value.color,
      defaultAuthMethod: sshGroup.value.defaultAuthMethod,
    } as any; // Type assertion for create request

    if (sshGroupId.value) {
      await sshStore.updateGroup(sshGroupId.value, groupData);
      message.success("SSH group updated successfully.");
    } else {
      await sshStore.createGroup(groupData);
      message.success("SSH group created successfully.");
    }
    closeOverlay("ssh-group-modal");
  } catch (error) {
    console.error("Error saving SSH group:", error);
    message.error(getErrorMessage(error, "Failed to save SSH group."));
  } finally {
    isLoading.value = false;
  }
};

// Watch for prop changes
watch(
  () => sshGroupId.value,
  (newId) => {
    console.log('🔍 SSHGroupModal prop changed:', { sshGroupId: newId });
    if (newId) {
      loadGroup();
    } else {
      sshGroup.value = {
        name: "",
        description: "",
        color: "#000000",
        isExpanded: true,
        defaultAuthMethod: "",
      } as Partial<SSHGroup>;
    }
  },
  { immediate: true },
);
</script>
