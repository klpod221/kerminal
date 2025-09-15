<template>
  <Drawer
    id="ssh-profile-drawer"
    title="SSH Profiles"
    position="left"
    :icon="Server"
    icon-background="bg-orange-500/20"
    icon-color="text-orange-400"
  >
    <template #headerAction>
      <!-- Search -->
      <Input
        id="search-ssh-profiles"
        v-model="searchQuery"
        type="text"
        placeholder="Search SSH profiles..."
        class="mb-2"
        :left-icon="Search"
        :helper="false"
      />

      <!-- Filter Toggle -->
      <div class="flex items-center justify-between">
        <Checkbox
          id="favorites-only"
          v-model="showFavoritesOnly"
          label="Favorites only"
          :helper="false"
        />

        <Button
          title="Refresh profiles"
          variant="ghost"
          size="sm"
          :icon="RefreshCw"
        />
      </div>
    </template>

    <!-- Footer -->
    <template #footer>
      <div class="flex justify-between items-center">
        <Button
          variant="ghost"
          size="sm"
          :icon="FolderPlus"
          text="Create Group"
          @click="openManageGroups"
        />

        <Button
          variant="warning"
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
import { ref } from "vue";
import Drawer from "./ui/Drawer.vue";
import Input from "./ui/Input.vue";
import Checkbox from "./ui/Checkbox.vue";
import Button from "./ui/Button.vue";
import { Server, Search, RefreshCw, FolderPlus, Plus } from "lucide-vue-next";

// State
const searchQuery = ref("");
const showFavoritesOnly = ref(false);

// Methods
const openManageGroups = () => {};

const createNewProfile = () => {};
</script>
