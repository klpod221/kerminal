<template>
  <div class="flex items-center text-center text-white text-xs">
    <div class="text-gray-300">{{ currentDate }}</div>
    <div class="mx-2 text-gray-500">|</div>
    <div class="text-gray-300">{{ currentTime }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";

const currentTime = ref("");
const currentDate = ref("");
let intervalId: number | null = null;

/**
 * Updates the current time and date display.
 */
const updateDateTime = () => {
  const now = new Date();

  currentTime.value = now.toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });

  currentDate.value = now
    .toLocaleDateString("en-GB", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
    })
    .replace(/\//g, "-");
};

onMounted(() => {
  updateDateTime();

  intervalId = window.setInterval(updateDateTime, 1000);
});

onUnmounted(() => {
  if (intervalId) {
    clearInterval(intervalId);
  }
});
</script>
