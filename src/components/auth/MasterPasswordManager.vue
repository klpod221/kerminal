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

    <!-- Password Confirm Modal -->
    <PasswordConfirmModal />

    <!-- Reset Confirm Modal -->
    <ResetConfirmModal />
  </div>
</template>

<script setup lang="ts">
import MasterPasswordSetup from "./MasterPasswordSetup.vue";
import MasterPasswordUnlock from "./MasterPasswordUnlock.vue";
import MasterPasswordSettings from "./MasterPasswordSettings.vue";
import MasterPasswordChange from "./MasterPasswordChange.vue";
import ForgotPasswordModal from "./ForgotPasswordModal.vue";
import PasswordConfirmModal from "./PasswordConfirmModal.vue";
import ResetConfirmModal from "./ResetConfirmModal.vue";
</script>
