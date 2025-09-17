/**
 * SSH Proxy configuration
 */
export interface SSHProxy {
  proxy_type: "http" | "socks4" | "socks5";
  host: string;
  port: number;
  username?: string;
  password?: string;
}

/**
 * Interface for SSH Tunnel configuration
 */
export interface SSHTunnel {
  id: string;
  name: string;
  description?: string;
  profileId: string;
  type: "local" | "remote" | "dynamic";
  localPort: number;
  remoteHost?: string;
  remotePort?: number;
  autoStart: boolean;
  status: "stopped" | "starting" | "running" | "error" | "reconnecting";
  created: Date;
  updated: Date;
  lastStarted?: Date;
  lastError?: string;
}

/**
 * Interface for SSH Group configuration
 */
export interface SSHGroup {
  id: string;
  name: string;
  description?: string;
  color?: string;
  icon?: string;
  sortOrder: number;
  is_expanded: boolean;
  created_at: Date;
  updated_at: Date;
}

/**
 * Interface for SSH Profile configuration
 */
export interface SSHProfile {
  id: string;
  name: string;
  host: string;
  port?: number;
  username: string;
  groupId?: string;
  auth_method: "password" | "privateKey" | "privateKeyWithPassphrase" | "agent" | "certificate" | "kerberos" | "PKCS11";
  auth_data?: object; // e.g., { password: string } or { keyPath: string, passphrase?: string }
  timeout?: number;
  keep_alive?: boolean;
  compression?: boolean;
  proxy?: SSHProxy;
  color?: string;
  icon?: string;
  sortOrder: number;
  description?: string;
  tags?: string[];
  created_at: Date;
  updated_at: Date;
  last_connected_at?: Date;
}
