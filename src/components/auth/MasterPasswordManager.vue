<template>
  <div class="relative">
    <!-- Initial Setup Modal -->
    <InitialSetup />

    <!-- Restore from Sync Modal -->
    <RestoreFromSync />

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
import InitialSetup from "./InitialSetup.vue";
import RestoreFromSync from "./RestoreFromSync.vue";
import MasterPasswordSetup from "./MasterPasswordSetup.vue";
import MasterPasswordUnlock from "./MasterPasswordUnlock.vue";
import MasterPasswordSettings from "./MasterPasswordSettings.vue";
import MasterPasswordChange from "./MasterPasswordChange.vue";
</script>
