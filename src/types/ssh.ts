/**
 * Base model interface from backend
 */
export interface BaseModel {
  id: string;
  createdAt: string;
  updatedAt: string;
  deviceId: string;
  version: number;
  syncStatus: "synced" | "pending" | "conflict";
}

/**
 * SSH Proxy configuration - matches backend ProxyConfig
 */
export interface ProxyConfig {
  proxyType: "Http" | "Socks4" | "Socks5";
  host: string;
  port: number;
  username?: string;
  password?: string;
}

/**
 * SSH Authentication methods - matches backend AuthMethod enum
 */
export type AuthMethod =
  | "Password"
  | "PrivateKey"
  | "PrivateKeyWithPassphrase"
  | "Agent"
  | "Certificate"
  | "Kerberos"
  | "PKCS11";

/**
 * SSH Key types - matches backend KeyType enum
 */
export type KeyType = "RSA" | "Ed25519" | "ECDSA" | "DSA";

/**
 * Certificate validity information
 */
export interface CertificateValidity {
  validFrom: string;
  validTo: string;
  serial?: string;
  caFingerprint?: string;
}

/**
 * SSH Authentication data - matches backend AuthData enum
 */
export type AuthData =
  | { Password: { password: string } }
  | { PrivateKey: { privateKey: string; keyType: KeyType; publicKey?: string } }
  | { PrivateKeyWithPassphrase: { privateKey: string; passphrase: string; keyType: KeyType; publicKey?: string } }
  | { Agent: { publicKey?: string } }
  | { Certificate: { certificate: string; privateKey: string; keyType: KeyType; validityPeriod?: CertificateValidity } }
  | { Kerberos: { realm: string; principal: string } }
  | { PKCS11: { libraryPath: string; slotId?: number; keyId: string; pin?: string } };

/**
 * SSH Group interface - matches backend SSHGroup
 */
export interface SSHGroup extends BaseModel {
  name: string;
  description?: string;
  color?: string;
  icon?: string;
  sortOrder: number;
  isExpanded: boolean;
  defaultAuthMethod?: string;
}

/**
 * SSH Profile interface - matches backend SSHProfile
 */
export interface SSHProfile extends BaseModel {
  name: string;
  host: string;
  port: number;
  username: string;
  groupId?: string;
  authMethod: AuthMethod;
  authData: AuthData;
  timeout?: number;
  keepAlive: boolean;
  compression: boolean;
  proxy?: ProxyConfig;
  color?: string;
  icon?: string;
  sortOrder: number;
  description?: string;
  tags: string[];
}

/**
 * Create SSH Group Request - matches backend CreateSSHGroupRequest
 */
export interface CreateSSHGroupRequest {
  name: string;
  description?: string;
  color?: string;
  icon?: string;
  sortOrder?: number;
  defaultAuthMethod?: string;
}

/**
 * Update SSH Group Request - matches backend UpdateSSHGroupRequest
 */
export interface UpdateSSHGroupRequest {
  name?: string;
  description?: string | null;
  color?: string | null;
  icon?: string | null;
  sortOrder?: number;
  isExpanded?: boolean;
  defaultAuthMethod?: string | null;
}

/**
 * Create SSH Profile Request - matches backend CreateSSHProfileRequest
 */
export interface CreateSSHProfileRequest {
  name: string;
  host: string;
  port?: number;
  username: string;
  groupId?: string;
  authMethod: AuthMethod;
  authData: AuthData;
  timeout?: number;
  keepAlive?: boolean;
  compression?: boolean;
  proxy?: ProxyConfig;
  color?: string;
  icon?: string;
  description?: string;
  tags?: string[];
}

/**
 * Update SSH Profile Request - matches backend UpdateSSHProfileRequest
 */
export interface UpdateSSHProfileRequest {
  name?: string;
  host?: string;
  port?: number;
  username?: string;
  groupId?: string | null;
  authMethod?: AuthMethod;
  authData?: AuthData;
  timeout?: number | null;
  keepAlive?: boolean;
  compression?: boolean;
  proxy?: ProxyConfig | null;
  color?: string | null;
  icon?: string | null;
  description?: string | null;
  tags?: string[];
}

/**
 * Delete Group Action - matches backend DeleteGroupAction
 */
export type DeleteGroupAction =
  | { actionType: "moveToGroup"; targetGroupId: string }
  | { actionType: "moveToUngrouped" }
  | { actionType: "deleteProfiles" };
