<template>
  <div
    class="flex flex-col items-center text-center text-white text-xs font-mono"
  >
    <div class="text-gray-300">{{ currentDate }}</div>
    <div class="text-white font-medium">{{ currentTime }}</div>
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

  // Format time as HH:MM:SS
  currentTime.value = now.toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });

  // Format date as DD/MM/YYYY
  currentDate.value = now.toLocaleDateString("en-GB", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
  });
};

onMounted(() => {
  // Update immediately
  updateDateTime();

  // Update every second
  intervalId = window.setInterval(updateDateTime, 1000);
});

onUnmounted(() => {
  if (intervalId) {
    clearInterval(intervalId);
  }
});
</script>
