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
    </Form>

    <template #footer>
      <div class="flex justify-between w-full">
        <Button
          type="button"
          variant="ghost"
          @click="closeOverlay('ssh-group-modal')"
        >
          Cancel
        </Button>
        <Button
          type="submit"
          variant="primary"
          :loading="sshStore.isLoading"
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
import Button from "../ui/Button.vue";
import { message } from "../../utils/message";
import { Save } from "lucide-vue-next";
import { useSSHStore } from "../../stores/ssh";
import { useOverlay } from "../../composables/useOverlay";
import type {
  SSHGroup,
  CreateSSHGroupRequest,
  UpdateSSHGroupRequest,
} from "../../types/ssh";

const props = defineProps<{
  sshGroupId?: string | null;
}>();

const sshStore = useSSHStore();
const { closeOverlay, getOverlayProp } = useOverlay();

const sshGroupId = getOverlayProp(
  "ssh-group-modal",
  "sshGroupId",
  props.sshGroupId,
  null,
);

const sshGroupForm = ref<InstanceType<typeof Form> | null>(null);
const sshGroup = ref({
  name: "",
  description: "",
  color: "#000000",
} as Partial<SSHGroup>);

const loadGroup = () => {
  if (!sshGroupId.value) return;

  const group = sshStore.findGroupById(sshGroupId.value);
  if (group) {
    sshGroup.value = { ...group };
  }
};

const handleSubmit = async () => {
  const isValid = await sshGroupForm.value?.validate();
  if (!isValid || !sshGroup.value) return;

  try {
    if (sshGroupId.value) {
      const updateData: UpdateSSHGroupRequest = {
        name: sshGroup.value.name,
        description: sshGroup.value.description || null,
        color: sshGroup.value.color || null,
      };
      await sshStore.updateGroup(sshGroupId.value, updateData);
      message.success("SSH group updated successfully.");
    } else {
      const createData: CreateSSHGroupRequest = {
        name: sshGroup.value.name!,
        description: sshGroup.value.description,
        color: sshGroup.value.color,
      };
      await sshStore.createGroup(createData);
      message.success("SSH group created successfully.");
    }
    closeModal();
  } catch (error) {
    // Error is handled by the store
  }
};

const closeModal = () => {
  sshGroup.value = {
    name: "",
    description: "",
    color: "#000000",
  } as Partial<SSHGroup>;
  closeOverlay("ssh-group-modal");
};

watch(
  () => sshGroupId.value,
  (newId) => {
    if (newId) {
      loadGroup();
    } else {
      sshGroup.value = {
        name: "",
        description: "",
        color: "#000000",
      } as Partial<SSHGroup>;
    }
  },
  { immediate: true },
);
</script>
