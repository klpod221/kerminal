<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <Transition name="fade">
    <div
      v-if="confirmation"
      class="fixed inset-0 z-9999 flex items-center justify-center bg-black/75 backdrop-blur-xs p-4"
      @keydown.esc="handleCancel"
    >
      <div
        class="w-full max-w-lg rounded-xl border border-red-500/40 bg-gray-900 p-6 shadow-2xl ring-1 ring-red-500/30 text-gray-100"
      >
        <!-- Header -->
        <div class="flex items-start gap-4">
          <div class="rounded-full bg-red-500/20 p-3 text-red-400">
            <AlertTriangle class="h-7 w-7 animate-pulse" />
          </div>
          <div class="flex-1">
            <h3 class="text-lg font-bold tracking-tight text-red-400">
              Dangerous Command Intercepted
            </h3>
            <p class="mt-1 text-sm text-gray-400">
              {{ confirmation.pattern.name }}
            </p>
          </div>
        </div>

        <!-- Description -->
        <div class="mt-4 rounded-lg bg-red-950/30 border border-red-900/40 p-3 text-xs text-red-300">
          {{ confirmation.pattern.description }}
        </div>

        <!-- Command Preview -->
        <div class="mt-4">
          <label class="block text-xs font-semibold uppercase tracking-wider text-gray-400 mb-1">
            Intercepted Command:
          </label>
          <div
            class="overflow-x-auto rounded-lg bg-black/60 border border-gray-800 p-3 font-mono text-sm text-amber-300 selection:bg-amber-500/30"
          >
            <code>{{ confirmation.command }}</code>
          </div>
        </div>

        <!-- Broadcast Warning (if applicable) -->
        <div
          v-if="confirmation.isBroadcast && confirmation.targets.length > 1"
          class="mt-4 rounded-lg bg-amber-950/40 border border-amber-500/50 p-3 text-xs text-amber-300"
        >
          <div class="flex items-center gap-2 font-bold text-amber-400">
            <Radio class="h-4 w-4 animate-spin" />
            <span>BROADCAST ACTIVE: Will execute on {{ confirmation.targets.length }} terminals simultaneously:</span>
          </div>
          <ul class="mt-2 list-disc list-inside space-y-0.5 text-gray-300">
            <li v-for="target in confirmation.targets" :key="target.id">
              {{ target.title || target.id }}
            </li>
          </ul>
        </div>

        <!-- Actions -->
        <div class="mt-6 flex items-center justify-end gap-3">
          <button
            ref="cancelButtonRef"
            type="button"
            class="rounded-lg bg-gray-800 px-4 py-2 text-sm font-medium text-gray-200 hover:bg-gray-700 transition-colors focus:ring-2 focus:ring-gray-600 focus:outline-none"
            @click="handleCancel"
          >
            Cancel & Abort (Esc)
          </button>
          <button
            type="button"
            class="rounded-lg bg-red-600 px-4 py-2 text-sm font-semibold text-white hover:bg-red-500 transition-colors shadow-lg shadow-red-950/50 focus:ring-2 focus:ring-red-400 focus:outline-none"
            @click="handleConfirm"
          >
            Confirm & Execute
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, ref, watch, nextTick } from "vue";
import { AlertTriangle, Radio } from "lucide-vue-next";
import { useSecurityStore } from "../../stores/security";

const securityStore = useSecurityStore();
const confirmation = computed(() => securityStore.pendingConfirmation);
const cancelButtonRef = ref<HTMLButtonElement | null>(null);

watch(confirmation, async (val) => {
  if (val) {
    await nextTick();
    cancelButtonRef.value?.focus();
  }
});

const handleConfirm = () => {
  securityStore.confirm();
};

const handleCancel = () => {
  securityStore.cancel();
};
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
