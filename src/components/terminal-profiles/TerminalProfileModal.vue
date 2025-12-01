<template>
  <Modal
    id="terminal-profile-modal"
    :title="profileId ? 'Edit Terminal Profile' : 'Create Terminal Profile'"
    size="md"
  >
    <Form ref="profileForm" @submit="handleSubmit">
      <Input
        id="profile-name"
        v-model="profile.name"
        label="Profile Name"
        placeholder="My Terminal"
        rules="required|min:3|max:50"
        :autofocus="true"
      />

      <Input
        id="profile-shell"
        v-model="profile.shell"
        label="Shell"
        placeholder="/bin/bash or /bin/zsh"
        rules="required"
      />

      <Input
        id="profile-working-dir"
        v-model="profile.workingDir"
        label="Working Directory (Optional)"
        placeholder="/home/user/projects"
      />

      <Collapsible
        title="Environment Variables"
        subtitle="Set custom environment variables"
        :default-expanded="false"
      >
        <div v-for="(_item, index) in envVars" :key="index" class="flex gap-2 mb-2">
          <Input
            :id="`env-key-${index}`"
            v-model="envVars[index].key"
            placeholder="KEY"
            class="flex-1"
          />
          <Input
            :id="`env-val-${index}`"
            v-model="envVars[index].value"
            placeholder="VALUE"
            class="flex-1"
          />
          <Button
            type="button"
            variant="ghost"
            :icon="Trash2"
            @click="removeEnvVar(index)"
          />
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          :icon="Plus"
          @click="addEnvVar"
        >
          Add Variable
        </Button>
      </Collapsible>

      <ColorPicker
        id="profile-color"
        v-model="profile.color"
        label="Profile Color"
        placeholder="Pick a color for the tab"
      />
    </Form>

    <template #footer>
      <div class="flex justify-between w-full">
        <Button
          type="button"
          variant="secondary"
          @click="closeOverlay('terminal-profile-modal')"
        >
          Cancel
        </Button>
        <Button
          type="submit"
          variant="primary"
          :icon="Save"
          @click="handleSubmit"
        >
          {{ profileId ? "Update Profile" : "Create Profile" }}
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
import Button from "../ui/Button.vue";
import Collapsible from "../ui/Collapsible.vue";
import ColorPicker from "../ui/ColorPicker.vue";
import { message } from "../../utils/message";
import { Save, Plus, Trash2 } from "lucide-vue-next";
import { useTerminalProfileStore } from "../../stores/terminalProfile";
import { useOverlay } from "../../composables/useOverlay";
import type { CreateTerminalProfileRequest, UpdateTerminalProfileRequest } from "../../types/terminalProfile";

const props = defineProps<{
  profileId?: string | null;
}>();

const store = useTerminalProfileStore();
const { closeOverlay, getOverlayProp } = useOverlay();

const profileId = getOverlayProp("terminal-profile-modal", "profileId", props.profileId, null);

const profileForm = ref<InstanceType<typeof Form> | null>(null);

const profile = ref({
  name: "",
  shell: "",
  workingDir: "",
  color: "#3b82f6",
});

const envVars = ref<{ key: string; value: string }[]>([]);

const loadProfile = () => {
  if (!profileId.value) return;

  const existing = store.getProfile(profileId.value);
  if (existing) {
    profile.value = {
      name: existing.name,
      shell: existing.shell,
      workingDir: existing.workingDir || "",
      color: existing.color || "#3b82f6",
    };
    if (existing.env) {
      envVars.value = Object.entries(existing.env).map(([key, value]) => ({ key, value }));
    } else {
      envVars.value = [];
    }
  }
};

const addEnvVar = () => {
  envVars.value.push({ key: "", value: "" });
};

const removeEnvVar = (index: number) => {
  envVars.value.splice(index, 1);
};

const handleSubmit = async () => {
  const isValid = await profileForm.value?.validate();
  if (!isValid) return;

  const env: Record<string, string> = {};
  envVars.value.forEach((item) => {
    if (item.key) {
      env[item.key] = item.value;
    }
  });

  const request = {
    ...profile.value,
    env: Object.keys(env).length > 0 ? env : undefined,
    workingDir: profile.value.workingDir || undefined,
  };

  if (profileId.value) {
    store.updateProfile(profileId.value, request as UpdateTerminalProfileRequest);
    message.success("Terminal profile updated");
  } else {
    store.createProfile(request as CreateTerminalProfileRequest);
    message.success("Terminal profile created");
  }

  closeOverlay("terminal-profile-modal");
};

watch(
  () => profileId.value,
  (newId) => {
    if (newId) {
      loadProfile();
    } else {
      profile.value = {
        name: "",
        shell: "/bin/bash",
        workingDir: "",
        color: "#3b82f6",
      };
      envVars.value = [];
    }
  },
  { immediate: true }
);
</script>
