<template>
  <div class="text-center py-8 px-4">
    <component
      :is="icon"
      :size="iconSize"
      class="mx-auto mb-4"
      :class="iconClass"
    />
    <p class="text-sm text-gray-400 mb-1 font-medium" v-if="title">
      {{ title }}
    </p>
    <p class="text-xs text-gray-500 mb-4" v-if="description">
      {{ description }}
    </p>
    <slot name="actions">
      <Button
        v-if="actionText"
        :variant="actionVariant"
        :size="actionSize"
        :icon="actionIcon"
        @click="$emit('action')"
      >
        {{ actionText }}
      </Button>
    </slot>
  </div>
</template>

<script setup lang="ts">
import Button from './Button.vue';

interface Props {
  icon: any;
  iconSize?: number;
  iconClass?: string;
  title?: string;
  description?: string;
  actionText?: string;
  actionIcon?: any;
  actionVariant?: 'primary' | 'secondary' | 'success' | 'danger' | 'outline' | 'ghost';
  actionSize?: 'sm' | 'md' | 'lg';
}

withDefaults(defineProps<Props>(), {
  iconSize: 48,
  iconClass: 'text-gray-500',
  actionVariant: 'outline',
  actionSize: 'sm',
});

defineEmits<{
  action: [];
}>();
</script>
