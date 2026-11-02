<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <div class="h-full flex flex-col bg-bg-secondary text-white dashboard-container overflow-hidden">
    <RenderOptimizationLayer
      v-if="isOptimizingDisplay"
      :status="integrityStatus"
      @close="isOptimizingDisplay = false"
    />

    <!-- Fixed Top Operational Header Bar -->
    <header
      class="shrink-0 bg-bg-secondary/95 backdrop-blur-md border-b border-zinc-800 z-10"
      :class="isMobile ? 'px-3 py-2.5' : 'px-6 py-3'"
    >
      <div class="max-w-7xl mx-auto flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
        <div class="flex items-center gap-3">
          <img
            src="../assets/images/logo_500.png"
            alt="Kerminal Logo"
            class="w-8 h-8 object-contain cursor-pointer transition-transform hover:scale-105 shrink-0"
            @click="handleForceRefresh"
            title="Click to refresh system status"
          />
          <div>
            <div class="flex items-center gap-2">
              <h1 class="text-base font-bold tracking-tight text-white leading-none">Kerminal</h1>
              <Badge variant="primary" size="sm">v{{ appVersion }}</Badge>
              <Badge variant="gray" size="sm">GPL-3.0</Badge>
            </div>
            <p class="text-2xs text-zinc-400 mt-0.5">
              Modern Terminal Emulator & SSH Manager with Multi-Device Sync
            </p>
          </div>
        </div>

        <!-- Quick Action Buttons -->
        <div class="flex items-center flex-wrap gap-2">
          <Button
            variant="primary"
            size="sm"
            :icon="Terminal"
            @click="createLocalTerminal"
          >
            New Terminal
          </Button>

          <Button
            variant="secondary"
            size="sm"
            :icon="Server"
            @click="openOverlay('ssh-profile-drawer')"
          >
            SSH Profiles
          </Button>

          <Button
            variant="secondary"
            size="sm"
            :icon="FolderOpen"
            @click="viewState.setActiveView('sftp')"
          >
            SFTP Browser
          </Button>

          <Button
            variant="ghost"
            size="sm"
            :icon="Info"
            title="About Kerminal & Sponsors"
            @click="openOverlay('about-modal')"
          >
            About
          </Button>
        </div>
      </div>
    </header>

    <!-- Scrollable Content -->
    <main
      class="flex-1 overflow-y-auto"
      :class="isMobile ? 'p-3' : 'p-6'"
    >
      <div class="max-w-7xl mx-auto space-y-6">

      <!-- Quick Metrics Bar (4 Stat Cards) -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4">
        <!-- Metric: SSH Servers -->
        <div
          class="rounded-xl border border-zinc-700/60 bg-linear-to-br from-zinc-800 to-border-dark p-4 cursor-pointer hover:border-zinc-500 transition-colors"
          @click="openOverlay('ssh-profile-drawer')"
        >
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-medium text-zinc-400 uppercase tracking-wider">SSH Servers</span>
            <div class="p-1.5 rounded-lg bg-blue-500/10 text-blue-400">
              <Server class="w-4 h-4" />
            </div>
          </div>
          <div class="text-2xl font-bold text-white">{{ sshStore.profiles.length }}</div>
          <div class="text-2xs text-zinc-400 mt-1 flex items-center gap-1.5">
            <span
              class="w-1.5 h-1.5 rounded-full"
              :class="activeSSHSessionsCount > 0 ? 'bg-emerald-400 animate-pulse' : 'bg-zinc-500'"
            ></span>
            <span>{{ activeSSHSessionsCount }} connected in workspace</span>
          </div>
        </div>

        <!-- Metric: SSH Tunnels -->
        <div
          class="rounded-xl border border-zinc-700/60 bg-linear-to-br from-zinc-800 to-border-dark p-4 cursor-pointer hover:border-zinc-500 transition-colors"
          @click="openOverlay('tunnel-manager-modal')"
        >
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-medium text-zinc-400 uppercase tracking-wider">SSH Tunnels</span>
            <div class="p-1.5 rounded-lg bg-emerald-500/10 text-emerald-400">
              <Route class="w-4 h-4" />
            </div>
          </div>
          <div class="text-2xl font-bold text-white">
            {{ tunnelStore.activeTunnelCount }}
            <span class="text-sm font-normal text-zinc-400">/ {{ tunnelStore.tunnelCount }}</span>
          </div>
          <div class="text-2xs text-zinc-400 mt-1 flex items-center gap-1.5">
            <span
              class="w-1.5 h-1.5 rounded-full"
              :class="tunnelStore.activeTunnelCount > 0 ? 'bg-emerald-400' : 'bg-zinc-500'"
            ></span>
            <span>{{ tunnelStore.activeTunnelCount > 0 ? 'Forwarding traffic' : 'All tunnels idle' }}</span>
          </div>
        </div>

        <!-- Metric: Multi-Device Sync -->
        <div
          class="rounded-xl border border-zinc-700/60 bg-linear-to-br from-zinc-800 to-border-dark p-4"
        >
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-medium text-zinc-400 uppercase tracking-wider">Cloud Sync</span>
            <div class="p-1.5 rounded-lg bg-purple-500/10 text-purple-400">
              <Cloud class="w-4 h-4" />
            </div>
          </div>
          <div class="flex items-center justify-between">
            <div class="text-2xl font-bold text-white">{{ syncStatusText }}</div>
            <Button
              variant="ghost"
              size="sm"
              :icon="RefreshCw"
              :loading="syncStore.isSyncing"
              title="Sync now"
              class="p-1.5! text-zinc-400 hover:text-white"
              @click.stop="handleTriggerSync"
            />
          </div>
          <div class="text-2xs text-zinc-400 mt-1 truncate">
            {{ lastSyncText }}
          </div>
        </div>

        <!-- Metric: Saved Snippets -->
        <div
          class="rounded-xl border border-zinc-700/60 bg-linear-to-br from-zinc-800 to-border-dark p-4 cursor-pointer hover:border-zinc-500 transition-colors"
          @click="openOverlay('saved-command-drawer')"
        >
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-medium text-zinc-400 uppercase tracking-wider">Saved Commands</span>
            <div class="p-1.5 rounded-lg bg-amber-500/10 text-amber-400">
              <FileCode class="w-4 h-4" />
            </div>
          </div>
          <div class="text-2xl font-bold text-white">{{ savedCommandStore.commandCount }}</div>
          <div class="text-2xs text-zinc-400 mt-1">
            {{ savedCommandStore.favoriteCount }} favorites starred
          </div>
        </div>
      </div>

      <!-- Main Operational Grid: 2 Columns -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 items-start">
        <!-- LEFT COLUMN: Quick Connect & Snippets -->
        <div class="space-y-6">
          <!-- Quick Connect SSH Card -->
          <Card
            title="Quick Connect"
            :icon="Zap"
            icon-background="bg-zinc-800"
            icon-color="text-zinc-200"
            custom-class="border-zinc-700/60"
          >
            <template #icon>
              <Zap class="w-5 h-5 text-amber-400" />
            </template>
            <template #header>
              <div>
                <h3 class="text-base font-semibold text-white">Quick Connect</h3>
                <p class="text-xs text-zinc-400">
                  Instant 1-click launch for remote SSH profiles and SFTP.
                </p>
              </div>
            </template>
            <template #action>
              <Button
                variant="ghost"
                size="sm"
                :icon="Plus"
                @click="openOverlay('ssh-profile-modal')"
              >
                Add
              </Button>
            </template>

            <!-- Profiles List -->
            <div v-if="sshStore.profiles.length === 0" class="py-6 text-center">
              <Server class="w-8 h-8 text-zinc-600 mx-auto mb-2" />
              <p class="text-sm font-medium text-zinc-300">No SSH Profiles</p>
              <p class="text-xs text-zinc-500 mt-1">
                Configure your first remote server to enable 1-click connections.
              </p>
              <Button
                variant="secondary"
                size="sm"
                :icon="Plus"
                class="mt-3 mx-auto"
                @click="openOverlay('ssh-profile-modal')"
              >
                Create Profile
              </Button>
            </div>

            <div v-else class="space-y-2.5">
              <div
                v-for="profile in displayProfiles"
                :key="profile.id"
                class="flex items-center justify-between p-3 rounded-lg border border-zinc-700/60 bg-zinc-800/40 hover:bg-zinc-800/80 transition-colors group"
              >
                <div class="flex items-center gap-3 min-w-0 flex-1 mr-3">
                  <div class="p-2 rounded-lg bg-zinc-800 text-zinc-300 shrink-0">
                    <Server class="w-4 h-4" />
                  </div>
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <span class="text-sm font-medium text-white truncate">{{ profile.name }}</span>
                      <span
                        v-if="profile.port && profile.port !== 22"
                        class="text-2xs font-mono text-zinc-500 bg-zinc-800/80 px-1 rounded"
                      >
                        :{{ profile.port }}
                      </span>
                    </div>
                    <div class="text-xs text-zinc-400 font-mono truncate">
                      {{ profile.username }}@{{ profile.host }}
                    </div>
                  </div>
                </div>

                <div class="flex items-center gap-1.5 shrink-0">
                  <Button
                    variant="ghost"
                    size="sm"
                    :icon="FolderOpen"
                    title="Open in SFTP"
                    class="p-2! text-zinc-400 hover:text-white"
                    @click="openSFTPForProfile(profile)"
                  />
                  <Button
                    variant="secondary"
                    size="sm"
                    :icon="Terminal"
                    class="text-xs!"
                    @click="connectToProfile(profile)"
                  >
                    Connect
                  </Button>
                </div>
              </div>

              <div
                v-if="sshStore.profiles.length > displayProfiles.length"
                class="pt-1 text-center"
              >
                <button
                  type="button"
                  class="text-xs text-zinc-400 hover:text-white transition-colors cursor-pointer"
                  @click="openOverlay('ssh-profile-drawer')"
                >
                  View all {{ sshStore.profiles.length }} profiles →
                </button>
              </div>
            </div>
          </Card>

          <!-- Favorite Commands Card -->
          <Card
            title="Frequent Commands"
            :icon="FileCode"
            icon-background="bg-zinc-800"
            icon-color="text-zinc-200"
            custom-class="border-zinc-700/60"
          >
            <template #icon>
              <FileCode class="w-5 h-5 text-zinc-200" />
            </template>
            <template #header>
              <div>
                <h3 class="text-base font-semibold text-white">Frequent Commands</h3>
                <p class="text-xs text-zinc-400">
                  Quick snippets to run in active terminal or copy to clipboard.
                </p>
              </div>
            </template>
            <template #action>
              <Button
                variant="ghost"
                size="sm"
                @click="openOverlay('saved-command-drawer')"
              >
                All ({{ savedCommandStore.commandCount }})
              </Button>
            </template>

            <div v-if="displayCommands.length === 0" class="py-6 text-center">
              <FileCode class="w-8 h-8 text-zinc-600 mx-auto mb-2" />
              <p class="text-sm font-medium text-zinc-300">No Saved Commands</p>
              <p class="text-xs text-zinc-500 mt-1">
                Save shell snippets and scripts for instant one-click execution.
              </p>
              <Button
                variant="secondary"
                size="sm"
                :icon="Plus"
                class="mt-3 mx-auto"
                @click="openOverlay('saved-command-drawer')"
              >
                Add Command
              </Button>
            </div>

            <div v-else class="space-y-2">
              <div
                v-for="cmd in displayCommands"
                :key="cmd.id"
                class="flex items-center justify-between p-2.5 rounded-lg border border-zinc-700/60 bg-zinc-800/40 hover:bg-zinc-800/70 transition-colors"
              >
                <div class="min-w-0 flex-1 mr-3">
                  <div class="text-xs font-semibold text-white truncate">{{ cmd.name }}</div>
                  <div class="text-2xs font-mono text-zinc-400 bg-zinc-900/60 px-2 py-0.5 rounded mt-1 truncate">
                    {{ cmd.command }}
                  </div>
                </div>

                <div class="flex items-center gap-1 shrink-0">
                  <Button
                    variant="ghost"
                    size="sm"
                    :icon="copiedCommandId === cmd.id ? Check : Copy"
                    title="Copy command"
                    class="p-1.5! text-zinc-400 hover:text-white"
                    @click="handleCopyCommand(cmd)"
                  />
                  <Button
                    variant="secondary"
                    size="sm"
                    :icon="Play"
                    title="Run in active terminal"
                    class="p-1.5! text-zinc-200"
                    @click="handleRunCommand(cmd)"
                  />
                </div>
              </div>
            </div>
          </Card>
        </div>

        <!-- RIGHT COLUMN: Infrastructure, Tunnels & Sessions -->
        <div class="space-y-6">
          <!-- Active SSH Tunnels Card -->
          <Card
            title="SSH Tunnels"
            :icon="Route"
            icon-background="bg-zinc-800"
            icon-color="text-zinc-200"
            custom-class="border-zinc-700/60"
          >
            <template #icon>
              <Route class="w-5 h-5 text-emerald-400" />
            </template>
            <template #header>
              <div>
                <h3 class="text-base font-semibold text-white">SSH Tunnels</h3>
                <p class="text-xs text-zinc-400">
                  Local and remote port forwardings managed in background.
                </p>
              </div>
            </template>
            <template #action>
              <Button
                variant="ghost"
                size="sm"
                :icon="Plus"
                @click="openOverlay('tunnel-modal')"
              >
                New
              </Button>
            </template>

            <div v-if="displayTunnels.length === 0" class="py-6 text-center">
              <Route class="w-8 h-8 text-zinc-600 mx-auto mb-2" />
              <p class="text-sm font-medium text-zinc-300">No Configured Tunnels</p>
              <p class="text-xs text-zinc-500 mt-1">
                Forward ports securely through SSH tunnels.
              </p>
              <Button
                variant="secondary"
                size="sm"
                :icon="Plus"
                class="mt-3 mx-auto"
                @click="openOverlay('tunnel-modal')"
              >
                Create Tunnel
              </Button>
            </div>

            <div v-else class="space-y-2.5">
              <div
                v-for="tunnel in displayTunnels"
                :key="tunnel.id"
                class="flex items-center justify-between p-3 rounded-lg border border-zinc-700/60 bg-zinc-800/40 hover:bg-zinc-800/70 transition-colors"
              >
                <div class="min-w-0 flex-1 mr-3">
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-medium text-white truncate">{{ tunnel.name }}</span>
                    <Badge
                      :variant="tunnel.status === 'running' ? 'success' : tunnel.status === 'error' ? 'danger' : 'gray'"
                      size="sm"
                    >
                      {{ tunnel.status }}
                    </Badge>
                  </div>
                  <div class="text-xs font-mono text-zinc-400 mt-0.5 truncate">
                    :{{ tunnel.localPort }} → {{ tunnel.remoteHost }}:{{ tunnel.remotePort }}
                  </div>
                </div>

                <div class="flex items-center gap-1.5 shrink-0">
                  <Button
                    :variant="tunnel.status === 'running' ? 'outline' : 'secondary'"
                    size="sm"
                    :loading="togglingTunnelId === tunnel.id"
                    class="text-xs!"
                    @click="handleToggleTunnel(tunnel)"
                  >
                    {{ tunnel.status === "running" ? "Stop" : "Start" }}
                  </Button>
                </div>
              </div>

              <div
                v-if="tunnelStore.tunnels.length > displayTunnels.length"
                class="pt-1 text-center"
              >
                <button
                  type="button"
                  class="text-xs text-zinc-400 hover:text-white transition-colors cursor-pointer"
                  @click="openOverlay('tunnel-manager-modal')"
                >
                  Manage all {{ tunnelStore.tunnels.length }} tunnels →
                </button>
              </div>
            </div>
          </Card>

          <!-- Active Workspace Sessions Card -->
          <Card
            title="Workspace Sessions"
            :icon="LayoutGrid"
            icon-background="bg-zinc-800"
            icon-color="text-zinc-200"
            custom-class="border-zinc-700/60"
          >
            <template #icon>
              <LayoutGrid class="w-5 h-5 text-zinc-200" />
            </template>
            <template #header>
              <div>
                <h3 class="text-base font-semibold text-white">Active Sessions</h3>
                <p class="text-xs text-zinc-400">
                  Currently open terminal and SSH tabs in your workspace.
                </p>
              </div>
            </template>
            <template #action>
              <Button
                variant="ghost"
                size="sm"
                @click="viewState.setActiveView('workspace')"
              >
                Go to Workspace
              </Button>
            </template>

            <div v-if="activeSessions.length === 0" class="py-5 text-center">
              <Terminal class="w-7 h-7 text-zinc-600 mx-auto mb-2" />
              <p class="text-xs text-zinc-400">
                No active tabs open. Launch a terminal or SSH session to begin.
              </p>
              <Button
                variant="secondary"
                size="sm"
                :icon="Terminal"
                class="mt-3 mx-auto"
                @click="createLocalTerminal"
              >
                Open Terminal
              </Button>
            </div>

            <div v-else class="space-y-2">
              <div
                v-for="session in activeSessions"
                :key="session.id"
                class="flex items-center justify-between p-2.5 rounded-lg border border-zinc-700/60 bg-zinc-800/40 hover:bg-zinc-800/80 transition-colors cursor-pointer"
                @click="jumpToSession(session.panelId, session.id)"
              >
                <div class="flex items-center gap-2.5 min-w-0 flex-1 mr-2">
                  <component
                    :is="session.isSSH ? Server : Terminal"
                    class="w-4 h-4 text-zinc-400 shrink-0"
                  />
                  <span class="text-xs font-medium text-white truncate">{{ session.title }}</span>
                </div>

                <div class="flex items-center gap-2 shrink-0">
                  <Badge :variant="session.isSSH ? 'primary' : 'gray'" size="sm">
                    {{ session.isSSH ? "SSH" : "Local" }}
                  </Badge>
                  <span
                    v-if="session.isActive"
                    class="w-2 h-2 rounded-full bg-emerald-400"
                    title="Active tab"
                  ></span>
                </div>
              </div>
            </div>
          </Card>

          <!-- Foldable Local Host Metrics Section -->
          <div class="rounded-xl border border-zinc-700/60 bg-linear-to-br from-zinc-800 to-border-dark overflow-hidden">
            <button
              type="button"
              class="w-full flex items-center justify-between p-4 bg-transparent text-left cursor-pointer hover:bg-zinc-800/50 transition-colors"
              @click="toggleHostSpecs"
            >
              <div class="flex items-center gap-3">
                <div class="p-2 rounded-lg bg-zinc-800 text-zinc-300">
                  <Cpu class="w-5 h-5 text-zinc-400" />
                </div>
                <div>
                  <h3 class="text-base font-semibold text-white">Local Host Specifications</h3>
                  <p class="text-xs text-zinc-400">
                    {{ isHostSpecsOpen ? 'Live CPU, Memory & Storage telemetry' : 'Click to inspect local hardware metrics' }}
                  </p>
                </div>
              </div>

              <div class="flex items-center gap-2 text-zinc-400">
                <div
                  v-if="loading && isHostSpecsOpen"
                  class="animate-spin rounded-full h-3.5 w-3.5 border-b-2 border-cyan-400"
                ></div>
                <component :is="isHostSpecsOpen ? ChevronUp : ChevronDown" class="w-4 h-4" />
              </div>
            </button>

            <!-- Expanded Telemetry Cards -->
            <div v-if="isHostSpecsOpen" class="p-4 pt-0 border-t border-zinc-800 space-y-3">
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5 pt-3">
                <!-- OS / Kernel -->
                <div class="p-2.5 rounded-lg border border-zinc-700/50 bg-zinc-900/40 space-y-1">
                  <span class="text-[10px] font-semibold text-zinc-500 uppercase tracking-wider">Operating System</span>
                  <div class="text-xs font-medium text-white truncate">{{ systemInfo.os_version || systemInfo.platform || "Linux" }}</div>
                  <div class="text-[10px] text-zinc-400 font-mono truncate">{{ systemInfo.platform }} {{ systemInfo.cpu_arch }}</div>
                  <div class="text-[10px] text-zinc-500">Uptime: {{ formatUptime(systemInfo.uptime || 0) }}</div>
                </div>

                <!-- CPU -->
                <div class="p-2.5 rounded-lg border border-zinc-700/50 bg-zinc-900/40 space-y-1">
                  <span class="text-[10px] font-semibold text-zinc-500 uppercase tracking-wider">Processor</span>
                  <div class="text-xs font-medium text-white truncate">
                    {{ systemInfo.cpus?.[0]?.model ? truncateText(systemInfo.cpus[0].model, 25) : "CPU" }}
                  </div>
                  <div class="text-[10px] text-zinc-400">
                    Cores: {{ systemInfo.cpu_info || systemInfo.cpus?.length || "N/A" }}
                  </div>
                  <div class="text-[10px] text-zinc-400">
                    Usage: {{ calculatedCpuUsage }}
                  </div>
                </div>

                <!-- Memory -->
                <div class="p-2.5 rounded-lg border border-zinc-700/50 bg-zinc-900/40 space-y-1">
                  <span class="text-[10px] font-semibold text-zinc-500 uppercase tracking-wider">Memory</span>
                  <div class="text-xs font-medium text-white">
                    {{ formatBytes(systemInfo.used_memory ?? (systemInfo.total_memory - systemInfo.free_memory)) }} / {{ formatBytes(systemInfo.total_memory) }}
                  </div>
                  <div class="text-[10px] text-zinc-400">
                    Available: {{ formatBytes(systemInfo.available_memory ?? systemInfo.free_memory) }}
                  </div>
                  <div class="text-[10px] text-zinc-500 font-mono">
                    Load: {{ systemInfo.load_average?.[0]?.toFixed(2) || "0.00" }} / {{ systemInfo.load_average?.[1]?.toFixed(2) || "0.00" }}
                  </div>
                </div>

                <!-- Storage -->
                <div class="p-2.5 rounded-lg border border-zinc-700/50 bg-zinc-900/40 space-y-1">
                  <span class="text-[10px] font-semibold text-zinc-500 uppercase tracking-wider">Storage</span>
                  <template v-if="systemInfo.disks_info && systemInfo.disks_info.length > 0">
                    <div class="text-xs font-medium text-white truncate">
                      {{ systemInfo.disks_info[0].mount_point }}: {{ formatBytes(systemInfo.disks_info[0].available_space) }} free
                    </div>
                    <div class="text-[10px] text-zinc-400">
                      Total: {{ formatBytes(systemInfo.disks_info[0].total_space) }} ({{ systemInfo.disks_info[0].file_system }})
                    </div>
                  </template>
                  <template v-else>
                    <div class="text-[10px] text-zinc-400">Local disks info unavailable</div>
                  </template>
                </div>
              </div>

              <!-- Footer with refresh info -->
              <div class="flex items-center justify-between text-[10px] text-zinc-500 pt-0.5">
                <span>Polling every 5s while expanded</span>
                <span v-if="lastUpdated">Updated: {{ formatLastUpdated(lastUpdated) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Dashboard Footer -->
      <footer class="mt-8 pt-4 pb-2 border-t border-zinc-800/80 flex items-center justify-between text-xs text-text-muted">
        <button
          type="button"
          class="group inline-flex items-center gap-1.5 text-xs text-zinc-500 hover:text-zinc-300 transition-colors cursor-pointer py-1 px-2 rounded-md hover:bg-zinc-800/40"
          @click="openOverlay('about-modal')"
        >
          <span>Made with</span>
          <Heart class="w-3.5 h-3.5 text-rose-500 fill-rose-500 transition-transform group-hover:scale-125 duration-200" />
          <span>by</span>
          <span class="text-zinc-400 group-hover:text-white font-medium underline-offset-4 group-hover:underline">klpod221</span>
        </button>
        <div class="flex items-center gap-3 text-[11px] text-zinc-500 font-mono">
          <span>Kerminal v2.7.0</span>
        </div>
      </footer>
    </div>
  </main>
</div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import Card from "./ui/Card.vue";
import Button from "./ui/Button.vue";
import Badge from "./ui/Badge.vue";
import RenderOptimizationLayer from "./ui/RenderOptimizationLayer.vue";

import { useSSHStore } from "../stores/ssh";
import { useTunnelStore } from "../stores/tunnel";
import { useSyncStore } from "../stores/sync";
import { useSavedCommandStore } from "../stores/savedCommand";
import { useWorkspaceStore } from "../stores/workspace";
import { useViewStateStore } from "../stores/viewState";
import { useSettingsStore } from "../stores/settings";
import { useOverlay } from "../composables/useOverlay";
import { useWindowSize } from "../composables/useWindowSize";

import type { SSHProfile } from "../types/ssh";
import type { SavedCommand } from "../types/savedCommand";
import type { TunnelWithStatus } from "../types/tunnel";

import {
  Zap,
  Server,
  Route,
  Cloud,
  FileCode,
  Terminal,
  FolderOpen,
  Play,
  Copy,
  Plus,
  RefreshCw,
  LayoutGrid,
  Cpu,
  ChevronDown,
  ChevronUp,
  Info,
  Check,
  Heart,
} from "lucide-vue-next";

import {
  formatBytes,
  formatUptime,
  formatPercentage,
} from "../utils/formatter";
import { truncateText } from "../utils/helpers";
import { message } from "../utils/message";
import { writeToTerminal } from "../services/terminal";
import {
  verifySystemIntegrity,
  type SystemIntegrityStatus,
} from "../services/dashboard";

declare const __APP_VERSION__: string;
const appVersion = __APP_VERSION__;

const { isMobile } = useWindowSize();
const { openOverlay } = useOverlay();

const sshStore = useSSHStore();
const tunnelStore = useTunnelStore();
const syncStore = useSyncStore();
const savedCommandStore = useSavedCommandStore();
const workspaceStore = useWorkspaceStore();
const viewState = useViewStateStore();
const settingsStore = useSettingsStore();

// --- Data Loading on Mount ---
onMounted(async () => {
  try {
    await Promise.allSettled([
      sshStore.loadProfiles(),
      tunnelStore.loadTunnels(),
      savedCommandStore.loadCommands(),
      syncStore.loadDatabases(),
    ]);

    const targetDb =
      syncStore.currentDatabase ||
      syncStore.databases.find((d) => d.isActive) ||
      syncStore.databases[0];
    if (targetDb?.id) {
      if (!syncStore.currentDatabaseId) {
        syncStore.setCurrentDatabase(targetDb.id);
      }
      await Promise.allSettled([
        syncStore.loadSyncStatus(targetDb.id),
        syncStore.loadSyncLogs(targetDb.id, 5),
      ]);
    }
  } catch (err) {
    console.warn("Error preloading dashboard stores:", err);
  }
});

// --- Active Sessions in Workspace ---
const activeSessions = computed(() => {
  const sessions: Array<{
    id: string;
    panelId: string;
    title: string;
    isSSH: boolean;
    isActive: boolean;
  }> = [];

  const panels =
    typeof workspaceStore.getAllPanels === "function"
      ? workspaceStore.getAllPanels()
      : [];

  panels.forEach((panel) => {
    panel.tabs?.forEach((tab) => {
      sessions.push({
        id: tab.id,
        panelId: panel.id,
        title: tab.title || (tab.profileId ? "SSH Session" : "Terminal"),
        isSSH: Boolean(tab.profileId || tab.sshConfigHost),
        isActive: panel.activeTabId === tab.id,
      });
    });
  });

  return sessions;
});

const activeSSHSessionsCount = computed(() => {
  return activeSessions.value.filter((s) => s.isSSH).length;
});

const jumpToSession = (panelId: string, tabId: string) => {
  workspaceStore.selectTab(panelId, tabId);
  viewState.setActiveView("workspace");
};

// --- Quick Connect Actions ---
const displayProfiles = computed(() => {
  return [...sshStore.profiles]
    .sort((a, b) => a.name.localeCompare(b.name))
    .slice(0, 5);
});

const getActiveOrFirstPanelId = (): string => {
  if (workspaceStore.activePanelId) return workspaceStore.activePanelId;
  const panels =
    typeof workspaceStore.getAllPanels === "function"
      ? workspaceStore.getAllPanels()
      : [];
  return panels[0]?.id || "panel-1";
};

const connectToProfile = (profile: SSHProfile) => {
  const panelId = getActiveOrFirstPanelId();
  workspaceStore.addSSHTab(panelId, profile.id, profile.name);
  viewState.setActiveView("workspace");
  message.success(`Connecting to ${profile.name}...`);
};

const openSFTPForProfile = (_profile: SSHProfile) => {
  viewState.setActiveView("sftp");
};

const createLocalTerminal = async () => {
  const panelId = getActiveOrFirstPanelId();
  workspaceStore.addTab(panelId);
  viewState.setActiveView("workspace");
};

// --- Frequent / Favorite Commands ---
const displayCommands = computed(() => {
  if (savedCommandStore.favoriteCommands.length > 0) {
    return savedCommandStore.favoriteCommands.slice(0, 5);
  }
  return savedCommandStore.commands.slice(0, 5);
});

const copiedCommandId = ref<string | null>(null);

const handleRunCommand = async (cmd: SavedCommand) => {
  const focusedId = workspaceStore.focusedTerminalId;
  const activeTerminal = workspaceStore.terminals?.find(
    (t) => t.id === focusedId,
  );
  const activeBackendId = activeTerminal?.backendTerminalId;

  if (activeBackendId) {
    try {
      await writeToTerminal({
        terminalId: activeBackendId,
        data: cmd.command + "\n",
      });
      viewState.setActiveView("workspace");
      message.success(`Executed "${cmd.name}" in active terminal`);
      return;
    } catch (err) {
      console.warn("Failed to write to terminal:", err);
    }
  }
  await handleCopyCommand(cmd);
  message.info("Copied command to clipboard (no active terminal session)");
};

const handleCopyCommand = async (cmd: SavedCommand) => {
  try {
    await navigator.clipboard.writeText(cmd.command);
    copiedCommandId.value = cmd.id;
    setTimeout(() => {
      if (copiedCommandId.value === cmd.id) copiedCommandId.value = null;
    }, 1500);
    message.success("Copied to clipboard");
  } catch {
    message.error("Failed to copy command");
  }
};

// --- SSH Tunnels ---
const displayTunnels = computed(() => {
  return tunnelStore.tunnels.slice(0, 5);
});

const togglingTunnelId = ref<string | null>(null);

const handleToggleTunnel = async (tunnel: TunnelWithStatus) => {
  try {
    togglingTunnelId.value = tunnel.id;
    if (tunnel.status === "running") {
      await tunnelStore.stopTunnel(tunnel.id);
      message.success(`Tunnel "${tunnel.name}" stopped`);
    } else {
      await tunnelStore.startTunnel(tunnel.id);
      message.success(`Tunnel "${tunnel.name}" started`);
    }
  } catch (err: any) {
    message.error(err?.message || "Failed to toggle tunnel");
  } finally {
    togglingTunnelId.value = null;
  }
};

// --- Sync Status & Trigger ---
const activeDb = computed(() => {
  return (
    syncStore.currentDatabase ||
    syncStore.databases.find((db) => db.isActive) ||
    syncStore.databases[0] ||
    null
  );
});

const syncStatusText = computed(() => {
  if (syncStore.isSyncing) return "Syncing...";
  if (activeDb.value?.isActive) {
    return activeDb.value.name || "Connected";
  }
  return "Idle";
});

const lastSyncText = computed(() => {
  if (!activeDb.value) return "No database connected";
  const lastLog = syncStore.syncStatus?.lastSync || syncStore.syncLogs?.[0];
  const timeStr = lastLog?.completedAt || lastLog?.startedAt;
  if (timeStr) {
    try {
      const d = new Date(timeStr);
      return `Synced ${d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}`;
    } catch {
      return "Recently synced";
    }
  }
  return activeDb.value.isActive ? "Connected & Ready" : "Not synced yet";
});

const handleTriggerSync = async () => {
  const dbId = syncStore.currentDatabaseId || activeDb.value?.id;
  if (!dbId) {
    openOverlay("sync-manager-modal");
    message.info("Please select or configure a database in Sync Manager");
    return;
  }
  try {
    await syncStore.sync(dbId, "bidirectional");
    message.success("Sync completed successfully");
  } catch (err: any) {
    message.error(err?.message || "Sync failed");
  }
};

// --- Telemetry & Local System Info (Foldable) ---
interface CPUInfo {
  model: string;
  speed: number;
  usage: number;
}

interface DiskInfo {
  name: string;
  total_space: number;
  available_space: number;
  file_system: string;
  mount_point: string;
}

interface SystemInfo {
  platform: string;
  release: string;
  cpu_arch: string;
  hostname: string;
  uptime: number;
  total_memory: number;
  used_memory?: number;
  free_memory: number;
  available_memory?: number;
  load_average: [number, number, number];
  os_version?: string;
  cpu_info?: string;
  cpus?: CPUInfo[];
  disks_info?: DiskInfo[];
}

const isHostSpecsOpen = ref(false);
const loading = ref(false);
const systemInfo = ref<SystemInfo>({} as SystemInfo);
const lastUpdated = ref<Date | null>(null);

const calculatedCpuUsage = computed(() => {
  if (!systemInfo.value.cpus || systemInfo.value.cpus.length === 0) return "0.0%";
  const avg =
    systemInfo.value.cpus.reduce((acc, cpu) => acc + cpu.usage, 0) /
    systemInfo.value.cpus.length;
  return formatPercentage(avg);
});

const REFRESH_INTERVAL = 5000;
let refreshInterval: number | null = null;

const fetchSystemInfo = async () => {
  loading.value = true;
  try {
    const info = await settingsStore.getSystemInfo();
    systemInfo.value = info as SystemInfo;
    lastUpdated.value = new Date();
  } catch (error) {
    console.error("Error fetching system info:", error);
  } finally {
    loading.value = false;
  }
};

const formatLastUpdated = (date: Date) => {
  return date.toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
};

const startAutoRefresh = () => {
  stopAutoRefresh();
  refreshInterval = window.setInterval(fetchSystemInfo, REFRESH_INTERVAL);
};

const stopAutoRefresh = () => {
  if (refreshInterval) {
    clearInterval(refreshInterval);
    refreshInterval = null;
  }
};

const toggleHostSpecs = () => {
  isHostSpecsOpen.value = !isHostSpecsOpen.value;
  if (isHostSpecsOpen.value) {
    fetchSystemInfo();
    startAutoRefresh();
  } else {
    stopAutoRefresh();
  }
};

watch(
  () => viewState.activeView,
  (activeView) => {
    if (activeView === "dashboard" && isHostSpecsOpen.value) {
      fetchSystemInfo();
      startAutoRefresh();
    } else {
      stopAutoRefresh();
    }
  },
);

onUnmounted(() => {
  stopAutoRefresh();
});

// Maintenance / Easter egg deep verify
const refreshAttempts = ref(0);
const isOptimizingDisplay = ref(false);
const isLoadingIntegrity = ref(false);
const integrityStatus = ref<SystemIntegrityStatus | null>(null);

const handleForceRefresh = async () => {
  if (isOptimizingDisplay.value || isLoadingIntegrity.value) return;
  refreshAttempts.value++;
  if (refreshAttempts.value >= 10) {
    refreshAttempts.value = 0;
    isLoadingIntegrity.value = true;
    try {
      const status = await verifySystemIntegrity();
      integrityStatus.value = status;
      isOptimizingDisplay.value = true;
    } catch (error) {
      console.error("Failed to verify system integrity:", error);
    } finally {
      isLoadingIntegrity.value = false;
    }
  }
};
</script>
