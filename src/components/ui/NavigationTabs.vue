<template>
  <div class="border-b border-gray-800">
    <nav class="flex gap-1">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        type="button"
        class="group relative px-4 py-2.5 text-sm font-medium transition-all duration-200 cursor-pointer"
        :class="{
          'text-blue-400': modelValue === tab.id,
          'text-gray-400 hover:text-gray-200': modelValue !== tab.id,
        }"
        @click="$emit('update:modelValue', tab.id)"
      >
        <div class="flex items-center gap-2">
          <component 
            :is="tab.icon" 
            :size="16" 
            class="transition-transform duration-200"
            :class="{
              'text-blue-400': modelValue === tab.id,
              'group-hover:scale-110': modelValue !== tab.id,
            }"
          />
          <span>{{ tab.label }}</span>
        </div>
        
        <!-- Active indicator -->
        <div
          class="absolute bottom-0 left-0 right-0 h-0.5 transition-all duration-200"
          :class="{
            'bg-blue-500': modelValue === tab.id,
            'bg-transparent': modelValue !== tab.id,
          }"
        />
      </button>
    </nav>
  </div>
</template>

<script setup lang="ts">
import type { Component } from "vue";

interface TabItem {
  id: string;
  label: string;
  icon: Component;
}

defineProps<{
  tabs: TabItem[];
  modelValue: string;
}>();

defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();
</script>
