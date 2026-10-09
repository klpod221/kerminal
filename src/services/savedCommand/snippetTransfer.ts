// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import type {
  SavedCommand,
  SavedCommandGroup,
  SavedCommandScope,
} from "../../types/savedCommand";
import {
  extractScopeFromTags,
  getDisplayTags,
  packTagsWithScope,
} from "../../types/savedCommand";

export interface SnippetExportData {
  version: number;
  exportedAt: string;
  source: string;
  groups: Array<{
    id: string;
    name: string;
    description?: string;
    color?: string;
    icon?: string;
  }>;
  commands: Array<{
    name: string;
    description?: string;
    command: string;
    groupName?: string;
    tags: string[];
    scope: SavedCommandScope;
    isFavorite: boolean;
  }>;
}

export interface ParsedImportResult {
  groups: Array<{ name: string; color?: string; icon?: string }>;
  commands: Array<{
    name: string;
    description?: string;
    command: string;
    groupName?: string;
    tags: string;
    scope: SavedCommandScope;
    isFavorite: boolean;
  }>;
}

/**
 * Export saved commands and groups to a formatted JSON string
 */
export function exportSnippetsToJson(
  commands: SavedCommand[],
  groups: SavedCommandGroup[],
): string {
  const groupMap = new Map(groups.map((g) => [g.id, g.name]));

  const exportData: SnippetExportData = {
    version: 1,
    exportedAt: new Date().toISOString(),
    source: "Kerminal",
    groups: groups.map((g) => ({
      id: g.id,
      name: g.name,
      description: g.description,
      color: g.color,
      icon: g.icon,
    })),
    commands: commands.map((c) => ({
      name: c.name,
      description: c.description,
      command: c.command,
      groupName: c.groupId ? groupMap.get(c.groupId) : undefined,
      tags: getDisplayTags(c.tags),
      scope: extractScopeFromTags(c.tags),
      isFavorite: c.isFavorite,
    })),
  };

  return JSON.stringify(exportData, null, 2);
}

/**
 * Export saved commands into a runnable shell script (.sh)
 */
export function exportSnippetsToShell(
  commands: SavedCommand[],
  groups: SavedCommandGroup[],
): string {
  const groupMap = new Map(groups.map((g) => [g.id, g.name]));
  const lines: string[] = [
    "#!/usr/bin/env bash",
    "# Kerminal Snippets Export",
    `# Exported: ${new Date().toLocaleString()}`,
    `# Total commands: ${commands.length}`,
    "",
  ];

  for (const cmd of commands) {
    const scope = extractScopeFromTags(cmd.tags);
    const tags = getDisplayTags(cmd.tags).join(", ");
    const groupName = cmd.groupId ? groupMap.get(cmd.groupId) || "Ungrouped" : "Ungrouped";

    lines.push(`# -----------------------------------------------------------------------------`);
    lines.push(`# Name:  ${cmd.name}`);
    lines.push(`# Group: ${groupName} | Scope: ${scope}${tags ? ` | Tags: ${tags}` : ""}`);
    if (cmd.description) {
      lines.push(`# Info:  ${cmd.description}`);
    }
    lines.push(cmd.command);
    lines.push("");
  }

  return lines.join("\n");
}

/**
 * Parse and validate an imported JSON string
 */
export function parseSnippetsFromJson(jsonStr: string): ParsedImportResult {
  const parsed = JSON.parse(jsonStr) as Partial<SnippetExportData>;

  const groups = Array.isArray(parsed.groups)
    ? parsed.groups.map((g) => ({
        name: String(g.name || "").trim(),
        color: g.color,
        icon: g.icon,
      })).filter((g) => g.name.length > 0)
    : [];

  const commandsList = Array.isArray(parsed.commands) ? parsed.commands : [];
  const commands = commandsList
    .map((c) => {
      const name = String(c.name || "").trim();
      const commandText = String(c.command || "").trim();
      if (!name || !commandText) return null;

      const scope: SavedCommandScope =
        c.scope === "local" || c.scope === "ssh" ? c.scope : "any";
      const tagsList = Array.isArray(c.tags) ? c.tags.map(String) : [];

      return {
        name,
        description: c.description ? String(c.description) : undefined,
        command: commandText,
        groupName: c.groupName ? String(c.groupName).trim() : undefined,
        tags: packTagsWithScope(tagsList, scope),
        scope,
        isFavorite: Boolean(c.isFavorite),
      };
    })
    .filter((c): c is NonNullable<typeof c> => c !== null);

  return { groups, commands };
}

/**
 * Parse simple shell script into snippet commands
 */
export function parseSnippetsFromShell(shContent: string): ParsedImportResult {
  const lines = shContent.split(/\r?\n/);
  const commands: ParsedImportResult["commands"] = [];
  let currentName = "";
  let currentDesc = "";
  let currentGroup = "";
  let currentScope: SavedCommandScope = "any";
  let currentTags: string[] = [];
  let commandLines: string[] = [];

  const flushCommand = () => {
    const rawCmd = commandLines.join("\n").trim();
    if (rawCmd.length > 0) {
      commands.push({
        name: currentName || rawCmd.slice(0, 30),
        description: currentDesc || undefined,
        command: rawCmd,
        groupName: currentGroup || undefined,
        tags: packTagsWithScope(currentTags, currentScope),
        scope: currentScope,
        isFavorite: false,
      });
    }
    currentName = "";
    currentDesc = "";
    currentGroup = "";
    currentScope = "any";
    currentTags = [];
    commandLines = [];
  };

  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith("# Name:")) {
      currentName = trimmed.replace(/^# Name:\s*/, "");
    } else if (trimmed.startsWith("# Group:")) {
      const parts = trimmed.replace(/^# Group:\s*/, "").split("|");
      currentGroup = (parts[0] || "").trim();
      for (const part of parts.slice(1)) {
        if (part.includes("Scope:")) {
          const s = part.replace(/.*Scope:\s*/, "").trim().toLowerCase();
          if (s === "local" || s === "ssh") currentScope = s;
        } else if (part.includes("Tags:")) {
          currentTags = part.replace(/.*Tags:\s*/, "").split(",").map((t) => t.trim()).filter(Boolean);
        }
      }
    } else if (trimmed.startsWith("# Info:")) {
      currentDesc = trimmed.replace(/^# Info:\s*/, "");
    } else if (trimmed.startsWith("# ---")) {
      if (commandLines.length > 0) flushCommand();
    } else if (!trimmed.startsWith("#") && trimmed.length > 0) {
      commandLines.push(line);
    }
  }

  if (commandLines.length > 0) {
    flushCommand();
  }

  return { groups: [], commands };
}

/**
 * Trigger browser file download with blob
 */
export function triggerFileDownload(content: string, filename: string, mimeType = "application/json"): void {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
