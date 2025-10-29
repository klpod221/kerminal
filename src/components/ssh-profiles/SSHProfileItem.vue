<template>
  <div
    class="group relative flex items-center gap-3 p-3.5 bg-[#2a2a2a] hover:bg-[#303030] border border-gray-700 hover:border-gray-600 rounded-lg cursor-pointer transition-all duration-200"
    @click="$emit('connect', profile)"
  >
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
    <div class="flex-1 min-w-0 space-y-1">
      <h4
        class="text-sm font-semibold text-white group-hover:text-blue-300 transition-colors truncate"
      >
        {{ profile.name }}
      </h4>
      <div class="flex items-center gap-2 text-xs text-gray-400">
        <code class="font-mono"
          >{{ profile.username }}@{{ profile.host }}:{{ profile.port }}</code
        >
      </div>
    </div>

    <!-- Action buttons (hover) -->
    <div
      class="flex flex-col items-center gap-1 transition-opacity duration-200 shrink-0"
      :class="isTouch ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'"
      @click.stop
    >
      <Button
        title="Edit profile"
        variant="ghost"
        size="sm"
        :icon="Edit3"
        @click="$emit('edit', profile)"
      />
      <Button
        title="Delete profile"
        variant="ghost"
        size="sm"
        :icon="Trash2"
        class="text-red-400 hover:text-red-300"
        @click="handleDelete"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SSHProfile } from "../../types/ssh";
import Button from "../ui/Button.vue";
import { Edit3, Trash2 } from "lucide-vue-next";
import { useWindowSize } from "../../composables/useWindowSize";

const { isTouch } = useWindowSize();

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

const handleDelete = () => {
  if (
    confirm(`Delete '${props.profile.name}'? This action cannot be undone.`)
  ) {
    emit("delete", props.profile);
  }
};
</script>
