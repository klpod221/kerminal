// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

export interface BaseModel {
  id: string;
  createdAt: string;
  updatedAt: string;
  deviceId: string;
  version: number;
  syncStatus: "synced" | "pending" | "conflict";
}

export type SavedCommandScope = "any" | "local" | "ssh";

export interface SavedCommand extends BaseModel {
  name: string;
  description?: string;
  command: string;
  groupId?: string;
  tags?: string; // JSON array as string
  scope?: SavedCommandScope;
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
  scope?: SavedCommandScope | "all";
  tags?: string[];
}

/**
 * Extract scope from a tags JSON string or array
 */
export function extractScopeFromTags(rawTags?: string | string[]): SavedCommandScope {
  if (!rawTags) return "any";
  try {
    const list: string[] = Array.isArray(rawTags)
      ? rawTags
      : JSON.parse(rawTags);
    if (list.includes("scope:local")) return "local";
    if (list.includes("scope:ssh")) return "ssh";
    return "any";
  } catch {
    return "any";
  }
}

/**
 * Clean tags to remove internal scope markers
 */
export function getDisplayTags(rawTags?: string | string[]): string[] {
  if (!rawTags) return [];
  try {
    const list: string[] = Array.isArray(rawTags)
      ? rawTags
      : JSON.parse(rawTags);
    return list.filter((t) => !t.startsWith("scope:"));
  } catch {
    return [];
  }
}

/**
 * Pack user tags and selected scope into a JSON tags string
 */
export function packTagsWithScope(
  tags: string[],
  scope: SavedCommandScope,
): string {
  const cleanTags = tags.filter((t) => !t.startsWith("scope:"));
  if (scope !== "any") {
    cleanTags.push(`scope:${scope}`);
  }
  return JSON.stringify(cleanTags);
}
