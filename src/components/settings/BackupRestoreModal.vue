<template>
  <Modal
    id="backup-restore-modal"
    :show-close-button="true"
    title="Backup & Restore"
    :icon="Archive"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    size="lg"
  >
    <div class="space-y-6">
      <!-- Export Section -->
      <Card
        title="Export Backup"
        :icon="Download"
        icon-background="bg-gradient-to-br from-blue-500/20 to-cyan-500/20"
        icon-color="text-blue-400"
        class="border-blue-500/20"
      >
        <template #description>
          <p class="text-sm text-gray-400 leading-relaxed">
            Create a complete backup of your SSH profiles, keys, tunnels, and
            saved commands. Optionally encrypt with a password for enhanced
            security.
          </p>
        </template>

        <div class="space-y-4 mt-4">
          <!-- Encryption Toggle Card -->
          <div
            class="rounded-lg border transition-all duration-300"
            :class="
              useEncryption
                ? 'border-blue-500/40 bg-blue-500/5'
                : 'border-gray-700/50 bg-gray-800/30'
            "
          >
            <div class="p-4">
              <div class="flex items-start gap-3">
                <div class="shrink-0 mt-0.5">
                  <Checkbox
                    id="use-encryption"
                    v-model="useEncryption"
                    class="mb-0!"
                  />
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    <Shield class="w-4 h-4 text-blue-400" />
                    <span class="text-sm font-medium text-gray-200">
                      Password Protection
                    </span>
                  </div>
                  <p class="text-xs text-gray-500">
                    Encrypt your backup with AES-256-GCM encryption
                  </p>
                </div>
              </div>

              <!-- Password Fields (animated) -->
              <Transition
                enter-active-class="transition-all duration-300 ease-out"
                enter-from-class="opacity-0 max-h-0"
                enter-to-class="opacity-100 max-h-96"
                leave-active-class="transition-all duration-200 ease-in"
                leave-from-class="opacity-100 max-h-96"
                leave-to-class="opacity-0 max-h-0"
              >
                <div
                  v-if="useEncryption"
                  class="mt-4 space-y-3 overflow-hidden"
                >
                  <div class="relative">
                    <Input
                      id="export-password"
                      v-model="exportPassword"
                      type="password"
                      placeholder="Enter a strong password"
                      class="mb-0!"
                      :left-icon="Lock"
                    />
                  </div>
                  <div class="relative">
                    <Input
                      id="export-password-confirm"
                      v-model="exportPasswordConfirm"
                      type="password"
                      placeholder="Confirm your password"
                      class="mb-0!"
                      :left-icon="LockKeyhole"
                    />
                  </div>

                  <!-- Password Strength Indicator -->
                  <div
                    v-if="exportPassword"
                    class="flex items-center gap-2 text-xs"
                  >
                    <div
                      class="flex-1 h-1.5 bg-gray-700 rounded-full overflow-hidden"
                    >
                      <div
                        class="h-full transition-all duration-300"
                        :class="passwordStrengthClass"
                        :style="{ width: passwordStrength + '%' }"
                      ></div>
                    </div>
                    <span class="text-gray-500 min-w-[60px]">{{
                      passwordStrengthText
                    }}</span>
                  </div>
                </div>
              </Transition>
            </div>
          </div>

          <!-- Export Button -->
          <Button
            variant="secondary"
            :icon="Download"
            :loading="exporting"
            @click="handleExport"
            class="w-full justify-center group"
          >
            <span class="flex items-center gap-2">
              <span>Export {{ useEncryption ? "Encrypted" : "" }} Backup</span>
              <ArrowRight
                class="w-4 h-4 group-hover:translate-x-0.5 transition-transform"
              />
            </span>
          </Button>

          <!-- Info Badge -->
          <div class="flex items-center gap-2 text-xs text-gray-500">
            <Info class="w-3.5 h-3.5" />
            <span>
              File format:
              <code class="text-blue-400 font-mono">
                .{{ useEncryption ? "kbak" : "json" }}
              </code>
            </span>
          </div>
        </div>
      </Card>

      <!-- Import Section -->
      <Card
        title="Import Backup"
        :icon="Upload"
        icon-background="bg-gradient-to-br from-yellow-500/20 to-orange-500/20"
        icon-color="text-yellow-400"
        class="border-yellow-500/20"
      >
        <template #description>
          <p class="text-sm text-gray-400 leading-relaxed">
            Restore your data from a backup file. Supports both plain and
            encrypted backups. Existing data will be updated.
          </p>
        </template>

        <div class="space-y-4 mt-4">
          <!-- Encrypted Backup Detected -->
          <Transition
            enter-active-class="transition-all duration-300 ease-out"
            enter-from-class="opacity-0 -translate-y-2"
            enter-to-class="opacity-100 translate-y-0"
            leave-active-class="transition-all duration-200 ease-in"
            leave-from-class="opacity-100 translate-y-0"
            leave-to-class="opacity-0 -translate-y-2"
          >
            <div
              v-if="importRequiresPassword"
              class="rounded-lg border border-yellow-500/40 bg-yellow-500/5 p-4"
            >
              <div class="flex items-start gap-3 mb-3">
                <ShieldCheck class="w-5 h-5 text-yellow-400 mt-0.5 shrink-0" />
                <div>
                  <div class="text-sm font-medium text-yellow-300 mb-1">
                    Encrypted Backup Detected
                  </div>
                  <p class="text-xs text-yellow-400/80">
                    This backup is password-protected. Please enter the password
                    to continue.
                  </p>
                </div>
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
            variant="warning"
            :icon="Upload"
            :loading="importing"
            @click="handleImport"
            class="w-full justify-center group"
          >
            <span class="flex items-center gap-2">
              <span>{{
                importRequiresPassword
                  ? "Decrypt & Import"
                  : "Select Backup File"
              }}</span>
              <ArrowRight
                class="w-4 h-4 group-hover:translate-x-0.5 transition-transform"
              />
            </span>
          </Button>

          <!-- Warning Badge -->
          <div
            class="flex items-start gap-2 p-3 rounded-lg bg-orange-500/5 border border-orange-500/20"
          >
            <AlertTriangle class="w-4 h-4 text-orange-400 mt-0.5 shrink-0" />
            <p class="text-xs text-orange-400/90 leading-relaxed">
              Importing will update existing data with matching IDs. The app
              will reload after import completes.
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
  ArrowRight,
} from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { save, open } from "@tauri-apps/plugin-dialog";
import { writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";
import { message } from "../../utils/message";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import Card from "../ui/Card.vue";
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

    const backupData = await invoke<string>("export_backup", {
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
      await writeTextFile(filePath, backupData);
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

    const filePath = await open({
      filters: [
        {
          name: "Kerminal Backup",
          extensions: ["json", "kbak"],
        },
      ],
    });

    if (filePath && typeof filePath === "string") {
      const content = await readTextFile(filePath);

      // Auto-detect if backup is encrypted (Base64 vs JSON)
      const isEncrypted = !content.trim().startsWith("{");

      if (isEncrypted && !importRequiresPassword.value) {
        // First time detecting encrypted file - show password field
        importRequiresPassword.value = true;
        importing.value = false;
        message.info("This backup is encrypted. Please enter the password.");
        return;
      }

      if (isEncrypted && !importPassword.value) {
        message.error("Please enter the backup password");
        importing.value = false;
        return;
      }

      await invoke("import_backup", {
        backupContent: content,
        password: isEncrypted ? importPassword.value : null,
      });

      message.success("Backup imported successfully!");

      // Reset form
      importPassword.value = "";
      importRequiresPassword.value = false;

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
