<template>
  <Modal id="ssh-import-modal" title="Import from SSH Config" size="lg">
    <div class="space-y-4">
      <div v-if="loading" class="flex justify-center py-8">
        <div
          class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"
        ></div>
      </div>

      <div v-else-if="error" class="bg-red-500/10 text-red-400 p-4 rounded-lg">
        {{ error }}
      </div>

      <div v-else class="space-y-4">
        <p class="text-gray-400 text-sm">
          Select hosts to import as SSH profiles. Existing profiles with the
          same name will be skipped unless you choose to overwrite.
        </p>

        <div class="border border-gray-700 rounded-lg overflow-hidden">
          <div
            class="bg-gray-800/50 px-4 py-2 border-b border-gray-700 flex items-center justify-between"
          >
            <div class="flex items-center space-x-2">
              <input
                type="checkbox"
                :checked="allSelected"
                @change="toggleAll"
                class="rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-offset-gray-900"
              />
              <span class="text-sm font-medium text-gray-300">Select All</span>
            </div>
            <span class="text-sm text-gray-400"
              >{{ selectedCount }} selected</span
            >
          </div>

          <div class="max-h-[400px] overflow-y-auto">
            <div
              v-for="host in hosts"
              :key="host.name"
              class="px-4 py-3 border-b border-gray-700 last:border-0 hover:bg-gray-800/30 transition-colors flex items-center space-x-3 cursor-pointer"
              @click="toggleHost(host)"
            >
              <input
                type="checkbox"
                :checked="selectedHosts.has(host.name)"
                class="rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-offset-gray-900"
                @click.stop="toggleHost(host)"
              />
              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between">
                  <span class="font-medium text-white truncate">{{
                    host.name
                  }}</span>
                  <span class="text-xs text-gray-500">{{ host.hostname }}</span>
                </div>
                <div
                  class="text-xs text-gray-400 mt-0.5 flex items-center space-x-2"
                >
                  <span v-if="host.user">User: {{ host.user }}</span>
                  <span v-if="host.port">Port: {{ host.port }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-between w-full">
        <Button
          type="button"
          variant="secondary"
          @click="closeOverlay('ssh-import-modal')"
        >
          Cancel
        </Button>
        <Button
          type="button"
          variant="primary"
          :icon="Download"
          :disabled="selectedCount === 0 || loading"
          @click="handleImport"
        >
          Import {{ selectedCount > 0 ? `(${selectedCount})` : "" }}
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { Download } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import { useSSHStore } from "../../stores/ssh";
import { message } from "../../utils/message";

const { closeOverlay, isOverlayVisible } = useOverlay();
const sshStore = useSSHStore();

const loading = ref(false);
const error = ref<string | null>(null);
const selectedHosts = ref<Set<string>>(new Set());

const hosts = computed(() => sshStore.configHosts);
const selectedCount = computed(() => selectedHosts.value.size);
const allSelected = computed(
  () =>
    hosts.value.length > 0 && selectedHosts.value.size === hosts.value.length,
);

const fetchHosts = async () => {
  loading.value = true;
  error.value = null;
  try {
    await sshStore.loadConfigHosts();
  } catch (e: any) {
    error.value = e.message || "Failed to load SSH config hosts";
  } finally {
    loading.value = false;
  }
};

const toggleAll = () => {
  if (allSelected.value) {
    selectedHosts.value.clear();
  } else {
    hosts.value.forEach((h) => selectedHosts.value.add(h.name));
  }
};

const toggleHost = (host: any) => {
  if (selectedHosts.value.has(host.name)) {
    selectedHosts.value.delete(host.name);
  } else {
    selectedHosts.value.add(host.name);
  }
};

const handleImport = async () => {
  if (selectedCount.value === 0) return;

  const hostsToImport = hosts.value.filter((h) =>
    selectedHosts.value.has(h.name),
  );

  try {
    let importedCount = 0;
    for (const host of hostsToImport) {
      // Check if profile exists
      const exists = sshStore.profiles.some((p) => p.name === host.name);
      if (exists) {
        continue;
      }

      await sshStore.createProfile({
        name: host.name,
        host: host.hostname,
        port: host.port || 22,
        username: host.user || "root",
        authMethod: host.identityFile ? "KeyReference" : "Password",
        authData: host.identityFile
          ? { KeyReference: { keyId: "" } }
          : { Password: { password: "" } },
      });
      importedCount++;
    }

    message.success(`Imported ${importedCount} profiles`);
    closeOverlay("ssh-import-modal");
    selectedHosts.value.clear();
  } catch (e: any) {
    message.error(`Import failed: ${e.message}`);
  }
};

// Reset when modal opens
watch(
  () => isOverlayVisible("ssh-import-modal"),
  (visible) => {
    if (visible) {
      fetchHosts();
      selectedHosts.value.clear();
    }
  },
);
</script>
