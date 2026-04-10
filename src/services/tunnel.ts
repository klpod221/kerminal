/*
 * Kerminal - Modern Terminal Emulator & SSH Manager
 * Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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
