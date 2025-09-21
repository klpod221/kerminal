<template>
  <div
    class="group flex items-center justify-between p-3 bg-[#2a2a2a] hover:bg-[#333333] hover:border-gray-500 border border-transparent rounded-lg cursor-pointer transition-all duration-300 transform hover:scale-[1.02] hover:shadow-lg"
    @click="$emit('connect', profile)"
  >
    <div class="flex items-center space-x-3 flex-1 min-w-0">
      <div class="flex-shrink-0">
        <div
          class="w-2 h-2 rounded-full transition-all duration-300 group-hover:w-3 group-hover:h-3"
          :style="{ backgroundColor: profile.color || fallbackColor || '#6b7280' }"
        ></div>
      </div>
      <div class="flex-1 min-w-0">
        <div class="flex items-center space-x-2">
          <p
            class="text-sm font-medium text-white group-hover:text-blue-300 truncate transition-colors duration-300"
          >
            {{ profile.name }}
          </p>
        </div>
        <p
          class="text-xs text-gray-400 group-hover:text-gray-300 truncate transition-colors duration-300"
        >
          {{ profile.username }}@{{ profile.host }}:{{ profile.port }}
        </p>
      </div>
    </div>

    <div
      class="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-all duration-300"
      @click.stop
    >
      <Button
        title="Edit profile"
        variant="ghost"
        size="sm"
        :icon="Edit3"
        @click="$emit('edit', profile)"
      />
      <PopConfirm
        :title="`Delete profile '${profile.name}'?`"
        content="This action cannot be undone."
        placement="bottom"
        @confirm="$emit('delete', profile)"
      >
        <Button
          title="Delete profile"
          variant="ghost"
          size="sm"
          :icon="Trash2"
        />
      </PopConfirm>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SSHProfile } from "../../types/ssh";
import Button from "../ui/Button.vue";
import PopConfirm from "../ui/PopConfirm.vue";
import { Edit3, Trash2 } from "lucide-vue-next";

interface Props {
  profile: SSHProfile;
  fallbackColor?: string;
}

interface Emits {
  connect: [profile: SSHProfile];
  edit: [profile: SSHProfile];
  delete: [profile: SSHProfile];
}

defineProps<Props>();
defineEmits<Emits>();
</script>
