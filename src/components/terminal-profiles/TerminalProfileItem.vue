<template>
  <Card
    :hover="true"
    no-padding
    custom-class="p-2 cursor-pointer"
    @click="$emit('launch', profile)"
  >
    <div class="flex items-center gap-3">
      <!-- Color indicator -->
      <div class="shrink-0">
        <div
          class="w-1 h-10 rounded-full transition-all duration-200"
          :style="{
            backgroundColor: profile.color || '#3b82f6',
          }"
        />
      </div>

      <!-- Profile info -->
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <h4 class="text-sm font-medium text-white truncate">
            {{ profile.name }}
          </h4>
          <span
            v-if="profile.isDefault"
            class="text-xs px-1.5 py-0.5 rounded-full bg-blue-500/20 text-blue-400 font-medium shrink-0"
          >
            Default
          </span>
        </div>
        <div class="text-xs text-gray-500 mt-0.5 flex items-center gap-2">
          <span class="font-mono bg-gray-800 px-1 rounded">{{
            profile.shell
          }}</span>
          <span
            v-if="profile.workingDir"
            class="truncate opacity-75"
            :title="profile.workingDir"
          >
            {{ profile.workingDir }}
          </span>
        </div>
      </div>

      <!-- Actions -->
      <div class="shrink-0 flex items-center gap-1">
        <!-- Set as default button -->
        <Button
          variant="ghost"
          size="sm"
          :icon="profile.isDefault ? StarOff : Star"
          :title="profile.isDefault ? 'Unset default' : 'Set as default'"
          :class="[
            'p-1.5!',
            profile.isDefault
              ? 'text-yellow-400 hover:text-yellow-300 hover:bg-yellow-600/20'
              : 'text-gray-400 hover:text-yellow-400 hover:bg-yellow-600/20',
          ]"
          @click.stop="handleToggleDefault"
        />

        <!-- Edit button -->
        <Button
          variant="ghost"
          size="sm"
          :icon="Edit3"
          title="Edit profile"
          class="p-1.5! text-gray-400 hover:text-blue-400 hover:bg-blue-600/20"
          @click.stop="$emit('edit', profile)"
        />

        <!-- Delete button -->
        <Button
          variant="ghost"
          size="sm"
          :icon="Trash2"
          title="Delete profile"
          class="p-1.5! text-gray-400 hover:text-red-400 hover:bg-red-600/20"
          @click.stop="handleDelete"
        />
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import type { TerminalProfile } from "../../types/terminalProfile";
import Card from "../ui/Card.vue";
import Button from "../ui/Button.vue";
import { Edit3, Trash2, Star, StarOff } from "lucide-vue-next";
import { showConfirm } from "../../utils/message";

interface Props {
  profile: TerminalProfile;
}

interface Emits {
  launch: [profile: TerminalProfile];
  edit: [profile: TerminalProfile];
  delete: [profile: TerminalProfile];
  "toggle-default": [profile: TerminalProfile];
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const handleDelete = async () => {
  const confirmed = await showConfirm(
    "Delete Profile",
    `Delete '${props.profile.name}'? This action cannot be undone.`,
  );
  if (confirmed) {
    emit("delete", props.profile);
  }
};

const handleToggleDefault = () => {
  emit("toggle-default", props.profile);
};
</script>
