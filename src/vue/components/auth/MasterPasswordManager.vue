<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <div class="relative">
    <!-- Setup Modal -->
    <MasterPasswordSetup />

    <!-- Unlock Modal -->
    <MasterPasswordUnlock />

    <!-- Settings Modal -->
    <MasterPasswordSettings />

    <!-- Change Modal -->
    <MasterPasswordChange />

    <!-- Forgot Password Modal -->
    <ForgotPasswordModal />
  </div>
</template>

<script setup lang="ts">
import MasterPasswordSetup from "./MasterPasswordSetup.vue";
import MasterPasswordUnlock from "./MasterPasswordUnlock.vue";
import MasterPasswordSettings from "./MasterPasswordSettings.vue";
import MasterPasswordChange from "./MasterPasswordChange.vue";
import ForgotPasswordModal from "./ForgotPasswordModal.vue";
</script>
