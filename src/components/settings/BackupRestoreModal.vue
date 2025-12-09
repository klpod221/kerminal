<template>
  <Modal
    id="backup-restore-modal"
    :show-close-button="true"
    title="Backup & Restore"
    :icon="Archive"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    size="md"
  >
    <div class="space-y-4">
      <!-- Export Section -->
      <Card
        title="Export Backup"
        :icon="Download"
        icon-background="bg-blue-500/20"
        icon-color="text-blue-400"
      >
        <p class="text-sm text-gray-400 mb-4">
          Download a full backup of your data including SSH profiles, keys,
          tunnels, and saved commands.
        </p>
        <div class="flex items-center gap-3">
          <Button
            variant="secondary"
            :icon="Download"
            :loading="exporting"
            @click="handleExport"
            class="w-full justify-center"
          >
            Export to JSON
          </Button>
        </div>
      </Card>

      <!-- Import Section -->
      <Card
        title="Import Backup"
        :icon="Upload"
        icon-background="bg-yellow-500/20"
        icon-color="text-yellow-400"
      >
        <p class="text-sm text-gray-400 mb-4">
          Restore data from a backup file. Existing data with matching IDs will
          be updated.
        </p>
        <Button
          variant="warning"
          :icon="Upload"
          :loading="importing"
          @click="handleImport"
          class="w-full justify-center"
        >
          Import from JSON
        </Button>
      </Card>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Download, Upload, Archive } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { save, open } from "@tauri-apps/plugin-dialog";
import { writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";
import { message } from "../../utils/message";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import Card from "../ui/Card.vue";
import { useOverlay } from "../../composables/useOverlay";

const { closeOverlay } = useOverlay();

const exporting = ref(false);
const importing = ref(false);

const handleExport = async () => {
  try {
    exporting.value = true;

    // Invoke: plugin:database|export_backup which is exposed as export_backup
    const backupJson = await invoke<string>("export_backup", {
      password: null,
    });

    const filePath = await save({
      filters: [
        {
          name: "Kerminal Backup",
          extensions: ["json"],
        },
      ],
      defaultPath: `kerminal-backup-${new Date().toISOString().split("T")[0]}.json`,
    });

    if (filePath) {
      await writeTextFile(filePath, backupJson);
      message.success("Backup exported successfully!");
      closeOverlay("backup-restore-modal");
    }
  } catch (error) {
    console.error("Export failed:", error);
    message.error("Failed to export backup: " + error);
  } finally {
    exporting.value = false;
  }
};

const handleImport = async () => {
  try {
    importing.value = true;

    const filePath = await open({
      filters: [
        {
          name: "Kerminal Backup",
          extensions: ["json"],
        },
      ],
    });

    if (filePath && typeof filePath === "string") {
      const content = await readTextFile(filePath);

      await invoke("import_backup", {
        backupContent: content,
        password: null,
      });

      message.success("Backup imported successfully!");
      closeOverlay("backup-restore-modal");

      // Optional: prompt reload
      setTimeout(() => {
        globalThis.location.reload();
      }, 1500);
    }
  } catch (error) {
    console.error("Import failed:", error);
    message.error("Failed to import backup: " + error);
  } finally {
    importing.value = false;
  }
};
</script>
