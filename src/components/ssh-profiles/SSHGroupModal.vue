<template>
  <Modal
    id="ssh-group-modal"
    :title="props.sshGroupId ? 'Edit SSH Group' : 'Create SSH Group'"
    size="md"
  >
    <Form ref="sshGroupForm" v-if="!isLoading">
      <Input
        id="group-name"
        v-model="sshGroup?.name"
        label="Group Name"
        placeholder="My Servers"
        rules="required|min:3|max:50"
        :autofocus="true"
      />

      <Textarea
        id="group-description"
        v-model="sshGroup?.description"
        label="Group Description"
        placeholder="A brief description of the group"
        :rows="3"
      />

      <ColorPicker
        id="group-color"
        v-model="sshGroup?.color"
        label="Group Color"
        placeholder="Pick a color for the group"
      />
    </Form>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button type="button" variant="secondary" @click="closeOverlay('ssh-group-modal')">
          Cancel
        </Button>
        <Button type="submit" variant="primary" :loading="isLoading" :icon="Save">
          {{ props.sshGroupId ? "Update Group" : "Create Group" }}
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
import { Save } from "lucide-vue-next";
import { useSSHStore } from "../../stores/ssh";
import { useOverlay } from "../../composables/useOverlay";
import { SSHGroup } from "../../types/ssh";

// Props
const props = defineProps<{
  sshGroupId: string | null;
}>();

// Store and composables
const sshStore = useSSHStore();
const { closeOverlay } = useOverlay();

// State
const isLoading = ref(false);
const sshGroupForm = ref<InstanceType<typeof Form> | null>(null);
const sshGroup = ref<SSHGroup | null>(null);

// Functions
const loadGroup = async () => {
  if (!props.sshGroupId) return;

  isLoading.value = true;
  try {
    const group = await sshStore.findGroupById(props.sshGroupId);
    sshGroup.value = group as SSHGroup;
  } catch (error) {
    console.error("Error loading SSH group:", error);
  } finally {
    isLoading.value = false;
  }
};

const handleSubmit = async () => {
  const isValid = await sshGroupForm.value?.validate();
  if (!isValid || !sshGroup.value) return;

  isLoading.value = true;
  try {
    if (props.sshGroupId) {
      await sshStore.updateGroup(props.sshGroupId, sshGroup.value);
    } else {
      await sshStore.createGroup(sshGroup.value);
    }
    closeOverlay("ssh-group-modal");
  } catch (error) {
    console.error("Error saving SSH group:", error);
  } finally {
    isLoading.value = false;
  }
};

// Watch for prop changes
watch(
  () => props.sshGroupId,
  (newId) => {
    if (newId) {
      loadGroup();
    } else {
      sshGroup.value = null;
    }
  },
  { immediate: true },
);
</script>
