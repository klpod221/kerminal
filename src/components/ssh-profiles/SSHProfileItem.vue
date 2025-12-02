<template>
  <Card
    :hover="true"
    no-padding
    custom-class="p-2 cursor-pointer"
    @click="$emit('connect', profile)"
  >
    <div class="flex items-center gap-3">
      <!-- Color indicator -->
      <div class="shrink-0">
        <div
          class="w-1 h-10 rounded-full transition-all duration-200"
          :style="{
            backgroundColor: profile.color || fallbackColor || '#6b7280',
          }"
        />
      </div>

      <!-- Profile info -->
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <h4 class="text-sm font-medium text-white truncate">
            {{ profile.name }}
          </h4>
        </div>
        <div class="text-xs text-gray-500 mt-0.5">
          <code class="font-mono"
            >{{ profile.username }}@{{ profile.host }}:{{ profile.port }}</code
          >
        </div>
      </div>

      <!-- Actions -->
      <div class="shrink-0 flex items-center gap-1">
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
import type { SSHProfile } from "../../types/ssh";
import Card from "../ui/Card.vue";
import Button from "../ui/Button.vue";
import { Edit3, Trash2 } from "lucide-vue-next";
import { showConfirm } from "../../utils/message";

interface Props {
  profile: SSHProfile;
  fallbackColor?: string;
}

interface Emits {
  connect: [profile: SSHProfile];
  edit: [profile: SSHProfile];
  delete: [profile: SSHProfile];
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
</script>
