<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <Modal
    id="about-modal"
    :show-close-button="true"
    title="About Kerminal"
    :icon="Info"
    icon-background="bg-zinc-800"
    icon-color="text-zinc-200"
    size="md"
  >
    <div class="space-y-4">
      <!-- App Header -->
      <div class="text-center pt-1 pb-1">
        <img
          src="../../assets/images/logo_500.png"
          alt="Kerminal Logo"
          class="w-20 h-20 mx-auto mb-2.5 drop-shadow-md"
        />
        <h2 class="text-2xl font-bold text-white tracking-tight">Kerminal</h2>
        <div class="flex items-center justify-center gap-2 mt-1.5">
          <Badge variant="primary" size="sm">v{{ appVersion }}</Badge>
          <Badge variant="gray" size="sm">GPL-3.0-or-later</Badge>
        </div>
        <p class="text-xs text-zinc-400 mt-2 max-w-sm mx-auto leading-relaxed">
          Modern Terminal Emulator & SSH Manager with Multi-Device Sync and Enterprise-Grade Encryption.
        </p>
      </div>

      <!-- Links & Author -->
      <div class="rounded-xl border border-zinc-700/60 bg-zinc-800/40 p-3.5 space-y-2.5">
        <div class="flex items-center justify-between text-xs">
          <div class="flex items-center gap-2 text-zinc-400">
            <User class="w-3.5 h-3.5 text-zinc-400" />
            <span>Author</span>
          </div>
          <button
            type="button"
            class="text-zinc-200 hover:text-white font-medium flex items-center gap-1.5 transition-colors cursor-pointer group"
            @click="openExternal('https://klpod221.com')"
          >
            <span>Bùi Thanh Xuân (klpod221)</span>
            <ExternalLink class="w-3 h-3 text-zinc-500 group-hover:text-zinc-300" />
          </button>
        </div>

        <div class="h-px bg-zinc-700/40"></div>

        <div class="flex items-center justify-between text-xs">
          <div class="flex items-center gap-2 text-zinc-400">
            <Github class="w-3.5 h-3.5 text-zinc-400" />
            <span>Repository</span>
          </div>
          <button
            type="button"
            class="text-zinc-200 hover:text-white font-medium flex items-center gap-1.5 transition-colors cursor-pointer group"
            @click="openExternal('https://github.com/klpod221/kerminal')"
          >
            <span>klpod221/kerminal</span>
            <ExternalLink class="w-3 h-3 text-zinc-500 group-hover:text-zinc-300" />
          </button>
        </div>
      </div>

      <!-- Sponsor / Support Section -->
      <div class="rounded-xl border border-zinc-700/60 bg-linear-to-b from-zinc-800/70 to-zinc-900/70 p-4 space-y-2.5">
        <div class="flex items-center gap-2">
          <Heart class="w-4 h-4 text-rose-400 fill-rose-500/20" />
          <h3 class="text-sm font-semibold text-white">Support Kerminal</h3>
        </div>

        <p class="text-xs text-zinc-400 leading-relaxed">
          If Kerminal improves your daily terminal workflow, consider sponsoring to support continuous updates and new features.
        </p>

        <!-- Premium Sponsor Buttons -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5 pt-1">
          <button
            type="button"
            class="group flex items-center justify-center gap-2 px-3.5 py-2.5 rounded-lg border border-pink-500/30 bg-pink-500/10 hover:bg-pink-500/20 hover:border-pink-500/60 text-white font-medium text-xs transition-all duration-200 cursor-pointer shadow-sm active:scale-98"
            @click="openExternal('https://github.com/sponsors/klpod221')"
          >
            <Github class="w-4 h-4 text-pink-400 group-hover:scale-110 transition-transform" />
            <span class="flex-1 text-left">GitHub Sponsors</span>
            <ExternalLink class="w-3 h-3 text-pink-400/60 group-hover:text-pink-300 shrink-0" />
          </button>

          <button
            type="button"
            class="group flex items-center justify-center gap-2 px-3.5 py-2.5 rounded-lg border border-amber-500/30 bg-amber-500/10 hover:bg-amber-500/20 hover:border-amber-500/60 text-white font-medium text-xs transition-all duration-200 cursor-pointer shadow-sm active:scale-98"
            @click="openExternal('https://www.buymeacoffee.com/klpod221')"
          >
            <Coffee class="w-4 h-4 text-amber-400 group-hover:scale-110 transition-transform" />
            <span class="flex-1 text-left">Buy Me a Coffee</span>
            <ExternalLink class="w-3 h-3 text-amber-400/60 group-hover:text-amber-300 shrink-0" />
          </button>
        </div>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { Info, ExternalLink, Heart, Github, Coffee, User } from "lucide-vue-next";
import { openUrl } from "@tauri-apps/plugin-opener";
import Modal from "../ui/Modal.vue";
import Badge from "../ui/Badge.vue";

declare const __APP_VERSION__: string;
const appVersion = __APP_VERSION__;

const openExternal = async (url: string) => {
  try {
    await openUrl(url);
  } catch (err) {
    console.error("Failed to open external url:", err);
  }
};
</script>
