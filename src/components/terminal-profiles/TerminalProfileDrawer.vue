<template>
  <Drawer
    id="terminal-profile-drawer"
    title="Terminal Profiles"
    position="left"
    :icon="Terminal"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
  >
    <template #headerAction>
      <Form>
        <Input
          id="search-terminal-profiles"
          v-model="searchQuery"
          type="text"
          placeholder="Search profiles..."
          :left-icon="Search"
          :helper="false"
        />
      </Form>
    </template>

    <EmptyState
      v-if="!store.profiles.length"
      :icon="Terminal"
      title="No Terminal Profiles"
      description="Create your first terminal profile to get started."
      action-text="Create Profile"
      :action-icon="Plus"
      action-variant="outline"
      @action="createNewProfile"
    />

    <div v-else class="space-y-4 p-4">
      <div v-if="filteredProfiles.length > 0" class="space-y-2">
        <TerminalProfileItem
          v-for="profile in filteredProfiles"
          :key="profile.id"
          :profile="profile"
          @launch="launchProfile"
          @edit="editProfile"
          @delete="deleteProfile"
        />
      </div>

      <div
        v-else
        class="p-3 text-gray-500 text-sm italic text-center border border-dashed border-gray-600 rounded-lg"
      >
        <p>No profiles found matching "{{ searchQuery }}"</p>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end w-full">
        <Button
          variant="primary"
          size="sm"
          :icon="Plus"
          text="New Profile"
          @click="createNewProfile"
        />
      </div>
    </template>
  </Drawer>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import Drawer from "../ui/Drawer.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import EmptyState from "../ui/EmptyState.vue";
import TerminalProfileItem from "./TerminalProfileItem.vue";
import { Terminal, Search, Plus } from "lucide-vue-next";
import { useTerminalProfileStore } from "../../stores/terminalProfile";
import { useWorkspaceStore } from "../../stores/workspace";
import { useOverlay } from "../../composables/useOverlay";
import { caseInsensitiveIncludes } from "../../utils/helpers";
import type { TerminalProfile } from "../../types/terminalProfile";

const store = useTerminalProfileStore();
const workspaceStore = useWorkspaceStore();
const { openOverlay, closeOverlay } = useOverlay();

const searchQuery = ref("");

const filteredProfiles = computed(() => {
  if (!searchQuery.value.trim()) {
    return store.profiles;
  }
  const query = searchQuery.value.trim();
  return store.profiles.filter(
    (p) =>
      caseInsensitiveIncludes(p.name, query) ||
      caseInsensitiveIncludes(p.shell, query),
  );
});

const createNewProfile = () => {
  openOverlay("terminal-profile-modal");
};

const editProfile = (profile: TerminalProfile) => {
  openOverlay("terminal-profile-modal", { profileId: profile.id });
};

const deleteProfile = async (profile: TerminalProfile) => {
  await store.deleteProfile(profile.id);
};

const launchProfile = (profile: TerminalProfile) => {
  const activePanelId = workspaceStore.activePanelId || "panel-1";
  workspaceStore.addTerminalProfileTab(activePanelId, profile);
  closeOverlay("terminal-profile-drawer");
};

onMounted(() => {
  store.loadProfiles();
});
</script>
