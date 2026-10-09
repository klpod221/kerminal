// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { ref } from "vue";
import { save, open } from "@tauri-apps/plugin-dialog";
import { useSavedCommandStore } from "../../stores/savedCommand";
import { message } from "../../utils/message";
import { createLocalFile, readLocalTextFile } from "../../services/localFs";
import {
  exportSnippetsToJson,
  exportSnippetsToShell,
  parseSnippetsFromJson,
  parseSnippetsFromShell,
} from "../../services/savedCommand/snippetTransfer";

export function useSnippetTransfer() {
  const savedCommandStore = useSavedCommandStore();
  const isImporting = ref(false);
  const isExporting = ref(false);

  /**
   * Export snippets: open native file picker dialog so user can select directory and filename
   */
  const exportSnippets = async () => {
    if (savedCommandStore.commands.length === 0) {
      message.warning("No commands to export.");
      return;
    }

    try {
      const defaultFilename = `kerminal-snippets-${new Date().toISOString().slice(0, 10)}.json`;
      const filePath = await save({
        filters: [
          { name: "JSON Snippets (*.json)", extensions: ["json"] },
          { name: "Shell Script (*.sh)", extensions: ["sh"] },
        ],
        defaultPath: defaultFilename,
      });

      if (!filePath) return;

      isExporting.value = true;
      const isShell = filePath.endsWith(".sh");
      const content = isShell
        ? exportSnippetsToShell(savedCommandStore.commands, savedCommandStore.groups)
        : exportSnippetsToJson(savedCommandStore.commands, savedCommandStore.groups);

      await createLocalFile(filePath, content);
      message.success(`Exported ${savedCommandStore.commands.length} snippets to: ${filePath}`);
    } catch (error) {
      console.error("Export failed:", error);
      message.error("Failed to export snippets: " + error);
    } finally {
      isExporting.value = false;
    }
  };

  /**
   * Import snippets: open native file dialog so user can select JSON or SH file
   */
  const triggerImportDialog = async () => {
    try {
      const filePath = await open({
        filters: [
          { name: "Snippet Files (*.json, *.sh)", extensions: ["json", "sh"] },
          { name: "JSON Snippets (*.json)", extensions: ["json"] },
          { name: "Shell Script (*.sh)", extensions: ["sh"] },
        ],
      });

      if (!filePath || typeof filePath !== "string") return;

      isImporting.value = true;
      const content = await readLocalTextFile(filePath);
      const isJson = filePath.endsWith(".json");
      const parsed = isJson
        ? parseSnippetsFromJson(content)
        : parseSnippetsFromShell(content);

      if (parsed.commands.length === 0) {
        message.warning("No valid snippets found in selected file.");
        return;
      }

      // Create missing groups first
      const groupMap = new Map<string, string>();
      for (const existingGroup of savedCommandStore.groups) {
        groupMap.set(existingGroup.name.toLowerCase(), existingGroup.id);
      }

      for (const grp of parsed.groups) {
        const lowerName = grp.name.toLowerCase();
        if (!groupMap.has(lowerName)) {
          const created = await savedCommandStore.createGroup({
            name: grp.name,
            color: grp.color,
            icon: grp.icon,
          });
          groupMap.set(lowerName, created.id);
        }
      }

      // Create imported commands
      let importedCount = 0;
      for (const cmd of parsed.commands) {
        const groupId = cmd.groupName ? groupMap.get(cmd.groupName.toLowerCase()) : undefined;
        await savedCommandStore.createCommand({
          name: cmd.name,
          command: cmd.command,
          description: cmd.description,
          groupId,
          tags: cmd.tags,
          isFavorite: cmd.isFavorite,
        });
        importedCount++;
      }

      await savedCommandStore.loadCommands();
      message.success(`Successfully imported ${importedCount} snippets from: ${filePath}`);
    } catch (err) {
      console.error("Import failed:", err);
      message.error("Failed to parse and import snippet file: " + err);
    } finally {
      isImporting.value = false;
    }
  };

  return {
    isImporting,
    isExporting,
    exportSnippets,
    exportAsJson: exportSnippets,
    triggerImportDialog,
  };
}
