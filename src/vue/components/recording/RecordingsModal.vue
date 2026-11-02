<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <Modal
    id="recordings-modal"
    title="Session Recordings"
    size="2xl"
    :icon="Video"
  >
    <RecordingsLibrary />
  </Modal>
</template>

<script setup lang="ts">
import { Video } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import RecordingsLibrary from "./RecordingsLibrary.vue";
</script>
