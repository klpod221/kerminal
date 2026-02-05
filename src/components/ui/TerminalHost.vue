<template>
  <div ref="hostRef" class="terminal-host" aria-hidden="true" />
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { TerminalRegistry } from "../../core";

const hostRef = ref<HTMLDivElement | null>(null);

onMounted(() => {
  if (hostRef.value) {
    TerminalRegistry.setHostElement(hostRef.value);
  }
});

onBeforeUnmount(() => {
  TerminalRegistry.clear();
});
</script>

<style scoped>
.terminal-host {
  position: fixed;
  left: -9999px;
  top: -9999px;
  width: 1px;
  height: 1px;
  overflow: hidden;
  pointer-events: none;
  visibility: hidden;
}
</style>
