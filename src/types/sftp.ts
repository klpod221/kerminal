/**
 * SFTP TypeScript types matching backend models
 */

/**
 * File type enumeration
 */
export type FileType = "file" | "directory" | "symlink" | "unknown";

/**
 * Represents a file or directory entry on the remote system
 */
export interface FileEntry {
  name: string;
  path: string;
  fileType: FileType;
  size: number | null;
  permissions: number;
  modified: string; // ISO 8601 datetime
  accessed: string | null; // ISO 8601 datetime
  symlinkTarget: string | null;
  uid: number | null;
  gid: number | null;
}

/**
 * Transfer status
 */
export type TransferStatus =
  | "queued"
  | "inprogress"
  | "paused"
  | "completed"
  | "failed"
  | "cancelled";

/**
 * Transfer direction
 */
export type TransferDirection = "upload" | "download";

/**
 * Transfer progress information
 */
export interface TransferProgress {
  transferId: string;
  status: TransferStatus;
  direction: TransferDirection;
  localPath: string;
  remotePath: string;
  totalBytes: number;
  transferredBytes: number;
  speedBytesPerSec: number | null;
  etaSeconds: number | null;
  error: string | null;
  startedAt: string; // ISO 8601 datetime
  completedAt: string | null; // ISO 8601 datetime
  priority: number; // 0-255, higher = higher priority
  retryCount: number; // Number of retry attempts made
  maxRetries: number; // Maximum number of retry attempts allowed
  nextRetryAt: string | null; // ISO 8601 datetime for next retry
}

/**
 * Synchronization direction
 */
export type SyncDirection = "localToRemote" | "remoteToLocal" | "bidirectional";

/**
 * Difference type between local and remote files
 */
export type DiffType =
  | "onlyLocal"
  | "onlyRemote"
  | "sizeDiffers"
  | "timeDiffers"
  | "identical"
  | "permissionsDiffer";

/**
 * Represents a difference between local and remote files
 */
export interface DiffEntry {
  path: string;
  diffType: DiffType;
  localEntry: FileEntry | null;
  remoteEntry: FileEntry | null;
}

/**
 * Synchronization operation parameters
 */
export interface SyncOperation {
  direction: SyncDirection;
  localPath: string;
  remotePath: string;
  deleteExtraFiles: boolean;
  preserveSymlinks: boolean;
  preservePermissions: boolean;
  maxFileSize: number | null;
  excludePatterns: string[];
}

/**
 * SFTP Error types from backend
 */
export interface SFTPError {
  message: string;
  kind?:
    | "SessionFailed"
    | "SessionNotFound"
    | "FileNotFound"
    | "FileExists"
    | "IoError"
    | "TransferNotFound"
    | "TransferNotResumable"
    | "Other";
}

/**
 * SFTP Session information
 */
export interface SFTPSession {
  sessionId: string;
  profileId: string;
  profileName: string;
  connectedAt: string;
  lastUsed: string;
}

/**
 * SFTP Browser state
 */
export interface SFTPBrowserState {
  activeSessionId: string | null;
  localPath: string;
  remotePath: string;
  localFiles: FileEntry[];
  remoteFiles: FileEntry[];
  selectedLocalFiles: Set<string>;
  selectedRemoteFiles: Set<string>;
  loading: {
    local: boolean;
    remote: boolean;
  };
  activeTransfers: Map<string, TransferProgress>;
}

/**
 * Drag and drop data structure for file browser
 */
export interface FileBrowserDragData {
  files: Partial<FileEntry>[];
  isRemote: boolean;
}

/**
 * Search result from backend
 */
export interface SearchResult {
  filePath: string;
  lineNumber: number;
  content: string;
}
