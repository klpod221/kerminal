<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <CommandPalette />
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount } from "vue";
import CommandPalette from "./ui/CommandPalette.vue";
import { useOverlay } from "../composables/useOverlay";

const { registerOverlay, unregisterOverlay } = useOverlay();

onMounted(() => {
  registerOverlay({
    id: "command-palette",
    type: "modal",
    title: "Command Palette",
    props: {
      size: "lg",
      showCloseButton: false,
      closeOnBackdrop: true,
      closeOnEsc: true
    }
  });
});

onBeforeUnmount(() => {
  unregisterOverlay("command-palette");
});
</script>
