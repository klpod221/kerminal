<template>
  <Modal
    id="tunnel-manager-modal"
    title="SSH Tunnel Manager"
    size="xl"
  >
    <!-- Empty State -->
    <EmptyState
      v-if="tunnelStore.tunnels.length === 0 && !tunnelStore.loading"
      :icon="Route"
      :icon-size="64"
      title="No SSH Tunnels"
      description="Create your first tunnel to securely forward ports or create SOCKS proxies"
      action-text="Create Tunnel"
      :action-icon="Plus"
      action-variant="primary"
      @action="openTunnelModal()"
    />

    <!-- Tunnel List -->
    <div v-else class="space-y-4">
      <!-- Header -->
      <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-4">
          <div class="text-sm text-gray-400">
            {{ tunnelStore.tunnels.length }} tunnel(s) configured
          </div>
          <div class="flex items-center gap-2">
            <Badge variant="success" size="sm" :dot="true">
              {{ activeTunnels.length }} active
            </Badge>
            <Badge variant="gray" size="sm" :dot="true">
              {{ stoppedTunnels.length }} stopped
            </Badge>
          </div>
        </div>
        <Button variant="primary" :icon="Plus" size="sm" @click="openTunnelModal()">
          Create Tunnel
        </Button>
      </div>

      <!-- Loading -->
      <div v-if="tunnelStore.loading" class="text-center py-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto"></div>
        <p class="text-gray-400 mt-4">Loading tunnels...</p>
      </div>

      <!-- Tunnels List -->
      <div v-else class="flex flex-col gap-4">
        <Card
          v-for="tunnel in validTunnels"
          :key="tunnel.id"
          :hover="true"
          class="relative"
        >
          <div class="space-y-3">
            <!-- Header -->
            <div class="flex items-start justify-between">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <h3 class="text-white font-semibold truncate">{{ tunnel.name }}</h3>
                  <TunnelStatusIndicator :status="tunnel.status" />
                </div>
                <div class="flex items-center gap-2 mt-1">
                  <Badge
                    variant="info"
                    size="xs"
                  >
                    {{ tunnel.tunnelType }}
                  </Badge>
                  <Badge
                    v-if="tunnel.autoStart"
                    variant="primary"
                    size="xs"
                  >
                    Auto-start
                  </Badge>
                </div>
              </div>
              <div class="flex items-center gap-1">
                <Button
                  v-if="tunnel.status === 'stopped'"
                  variant="ghost"
                  size="sm"
                  :icon="Play"
                  title="Start tunnel"
                  @click="startTunnel(tunnel)"
                />
                <Button
                  v-else-if="tunnel.status === 'running'"
                  variant="ghost"
                  size="sm"
                  :icon="Square"
                  title="Stop tunnel"
                  @click="stopTunnel(tunnel)"
                />
                <Button
                  variant="ghost"
                  size="sm"
                  :icon="Edit3"
                  title="Edit tunnel"
                  @click="openTunnelModal(tunnel)"
                />
                <Button
                  variant="ghost"
                  size="sm"
                  :icon="Copy"
                  title="Duplicate tunnel"
                  @click="duplicateTunnel(tunnel)"
                />
                <Button
                  variant="ghost"
                  size="sm"
                  :icon="Trash2"
                  title="Delete tunnel"
                  @click="confirmDelete(tunnel)"
                />
              </div>
            </div>

            <!-- Connection Details -->
            <div class="space-y-2">
              <div class="text-xs text-gray-400">Configuration:</div>
              <div class="text-xs font-mono text-gray-300 bg-gray-800 px-2 py-1 rounded">
                {{ formatTunnelConfig(tunnel) }}
              </div>
            </div>

            <!-- Profile Info -->
            <div v-if="getProfileName(tunnel.profileId)" class="text-xs text-gray-400">
              Profile: <span class="text-gray-300">{{ getProfileName(tunnel.profileId) }}</span>
            </div>

            <!-- Description -->
            <div v-if="tunnel.description" class="text-sm text-gray-400">
              {{ tunnel.description }}
            </div>

            <!-- Error Message -->
            <Badge
              v-if="tunnel.errorMessage"
              variant="danger"
              size="sm"
              custom-class="w-full justify-start"
            >
              {{ tunnel.errorMessage }}
            </Badge>
          </div>
        </Card>
      </div>
    </div>



  </Modal>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useTunnelStore } from '../../stores/tunnel';
import { useSSHStore } from '../../stores/ssh';
import { useOverlay } from '../../composables/useOverlay';
import type { TunnelWithStatus } from '../../types/tunnel';
import Modal from '../ui/Modal.vue';
import Button from '../ui/Button.vue';
import Badge from '../ui/Badge.vue';
import Card from '../ui/Card.vue';
import EmptyState from '../ui/EmptyState.vue';
import TunnelStatusIndicator from './TunnelStatusIndicator.vue';
import {
  Plus,
  Route,
  Edit3,
  Trash2,
  Copy,
  Play,
  Square,
} from 'lucide-vue-next';

const tunnelStore = useTunnelStore();
const sshStore = useSSHStore();
const { openOverlay } = useOverlay();


// Computed
const validTunnels = computed(() => {
  return tunnelStore.tunnels.filter(tunnel =>
    tunnel &&
    tunnel.id &&
    typeof tunnel.id === 'string'
  );
});

const activeTunnels = computed(() => tunnelStore.activeTunnels);
const stoppedTunnels = computed(() => tunnelStore.stoppedTunnels);

// Methods
const openTunnelModal = (tunnel?: TunnelWithStatus) => {
  openOverlay("tunnel-modal", { tunnelId: tunnel?.id || null });
};

const startTunnel = async (tunnel: TunnelWithStatus) => {
  if (!tunnel?.id) {
    console.error('Invalid tunnel data for start operation');
    return;
  }

  try {
    await tunnelStore.startTunnel(tunnel.id);
  } catch (error) {
    console.error('Failed to start tunnel:', error);
  }
};

const stopTunnel = async (tunnel: TunnelWithStatus) => {
  if (!tunnel?.id) {
    console.error('Invalid tunnel data for stop operation');
    return;
  }

  try {
    await tunnelStore.stopTunnel(tunnel.id);
  } catch (error) {
    console.error('Failed to stop tunnel:', error);
  }
};

const duplicateTunnel = async (tunnel: TunnelWithStatus) => {
  if (!tunnel?.id) {
    console.error('Invalid tunnel data for duplicate operation');
    return;
  }

  const duplicateData = {
    name: `${tunnel.name} (Copy)`,
    description: tunnel.description,
    profileId: tunnel.profileId,
    tunnelType: tunnel.tunnelType,
    localHost: tunnel.localHost,
    localPort: tunnel.localPort + 1, // Increment port to avoid conflicts
    remoteHost: tunnel.remoteHost,
    remotePort: tunnel.remotePort,
    autoStart: false, // Don't auto-start duplicates
  };

  try {
    await tunnelStore.createTunnel(duplicateData);
  } catch (error) {
    console.error('Failed to duplicate tunnel:', error);
  }
};

const confirmDelete = async (tunnel: TunnelWithStatus) => {
  const confirmed = confirm(`Are you sure you want to delete "${tunnel.name}"?\n\nThis action cannot be undone. The tunnel will be permanently removed.`);

  if (confirmed && tunnel.id) {
    try {
      await tunnelStore.deleteTunnel(tunnel.id);
    } catch (error) {
      console.error('Failed to delete tunnel:', error);
    }
  }
};

const formatTunnelConfig = (tunnel: TunnelWithStatus) => {
  switch (tunnel.tunnelType) {
    case 'Local':
      return `${tunnel.localHost}:${tunnel.localPort} → ${tunnel.remoteHost}:${tunnel.remotePort}`;
    case 'Remote':
      return `${tunnel.remoteHost}:${tunnel.remotePort} → ${tunnel.localHost}:${tunnel.localPort}`;
    case 'Dynamic':
      return `SOCKS proxy on ${tunnel.localHost}:${tunnel.localPort}`;
    default:
      return 'Unknown configuration';
  }
};

const getProfileName = (profileId: string) => {
  if (!profileId || !sshStore.profiles) {
    return 'Unknown Profile';
  }

  const profile = sshStore.profiles.find(p => p?.id === profileId);
  return profile?.name || 'Unknown Profile';
};

// Lifecycle
onMounted(async () => {
  await Promise.all([
    tunnelStore.loadTunnels(),
    sshStore.loadProfiles()
  ]);
});
</script>
