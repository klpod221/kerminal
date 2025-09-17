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
  </div>
</template>

<script setup lang="ts">
import MasterPasswordSetup from './MasterPasswordSetup.vue'
import MasterPasswordUnlock from './MasterPasswordUnlock.vue'
import MasterPasswordSettings from './MasterPasswordSettings.vue'
import MasterPasswordChange from './MasterPasswordChange.vue'
</script>
