<template>
  <span
    :class="[
      'inline-flex items-center rounded-full font-medium transition-all duration-200',
      sizeClasses[size],
      variantClasses[variant],
      dot && 'pl-1.5',
      customClass,
    ]"
  >
    <span
      v-if="dot"
      :class="['w-1.5 h-1.5 rounded-full mr-1.5', dotColorClasses[variant]]"
    />
    <slot />
  </span>
</template>

<script setup lang="ts">
interface BadgeProps {
  variant?:
    | "default"
    | "primary"
    | "success"
    | "warning"
    | "danger"
    | "info"
    | "gray"
    | "outline";
  size?: "xs" | "sm" | "md";
  dot?: boolean;
  customClass?: string;
}

withDefaults(defineProps<BadgeProps>(), {
  variant: "default",
  size: "sm",
  dot: false,
});

const sizeClasses = {
  xs: "px-1.5 py-0.5 text-[10px]",
  sm: "px-2 py-0.5 text-xs",
  md: "px-2.5 py-1 text-sm",
};

const variantClasses = {
  default: "bg-gray-700 text-gray-300",
  primary: "bg-blue-900/30 text-blue-300 border border-blue-800/50",
  success: "bg-green-900/30 text-green-300 border border-green-800/50",
  warning: "bg-yellow-900/30 text-yellow-300 border border-yellow-800/50",
  danger: "bg-red-900/30 text-red-300 border border-red-800/50",
  info: "bg-cyan-900/30 text-cyan-300 border border-cyan-800/50",
  gray: "bg-gray-800 text-gray-300",
  outline: "border border-gray-600 text-gray-300 bg-transparent",
};

const dotColorClasses = {
  default: "bg-gray-400",
  primary: "bg-blue-400",
  success: "bg-green-400",
  warning: "bg-yellow-400",
  danger: "bg-red-400",
  info: "bg-cyan-400",
  gray: "bg-gray-400",
  outline: "bg-gray-400",
};
</script>
