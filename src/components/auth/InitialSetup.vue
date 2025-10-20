<template>
  <Modal
    id="initial-setup"
    :show-close-button="false"
    :close-on-backdrop="false"
    :close-on-esc="false"
    title="Welcome to Kerminal"
    size="md"
  >
    <div class="flex flex-col gap-6">
      <Card>
        <div class="flex items-start gap-4">
          <img
            src="../../assets/images/logo_500.png"
            alt="Kerminal Logo"
            class="w-20 h-20"
          />
          <div>
            <h3 class="text-lg font-semibold text-gray-100 mb-2">
              Get Started with Kerminal
            </h3>
            <p class="text-sm text-gray-400">
              Modern SSH terminal with cross-device sync capabilities
            </p>
          </div>
        </div>
      </Card>

      <div class="space-y-4">
        <h4 class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2">
          Do you have existing data to restore?
        </h4>

        <div class="grid grid-cols-2 gap-4">
          <Card
            :hover="true"
            class="cursor-pointer border-2 transition-all duration-200"
            :class="
              selectedOption === 'new'
                ? 'border-green-500 bg-green-500/10 ring-2 ring-green-500/30'
                : 'border-gray-700 hover:border-gray-600'
            "
            @click="selectedOption = 'new'"
          >
            <div class="flex flex-col items-center gap-3 py-2">
              <div
                class="w-12 h-12 rounded-full flex items-center justify-center transition-all duration-200"
                :class="selectedOption === 'new' ? 'bg-green-500/30' : 'bg-green-500/20'"
              >
                <PlusCircle :size="24" class="text-green-400" />
              </div>
              <div class="text-center">
                <h5 class="font-semibold text-gray-100 mb-1">New Setup</h5>
                <p class="text-xs text-gray-400">
                  Start fresh with new configuration
                </p>
              </div>
            </div>
          </Card>

          <Card
            :hover="true"
            class="cursor-pointer border-2 transition-all duration-200"
            :class="
              selectedOption === 'restore'
                ? 'border-blue-500 bg-blue-500/10 ring-2 ring-blue-500/30'
                : 'border-gray-700 hover:border-gray-600'
            "
            @click="selectedOption = 'restore'"
          >
            <div class="flex flex-col items-center gap-3 py-2">
              <div
                class="w-12 h-12 rounded-full flex items-center justify-center transition-all duration-200"
                :class="selectedOption === 'restore' ? 'bg-blue-500/30' : 'bg-blue-500/20'"
              >
                <Download :size="24" class="text-blue-400" />
              </div>
              <div class="text-center">
                <h5 class="font-semibold text-gray-100 mb-1">Restore Data</h5>
                <p class="text-xs text-gray-400">
                  Pull existing data from sync server
                </p>
              </div>
            </div>
          </Card>
        </div>

        <div
          v-if="selectedOption === 'restore'"
          class="bg-yellow-900/20 border border-yellow-700/50 rounded-lg p-4"
        >
          <div class="flex items-start gap-2">
            <AlertCircle :size="20" class="text-yellow-500 mt-0.5 flex-shrink-0" />
            <div class="flex-1">
              <h4 class="text-sm font-medium text-yellow-200 mb-1">Important</h4>
              <p class="text-xs text-yellow-100/80">
                You'll need your database connection details and master password to restore data.
                This will download your profiles, keys, and settings from the sync server.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button
          variant="primary"
          :disabled="!selectedOption"
          :icon="selectedOption === 'restore' ? Download : ArrowRight"
          @click="handleContinue"
        >
          {{ selectedOption === 'restore' ? 'Restore Data' : 'Continue' }}
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { PlusCircle, Download, AlertCircle, ArrowRight } from "lucide-vue-next";
import { useOverlay } from "../../composables/useOverlay";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import Card from "../ui/Card.vue";

const { closeOverlay, openOverlay } = useOverlay();

const selectedOption = ref<'new' | 'restore' | null>(null);

const handleContinue = () => {
  if (!selectedOption.value) return;

  closeOverlay("initial-setup");

  if (selectedOption.value === 'restore') {
    openOverlay("restore-from-sync");
  } else {
    openOverlay("master-password-setup");
  }
};
</script>
