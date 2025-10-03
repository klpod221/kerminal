// SSH Tunnel types
export type TunnelType = "Local" | "Remote" | "Dynamic";
export type TunnelStatus = "stopped" | "starting" | "running" | "error";

export interface BaseModel {
  id: string;
  createdAt: string;
  updatedAt: string;
  deviceId: string;
  version: number;
  syncStatus: "synced" | "pending" | "conflict";
}

export interface SSHTunnel extends BaseModel {
  name: string;
  description?: string;
  profileId: string;
  tunnelType: TunnelType;
  localHost: string;
  localPort: number;
  remoteHost?: string;
  remotePort?: number;
  autoStart: boolean;
}

export interface TunnelWithStatus extends SSHTunnel {
  status: TunnelStatus;
  errorMessage?: string;
}

export interface CreateSSHTunnelRequest {
  name: string;
  description?: string;
  profileId: string;
  tunnelType: TunnelType;
  localHost: string;
  localPort: number;
  remoteHost?: string;
  remotePort?: number;
  autoStart?: boolean;
}

export interface UpdateSSHTunnelRequest {
  name?: string;
  description?: string;
  profileId?: string;
  tunnelType?: TunnelType;
  localHost?: string;
  localPort?: number;
  remoteHost?: string;
  remotePort?: number;
  autoStart?: boolean;
}
