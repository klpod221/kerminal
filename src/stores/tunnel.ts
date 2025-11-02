import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  SSHTunnel,
  TunnelWithStatus,
  CreateSSHTunnelRequest,
  UpdateSSHTunnelRequest,
} from "../types/tunnel";
import { tunnelService } from "../services/tunnel";
import { api } from "../services/api";
import {
  withRetry,
  handleError,
  type ErrorContext,
} from "../utils/errorHandler";
import { message } from "../utils/message";

export const useTunnelStore = defineStore("tunnel", () => {
  const tunnels = ref<TunnelWithStatus[]>([]);
  const isLoading = ref(false);

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

  /**
   * Load all tunnels with retry logic
   */
  const loadTunnels = async () => {
    isLoading.value = true;

    const context: ErrorContext = {
      operation: "Load Tunnels",
    };

    try {
      const loadedTunnels = await withRetry(
        () => tunnelService.getTunnels(),
        { maxRetries: 2 },
        context,
      );
      tunnels.value = (loadedTunnels || []).filter(
        (tunnel) => tunnel && tunnel.id && typeof tunnel.id === "string",
      );
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      tunnels.value = []; // Ensure we have a valid array even on error
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Create a new SSH tunnel with error handling
   * @param request - Tunnel creation request
   * @returns Created tunnel
   */
  const createTunnel = async (
    request: CreateSSHTunnelRequest,
  ): Promise<SSHTunnel> => {
    isLoading.value = true;

    const context: ErrorContext = {
      operation: "Create Tunnel",
      context: { tunnelType: request.tunnelType, name: request.name },
    };

    try {
      const tunnel = await withRetry(
        () => tunnelService.createTunnel(request),
        { maxRetries: 1 },
        context,
      );
      await loadTunnels(); // Reload to get status
      return tunnel;
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  const updateTunnel = async (
    id: string,
    request: UpdateSSHTunnelRequest,
  ): Promise<SSHTunnel> => {
    isLoading.value = true;

    try {
      const tunnel = await tunnelService.updateTunnel(id, request);
      // Optimistically mark as stopped to avoid stale "starting" state after edits
      const idx = tunnels.value.findIndex((t) => t?.id === id);
      if (idx !== -1 && tunnels.value[idx]) {
        tunnels.value[idx] = {
          ...tunnels.value[idx],
          ...tunnel,
          status: "stopped" as const,
          errorMessage: undefined,
        };
      }
      await loadTunnels(); // Reload to get updated data
      return tunnel;
    } catch (err) {
      const errorMessage = handleError(err, {
        operation: "Update Tunnel",
        context: { tunnelId: id },
      });
      message.error(errorMessage);
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  const deleteTunnel = async (id: string): Promise<void> => {
    isLoading.value = true;

    try {
      await tunnelService.deleteTunnel(id);
      await loadTunnels(); // Reload to remove deleted tunnel
    } catch (err) {
      const errorMessage = handleError(err, {
        operation: "Delete Tunnel",
        context: { tunnelId: id },
      });
      message.error(errorMessage);
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Start a tunnel with retry logic and error handling
   * @param id - Tunnel ID to start
   */
  const startTunnel = async (id: string): Promise<void> => {
    if (!id) {
      throw new Error("Tunnel ID is required");
    }

    const context: ErrorContext = {
      operation: "Start Tunnel",
      context: { tunnelId: id },
    };

    try {
      const tunnel = tunnels.value.find((t) => t?.id === id);
      if (tunnel) {
        tunnel.status = "starting";
      }

      await withRetry(
        () => tunnelService.startTunnel(id),
        { maxRetries: 2, retryDelay: 2000 },
        context,
      );

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
          const errorMessage = handleError(err, {
            operation: "Refresh Tunnel Status",
            context: { tunnelId: id },
          });
          message.error(errorMessage);
        }
      }, 1000);
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      const tunnel = tunnels.value.find((t) => t?.id === id);
      if (tunnel) {
        tunnel.status = "error";
        tunnel.errorMessage = errorMessage;
      }
      throw new Error(errorMessage);
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

  const upsertTunnel = (updated: TunnelWithStatus) => {
    if (!updated?.id) return;
    const index = tunnels.value.findIndex((t) => t?.id === updated.id);
    if (index === -1) {
      tunnels.value = [...tunnels.value, updated];
    } else if (tunnels.value[index]) {
      tunnels.value[index] = { ...tunnels.value[index]!, ...updated };
    }
  };

  const setTunnelStatus = (
    id: string,
    status: TunnelWithStatus["status"],
    errorMessage?: string,
  ) => {
    const index = tunnels.value.findIndex((t) => t?.id === id);
    if (index !== -1 && tunnels.value[index]) {
      tunnels.value[index] = {
        ...tunnels.value[index]!,
        status,
        errorMessage,
      };
    }
  };

  const removeTunnel = (id: string) => {
    tunnels.value = tunnels.value.filter((t) => t?.id !== id);
  };

  let unsubscribeStatusRealtime: (() => void) | null = null;
  let unsubscribeCrudRealtime: (() => void) | null = null;

  const startRealtimeStatus = async (): Promise<void> => {
    if (unsubscribeStatusRealtime) return;
    try {
      const u1 = await api.listen<TunnelWithStatus>("tunnel_started", (t) =>
        upsertTunnel(t),
      );
      const u2 = await api.listen<TunnelWithStatus>("tunnel_stopped", (t) =>
        upsertTunnel(t),
      );
      const u3 = await api.listen<TunnelWithStatus>(
        "tunnel_status_changed",
        (t) => upsertTunnel(t),
      );
      unsubscribeStatusRealtime = () => {
        u1();
        u2();
        u3();
      };
    } catch (e) {
      console.error("Failed to subscribe tunnel status realtime:", e);
    }
  };

  const startRealtimeCrud = async (): Promise<void> => {
    if (unsubscribeCrudRealtime) return;
    try {
      const u1 = await api.listen<TunnelWithStatus>("tunnel_created", (t) =>
        upsertTunnel(t),
      );
      const u2 = await api.listen<TunnelWithStatus>("tunnel_updated", (t) =>
        upsertTunnel(t),
      );
      const u3 = await api.listen<{ id: string }>("tunnel_deleted", ({ id }) =>
        removeTunnel(id),
      );
      unsubscribeCrudRealtime = () => {
        u1();
        u2();
        u3();
      };
    } catch (e) {
      console.error("Failed to subscribe tunnel CRUD realtime:", e);
    }
  };

  const startRealtime = async (): Promise<void> => {
    await Promise.all([startRealtimeStatus(), startRealtimeCrud()]);
  };

  const stopRealtimeStatus = (): void => {
    if (unsubscribeStatusRealtime) {
      unsubscribeStatusRealtime();
      unsubscribeStatusRealtime = null;
    }
  };

  const stopRealtimeCrud = (): void => {
    if (unsubscribeCrudRealtime) {
      unsubscribeCrudRealtime();
      unsubscribeCrudRealtime = null;
    }
  };

  const stopRealtime = (): void => {
    stopRealtimeStatus();
    stopRealtimeCrud();
  };

  return {
    tunnels,
    isLoading,

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
    upsertTunnel,
    setTunnelStatus,
    removeTunnel,
    startRealtimeStatus,
    stopRealtimeStatus,
    startRealtimeCrud,
    stopRealtimeCrud,
    startRealtime,
    stopRealtime,
  };
});
