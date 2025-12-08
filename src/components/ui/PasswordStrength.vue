<template>
  <div class="space-y-1">
    <div
      class="flex h-1.5 w-full gap-1 rounded-full overflow-hidden bg-gray-700"
    >
      <div
        v-for="index in 4"
        :key="index"
        class="h-full flex-1 transition-all duration-300"
        :class="getSegmentColor(index)"
      ></div>
    </div>
    <div class="flex justify-between items-center text-xs">
      <span class="text-gray-400">Strength</span>
      <span :class="strengthTextColor" class="font-medium">{{
        strengthText
      }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

interface Props {
  password?: string;
}

const props = withDefaults(defineProps<Props>(), {
  password: "",
});

const score = computed(() => {
  if (!props.password) return 0;

  let s = 0;
  const pwd = props.password;

  // Length check
  if (pwd.length > 6) s += 1;
  if (pwd.length > 10) s += 1;

  // Complexity check
  if (/[A-Z]/.test(pwd)) s += 0.5;
  if (/[0-9]/.test(pwd)) s += 0.5;
  if (/[^A-Za-z0-9]/.test(pwd)) s += 1;

  return Math.min(4, Math.floor(s));
});

const strengthText = computed(() => {
  switch (score.value) {
    case 0:
      return "";
    case 1:
      return "Weak";
    case 2:
      return "Fair";
    case 3:
      return "Good";
    case 4:
      return "Strong";
    default:
      return "";
  }
});

const strengthTextColor = computed(() => {
  switch (score.value) {
    case 1:
      return "text-red-400";
    case 2:
      return "text-yellow-400";
    case 3:
      return "text-blue-400";
    case 4:
      return "text-green-400";
    default:
      return "text-gray-400";
  }
});

const getSegmentColor = (index: number) => {
  if (index > score.value) return "bg-gray-700";

  switch (score.value) {
    case 1:
      return "bg-red-500";
    case 2:
      return "bg-yellow-500";
    case 3:
      return "bg-blue-500";
    case 4:
      return "bg-green-500";
    default:
      return "bg-gray-700";
  }
};
</script>
