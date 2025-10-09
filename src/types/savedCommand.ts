// Base model interface (from existing types)
export interface BaseModel {
  id: string;
  createdAt: string;
  updatedAt: string;
  deviceId: string;
  version: number;
  syncStatus: "synced" | "pending" | "conflict";
}

// Saved command model
export interface SavedCommand {
  base: BaseModel;
  name: string;
  description?: string;
  command: string;
  groupId?: string;
  tags?: string; // JSON array as string
  isFavorite: boolean;
  usageCount: number;
  lastUsedAt?: string;
}

// Saved command group model
export interface SavedCommandGroup {
  base: BaseModel;
  name: string;
  description?: string;
  color?: string;
  icon?: string;
}

// Request to create a new saved command
export interface CreateSavedCommandRequest {
  name: string;
  description?: string;
  command: string;
  groupId?: string;
  tags?: string;
  isFavorite?: boolean;
}

// Request to update an existing saved command
export interface UpdateSavedCommandRequest {
  name?: string;
  description?: string;
  command?: string;
  groupId?: string;
  tags?: string;
  isFavorite?: boolean;
}

// Request to create a new saved command group
export interface CreateSavedCommandGroupRequest {
  name: string;
  description?: string;
  color?: string;
  icon?: string;
}

// Request to update an existing saved command group
export interface UpdateSavedCommandGroupRequest {
  name?: string;
  description?: string;
  color?: string;
  icon?: string;
}

// Enhanced interfaces for UI

// Saved command group with command count for UI display
export interface SavedCommandGroupWithStats extends SavedCommandGroup {
  commandCount: number;
}

// Parsed tags for easier frontend handling
export interface SavedCommandWithParsedTags extends SavedCommand {
  parsedTags: string[];
}

// Grouped data structure for drawer display (similar to SSH profiles)
export interface GroupedSavedCommandsData {
  group?: SavedCommandGroup;
  commands: SavedCommand[];
  commandCount: number;
}

// Sort and filter options for saved commands
export type SavedCommandSortBy =
  | "name"
  | "lastUsed"
  | "usageCount"
  | "createdAt"
  | "updatedAt";

export type SavedCommandFilterBy =
  | "all"
  | "favorites"
  | "recent"
  | "unused";

// Search and filter parameters
export interface SavedCommandSearchParams {
  query?: string;
  groupId?: string;
  sortBy?: SavedCommandSortBy;
  sortOrder?: "asc" | "desc";
  filterBy?: SavedCommandFilterBy;
  tags?: string[];
}
