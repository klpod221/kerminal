<template>
  <div class="relative">
    <!-- Setup Modal -->
    <MasterPasswordSetup />

    <!-- Unlock Modal -->
    <MasterPasswordUnlock />

    <MasterPasswordSettings />

    <!-- <MasterPasswordChange /> -->
  </div>
</template>

<script setup lang="ts">
import MasterPasswordSetup from './MasterPasswordSetup.vue'
import MasterPasswordUnlock from './MasterPasswordUnlock.vue'
import MasterPasswordSettings from './MasterPasswordSettings.vue'
</script>
