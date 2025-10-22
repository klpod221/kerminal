import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  SSHTunnel,
  TunnelWithStatus,
  CreateSSHTunnelRequest,
  UpdateSSHTunnelRequest,
} from "../types/tunnel";
import { tunnelService } from "../services/tunnel";

export const useTunnelStore = defineStore("tunnel", () => {
  const tunnels = ref<TunnelWithStatus[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const activeTunnels = computed(() =>
    tunnels.value.filter(
      (t) => t && t.id && (t.status === "running" || t.status === "starting"),
    ),
  );

  const stoppedTunnels = computed(() =>
    tunnels.value.filter((t) => t && t.id && t.status === "stopped"),
  );

  const errorTunnels = computed(() =>
    tunnels.value.filter((t) => t && t.id && t.status === "error"),
  );

  const tunnelCount = computed(() => tunnels.value.length);
  const activeTunnelCount = computed(() => activeTunnels.value.length);

  const loadTunnels = async () => {
    loading.value = true;
    error.value = null;

    try {
      const loadedTunnels = await tunnelService.getTunnels();
      tunnels.value = (loadedTunnels || []).filter(
        (tunnel) => tunnel && tunnel.id && typeof tunnel.id === "string",
      );
    } catch (err) {
      error.value =
        err instanceof Error ? err.message : "Failed to load tunnels";
      console.error("Failed to load tunnels:", err);
      tunnels.value = []; // Ensure we have a valid array even on error
    } finally {
      loading.value = false;
    }
  };

  const createTunnel = async (
    request: CreateSSHTunnelRequest,
  ): Promise<SSHTunnel> => {
    loading.value = true;
    error.value = null;

    try {
      const tunnel = await tunnelService.createTunnel(request);
      await loadTunnels(); // Reload to get status
      return tunnel;
    } catch (err) {
      error.value =
        err instanceof Error ? err.message : "Failed to create tunnel";
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const updateTunnel = async (
    id: string,
    request: UpdateSSHTunnelRequest,
  ): Promise<SSHTunnel> => {
    loading.value = true;
    error.value = null;

    try {
      const tunnel = await tunnelService.updateTunnel(id, request);
      await loadTunnels(); // Reload to get updated data
      return tunnel;
    } catch (err) {
      error.value =
        err instanceof Error ? err.message : "Failed to update tunnel";
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const deleteTunnel = async (id: string): Promise<void> => {
    loading.value = true;
    error.value = null;

    try {
      await tunnelService.deleteTunnel(id);
      await loadTunnels(); // Reload to remove deleted tunnel
    } catch (err) {
      error.value =
        err instanceof Error ? err.message : "Failed to delete tunnel";
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const startTunnel = async (id: string): Promise<void> => {
    if (!id) {
      throw new Error("Tunnel ID is required");
    }

    try {
      const tunnel = tunnels.value.find((t) => t?.id === id);
      if (tunnel) {
        tunnel.status = "starting";
      }

      await tunnelService.startTunnel(id);

      setTimeout(async () => {
        try {
          const updatedTunnel = await tunnelService.getTunnel(id);
          if (updatedTunnel?.id) {
            const index = tunnels.value.findIndex((t) => t?.id === id);
            if (index !== -1) {
              tunnels.value[index] = updatedTunnel;
            }
          }
        } catch (err) {
          console.error("Failed to refresh tunnel status:", err);
        }
      }, 1000);
    } catch (err) {
      const tunnel = tunnels.value.find((t) => t?.id === id);
      if (tunnel) {
        tunnel.status = "error";
        tunnel.errorMessage =
          err instanceof Error ? err.message : "Failed to start tunnel";
      }
      throw err;
    }
  };

  const stopTunnel = async (id: string): Promise<void> => {
    if (!id) {
      throw new Error("Tunnel ID is required");
    }

    try {
      const tunnel = tunnels.value.find((t) => t?.id === id);
      if (tunnel) {
        tunnel.status = "stopped";
        tunnel.errorMessage = undefined;
      }

      await tunnelService.stopTunnel(id);
    } catch (err) {
      const tunnel = tunnels.value.find((t) => t?.id === id);
      if (tunnel) {
        tunnel.status = "error";
        tunnel.errorMessage =
          err instanceof Error ? err.message : "Failed to stop tunnel";
      }
      throw err;
    }
  };

  const refreshTunnelStatus = async (id: string): Promise<void> => {
    if (!id) {
      console.error("Tunnel ID is required for status refresh");
      return;
    }

    try {
      const updatedTunnel = await tunnelService.getTunnel(id);
      if (updatedTunnel?.id) {
        const index = tunnels.value.findIndex((t) => t?.id === id);
        if (index !== -1) {
          tunnels.value[index] = updatedTunnel;
        }
      }
    } catch (err) {
      console.error("Failed to refresh tunnel status:", err);
    }
  };

  const refreshAllTunnelStatus = async (): Promise<void> => {
    try {
      const updatedTunnels = await tunnelService.getTunnels();
      tunnels.value = (updatedTunnels || []).filter(
        (tunnel) => tunnel && tunnel.id && typeof tunnel.id === "string",
      );
    } catch (err) {
      console.error("Failed to refresh tunnel status:", err);
    }
  };

  const clearError = () => {
    error.value = null;
  };

  return {
    tunnels,
    loading,
    error,

    activeTunnels,
    stoppedTunnels,
    errorTunnels,
    tunnelCount,
    activeTunnelCount,

    loadTunnels,
    createTunnel,
    updateTunnel,
    deleteTunnel,
    startTunnel,
    stopTunnel,
    refreshTunnelStatus,
    refreshAllTunnelStatus,
    clearError,
  };
});
