<template>
  <div class="relative">
    <!-- Setup Modal -->
    <MasterPasswordSetup />

    <!-- Unlock Modal -->
    <MasterPasswordUnlock />

    <!-- <MasterPasswordChange /> -->

    <!-- <MasterPasswordSettings /> -->
  </div>
</template>

<script setup lang="ts">
import MasterPasswordSetup from './MasterPasswordSetup.vue'
import MasterPasswordUnlock from './MasterPasswordUnlock.vue'
</script>
