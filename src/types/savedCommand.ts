export interface BaseModel {
  id: string;
  createdAt: string;
  updatedAt: string;
  deviceId: string;
  version: number;
  syncStatus: "synced" | "pending" | "conflict";
}

export interface SavedCommand extends BaseModel {
  name: string;
  description?: string;
  command: string;
  groupId?: string;
  tags?: string; // JSON array as string
  isFavorite: boolean;
  usageCount: number;
  lastUsedAt?: string;
}

export interface SavedCommandGroup extends BaseModel {
  name: string;
  description?: string;
  color?: string;
  icon?: string;
}

export interface CreateSavedCommandRequest {
  name: string;
  description?: string;
  command: string;
  groupId?: string;
  tags?: string;
  isFavorite?: boolean;
}

export interface UpdateSavedCommandRequest {
  name?: string;
  description?: string;
  command?: string;
  groupId?: string;
  tags?: string;
  isFavorite?: boolean;
}

export interface CreateSavedCommandGroupRequest {
  name: string;
  description?: string;
  color?: string;
  icon?: string;
}

export interface UpdateSavedCommandGroupRequest {
  name?: string;
  description?: string;
  color?: string;
  icon?: string;
}

export interface SavedCommandGroupWithStats extends SavedCommandGroup {
  commandCount: number;
}

export interface SavedCommandWithParsedTags extends SavedCommand {
  parsedTags: string[];
}

export interface GroupedSavedCommandsData {
  group?: SavedCommandGroup;
  commands: SavedCommand[];
  commandCount: number;
}

export type SavedCommandSortBy =
  | "name"
  | "lastUsed"
  | "usageCount"
  | "createdAt"
  | "updatedAt";

export type SavedCommandFilterBy = "all" | "favorites" | "recent" | "unused";

export interface SavedCommandSearchParams {
  query?: string;
  groupId?: string;
  sortBy?: SavedCommandSortBy;
  sortOrder?: "asc" | "desc";
  filterBy?: SavedCommandFilterBy;
  tags?: string[];
}
