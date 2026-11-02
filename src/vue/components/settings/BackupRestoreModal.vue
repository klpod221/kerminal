<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <Modal
    id="backup-restore-modal"
    :show-close-button="true"
    title="Backup & Restore"
    :icon="Archive"
    icon-background="bg-gray-800"
    icon-color="text-gray-200"
    size="md"
  >
    <div class="space-y-4">
      <!-- Export Section -->
      <Card
        title="Export Backup"
        :icon="Download"
        icon-background="bg-zinc-800"
        icon-color="text-zinc-200"
        custom-class="border-zinc-700/60"
      >
        <template #icon>
          <Download class="w-5 h-5 text-zinc-200" />
        </template>
        <template #header>
          <div>
            <h3 class="text-sm font-semibold text-white">Export Backup</h3>
            <p class="text-xs text-zinc-400">
              Backup SSH profiles, keys, tunnels, and commands to a file.
            </p>
          </div>
        </template>

        <div class="space-y-3">
          <!-- Encryption Toggle Card -->
          <div
            class="rounded-lg border p-3 transition-colors"
            :class="
              useEncryption
                ? 'border-zinc-600 bg-zinc-800/60'
                : 'border-zinc-700/60 bg-zinc-900/40'
            "
          >
            <div class="flex items-start gap-2.5">
              <div class="shrink-0 mt-0.5">
                <Checkbox
                  id="use-encryption"
                  v-model="useEncryption"
                  class="mb-0!"
                />
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-1.5 mb-0.5">
                  <Shield class="w-3.5 h-3.5 text-zinc-400" />
                  <span class="text-xs font-medium text-zinc-200">
                    Password Protection
                  </span>
                </div>
                <p class="text-2xs text-zinc-400">
                  Encrypt your backup with AES-256-GCM encryption
                </p>
              </div>
            </div>

            <!-- Password Fields -->
            <Transition
              enter-active-class="transition-all duration-200 ease-out"
              enter-from-class="opacity-0 max-h-0"
              enter-to-class="opacity-100 max-h-96"
              leave-active-class="transition-all duration-150 ease-in"
              leave-from-class="opacity-100 max-h-96"
              leave-to-class="opacity-0 max-h-0"
            >
              <div
                v-if="useEncryption"
                class="mt-3 pt-3 border-t border-zinc-700/60 space-y-2.5 overflow-hidden"
              >
                <Input
                  id="export-password"
                  v-model="exportPassword"
                  type="password"
                  placeholder="Enter a strong password"
                  class="mb-0!"
                  :left-icon="Lock"
                />
                <Input
                  id="export-password-confirm"
                  v-model="exportPasswordConfirm"
                  type="password"
                  placeholder="Confirm your password"
                  class="mb-0!"
                  :left-icon="LockKeyhole"
                />

                <!-- Password Strength Indicator -->
                <div
                  v-if="exportPassword"
                  class="flex items-center gap-2 text-xs pt-1"
                >
                  <div
                    class="flex-1 h-1.5 bg-zinc-700 rounded-full overflow-hidden"
                  >
                    <div
                      class="h-full transition-all duration-300"
                      :class="passwordStrengthClass"
                      :style="{ width: passwordStrength + '%' }"
                    ></div>
                  </div>
                  <span class="text-2xs text-zinc-400 min-w-[50px]">{{
                    passwordStrengthText
                  }}</span>
                </div>
              </div>
            </Transition>
          </div>

          <!-- Export Button -->
          <Button
            variant="secondary"
            :icon="Download"
            :loading="exporting"
            @click="handleExport"
            class="w-full justify-center"
          >
            Export {{ useEncryption ? "Encrypted " : "" }}Backup
          </Button>

          <!-- Info Badge -->
          <div class="flex items-center gap-1.5 text-2xs text-zinc-400 px-0.5">
            <Info class="w-3 h-3 text-zinc-400 shrink-0" />
            <span>Format: <code class="font-mono text-zinc-300">.{{ useEncryption ? "kbak" : "json" }}</code></span>
          </div>
        </div>
      </Card>

      <!-- Import Section -->
      <Card
        title="Import Backup"
        :icon="Upload"
        icon-background="bg-zinc-800"
        icon-color="text-zinc-200"
        custom-class="border-zinc-700/60"
      >
        <template #icon>
          <Upload class="w-5 h-5 text-zinc-200" />
        </template>
        <template #header>
          <div>
            <h3 class="text-sm font-semibold text-white">Import Backup</h3>
            <p class="text-xs text-zinc-400">
              Restore your data from a plain JSON or encrypted backup file.
            </p>
          </div>
        </template>

        <div class="space-y-3">
          <!-- Encrypted Backup Detected -->
          <Transition
            enter-active-class="transition-all duration-200 ease-out"
            enter-from-class="opacity-0 -translate-y-1"
            enter-to-class="opacity-100 translate-y-0"
            leave-active-class="transition-all duration-150 ease-in"
            leave-from-class="opacity-100 translate-y-0"
            leave-to-class="opacity-0 -translate-y-1"
          >
            <div
              v-if="importRequiresPassword"
              class="rounded-lg border border-zinc-700 bg-zinc-800/40 p-3 space-y-2"
            >
              <div class="flex items-center gap-2">
                <ShieldCheck class="w-4 h-4 text-zinc-300" />
                <span class="text-xs font-medium text-zinc-200">
                  Password Required
                </span>
              </div>
              <Input
                id="import-password"
                v-model="importPassword"
                type="password"
                placeholder="Enter backup password"
                class="mb-0!"
                :left-icon="Key"
              />
            </div>
          </Transition>

          <!-- Import Button -->
          <Button
            variant="secondary"
            :icon="Upload"
            :loading="importing"
            @click="handleImport"
            class="w-full justify-center"
          >
            {{
              importRequiresPassword
                ? "Decrypt & Import"
                : "Select Backup File"
            }}
          </Button>

          <!-- Warning Note -->
          <div
            class="flex items-start gap-2 p-2.5 rounded-lg bg-zinc-900/60 border border-zinc-800 text-xs text-zinc-400 leading-relaxed"
          >
            <AlertTriangle class="w-3.5 h-3.5 text-amber-400/80 mt-0.5 shrink-0" />
            <p class="text-2xs text-zinc-400">
              Importing will update existing records with matching IDs. The app will reload after import completes.
            </p>
          </div>
        </div>
      </Card>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import {
  Download,
  Upload,
  Archive,
  Shield,
  Lock,
  LockKeyhole,
  Key,
  ShieldCheck,
  Info,
  AlertTriangle,
} from "lucide-vue-next";

import { save, open } from "@tauri-apps/plugin-dialog";
import { api } from "../../services/api";
import { message } from "../../utils/message";
import Modal from "../ui/Modal.vue";
import Card from "../ui/Card.vue";
import Button from "../ui/Button.vue";
import Input from "../ui/Input.vue";
import Checkbox from "../ui/Checkbox.vue";
import { useOverlay } from "../../composables/useOverlay";

const { closeOverlay } = useOverlay();

const exporting = ref(false);
const importing = ref(false);

// Export state
const useEncryption = ref(false);
const exportPassword = ref("");
const exportPasswordConfirm = ref("");

// Import state
const importPassword = ref("");
const importRequiresPassword = ref(false);
const selectedImportFilePath = ref<string | null>(null);

// Password strength calculation
const passwordStrength = computed(() => {
  const pwd = exportPassword.value;
  if (!pwd) return 0;

  let strength = 0;
  if (pwd.length >= 8) strength += 25;
  if (pwd.length >= 12) strength += 25;
  if (/[a-z]/.test(pwd) && /[A-Z]/.test(pwd)) strength += 25;
  if (/\d/.test(pwd)) strength += 15;
  if (/[^a-zA-Z0-9]/.test(pwd)) strength += 10;

  return Math.min(100, strength);
});

const passwordStrengthText = computed(() => {
  const strength = passwordStrength.value;
  if (strength < 40) return "Weak";
  if (strength < 70) return "Medium";
  return "Strong";
});

const passwordStrengthClass = computed(() => {
  const strength = passwordStrength.value;
  if (strength < 40) return "bg-red-500";
  if (strength < 70) return "bg-yellow-500";
  return "bg-green-500";
});

const handleExport = async () => {
  try {
    // Validate passwords if encryption is enabled
    if (useEncryption.value) {
      if (!exportPassword.value) {
        message.error("Please enter a password");
        return;
      }
      if (exportPassword.value !== exportPasswordConfirm.value) {
        message.error("Passwords do not match");
        return;
      }
      if (passwordStrength.value < 40) {
        message.warning(
          "Your password is weak. Consider using a stronger password.",
        );
      }
    }

    exporting.value = true;

    const backupData = await api.callRaw<string>("export_backup", {
      password: useEncryption.value ? exportPassword.value : null,
    });

    const fileExtension = useEncryption.value ? "kbak" : "json";
    const fileName = `kerminal-backup-${new Date().toISOString().split("T")[0]}.${fileExtension}`;

    const filePath = await save({
      filters: [
        {
          name: useEncryption.value
            ? "Kerminal Encrypted Backup"
            : "Kerminal Backup",
          extensions: [fileExtension],
        },
      ],
      defaultPath: fileName,
    });

    if (filePath) {
      await api.callRaw("local_fs_write_text_file", { path: filePath, contents: backupData });
      message.success(
        `Backup exported successfully${useEncryption.value ? " (encrypted)" : ""}!`,
      );

      // Reset form
      exportPassword.value = "";
      exportPasswordConfirm.value = "";
      useEncryption.value = false;

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

    let targetPath = selectedImportFilePath.value;
    if (!targetPath) {
      const chosen = await open({
        filters: [
          {
            name: "Kerminal Backup",
            extensions: ["json", "kbak"],
          },
        ],
      });
      if (chosen && typeof chosen === "string") {
        targetPath = chosen;
      }
    }

    if (targetPath) {
      const content = await api.callRaw<string>("local_fs_read_text_file", { path: targetPath });

      // Auto-detect if backup is encrypted (Base64 vs JSON)
      const isEncrypted = !content.trim().startsWith("{");

      if (isEncrypted && !importRequiresPassword.value) {
        // First time detecting encrypted file - remember path and show password field
        selectedImportFilePath.value = targetPath;
        importRequiresPassword.value = true;
        importing.value = false;
        message.info("This backup is encrypted. Please enter the password and click Import Backup.");
        return;
      }

      if (isEncrypted && !importPassword.value) {
        message.error("Please enter the backup password");
        importing.value = false;
        return;
      }

      await api.callRaw("import_backup", {
        backupContent: content,
        password: isEncrypted ? importPassword.value : null,
      });

      message.success("Backup imported successfully!");

      // Reset form
      importPassword.value = "";
      importRequiresPassword.value = false;
      selectedImportFilePath.value = null;

      closeOverlay("backup-restore-modal");

      // Reload to reflect imported data
      setTimeout(() => {
        globalThis.location.reload();
      }, 1500);
    }
  } catch (error) {
    console.error("Import failed:", error);
    const errorMsg = String(error);

    if (
      errorMsg.includes("Decryption failed") ||
      errorMsg.includes("Invalid")
    ) {
      message.error("Failed to decrypt backup. Please check your password.");
      importPassword.value = "";
    } else {
      message.error("Failed to import backup: " + error);
    }
  } finally {
    importing.value = false;
  }
};
</script>
