import { api } from "./api";
import type {
  SSHTunnel,
  TunnelWithStatus,
  CreateSSHTunnelRequest,
  UpdateSSHTunnelRequest,
  TunnelStatus,
} from "../types/tunnel";

/**
 * SSH Tunnel service for frontend
 */
export const tunnelService = {
  /**
   * Create new SSH tunnel
   */
  async createTunnel(request: CreateSSHTunnelRequest): Promise<SSHTunnel> {
    return await api.call("create_tunnel", request);
  },

  /**
   * Get all SSH tunnels with status
   */
  async getTunnels(): Promise<TunnelWithStatus[]> {
    return await api.callRaw("get_tunnels");
  },

  /**
   * Get SSH tunnel by ID with status
   */
  async getTunnel(id: string): Promise<TunnelWithStatus> {
    return await api.callRaw("get_tunnel", id);
  },

  /**
   * Update SSH tunnel
   */
  async updateTunnel(
    id: string,
    request: UpdateSSHTunnelRequest,
  ): Promise<SSHTunnel> {
    return await api.callRaw("update_tunnel", id, request);
  },

  /**
   * Delete SSH tunnel
   */
  async deleteTunnel(id: string): Promise<void> {
    return await api.callRaw("delete_tunnel", id);
  },

  /**
   * Start SSH tunnel
   */
  async startTunnel(id: string): Promise<void> {
    return await api.callRaw("start_tunnel", id);
  },

  /**
   * Stop SSH tunnel
   */
  async stopTunnel(id: string): Promise<void> {
    return await api.callRaw("stop_tunnel", id);
  },

  /**
   * Get tunnel status
   */
  async getTunnelStatus(id: string): Promise<TunnelStatus> {
    return await api.callRaw("get_tunnel_status", id);
  },
};
