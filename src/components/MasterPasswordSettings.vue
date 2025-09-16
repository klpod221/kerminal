<template>
  <div class="flex flex-col gap-6 max-w-4xl mx-auto p-4">
    <div class="flex items-center gap-4 p-4 bg-blue-100 rounded-lg border border-blue-400">
      <Shield :size="24" class="text-blue-500" />
      <div>
        <h3 class="text-xl font-semibold text-gray-100 mb-1">Master Password Settings</h3>
        <p class="text-sm text-gray-400">Manage security and encryption settings for your SSH profiles.</p>
      </div>
    </div>

    <!-- Status Section -->
    <Card title="Security Status" class="bg-gray-800 border border-gray-700">
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div class="flex flex-col gap-1">
          <div class="text-xs font-medium text-gray-400 uppercase tracking-wide">Status</div>
          <div class="flex items-center gap-2 text-sm font-medium text-gray-100">
            <span class="w-2 h-2 rounded-full" :class="isUnlocked ? 'bg-green-500' : 'bg-red-500'"></span>
            {{ isUnlocked ? 'Unlocked' : 'Locked' }}
          </div>
        </div>

        <div class="flex flex-col gap-1">
          <div class="text-xs font-medium text-gray-400 uppercase tracking-wide">Auto-unlock</div>
          <div class="text-sm font-medium text-gray-100">
            {{ status?.autoUnlockEnabled ? 'Enabled' : 'Disabled' }}
          </div>
        </div>

        <div class="flex flex-col gap-1">
          <div class="text-xs font-medium text-gray-400 uppercase tracking-wide">Session</div>
          <div class="text-sm font-medium text-gray-100">
            {{ status?.sessionActive ? 'Active' : 'Inactive' }}
          </div>
        </div>

        <div class="flex flex-col gap-1">
          <div class="text-xs font-medium text-gray-400 uppercase tracking-wide">Device</div>
          <div class="text-sm font-medium text-gray-100">
            {{ currentDevice?.name || 'Unknown' }}
          </div>
        </div>

        <div class="flex flex-col gap-1">
          <div class="text-xs font-medium text-gray-400 uppercase tracking-wide">Last unlock</div>
          <div class="text-sm font-medium text-gray-100">
            {{ status?.sessionExpiresAt ? 'Recently' : 'Unknown' }}
          </div>
        </div>

        <div class="flex flex-col gap-1">
          <div class="text-xs font-medium text-gray-400 uppercase tracking-wide">Created</div>
          <div class="text-sm font-medium text-gray-100">
            Unknown
          </div>
        </div>
      </div>
    </Card>

    <!-- Security Settings -->
    <Card title="Security Configuration" class="bg-gray-800 border border-gray-700">
      <div class="flex flex-col gap-6">
        <div class="flex justify-between items-start gap-4 p-4 bg-gray-700 rounded border border-gray-600">
          <div class="flex-1">
            <div class="text-sm font-medium text-gray-100 mb-1">Auto-unlock on startup</div>
            <div class="text-xs text-gray-400">
              Automatically unlock master password when the application starts using system keychain
            </div>
          </div>
          <div class="flex-shrink-0">
            <Checkbox
              id="auto-unlock-enabled"
              v-model="autoUnlockEnabled"
              :disabled="!isUnlocked || isLoading"
              @change="handleAutoUnlockToggle"
            />
          </div>
        </div>

        <div class="flex justify-between items-start gap-4 p-4 bg-gray-700 rounded border border-gray-600">
          <div class="flex-1">
            <div class="text-sm font-medium text-gray-100 mb-1">Session timeout</div>
            <div class="text-xs text-gray-400">
              Automatically lock the session after period of inactivity
            </div>
          </div>
          <div class="flex-shrink-0">
            <select
              v-model="sessionTimeoutMinutes"
              :disabled="!isUnlocked || isLoading"
              @change="handleSessionTimeoutChange"
              class="bg-gray-600 border border-gray-500 rounded px-3 py-1 text-gray-100 text-sm min-w-32 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="0">Never</option>
              <option value="5">5 minutes</option>
              <option value="15">15 minutes</option>
              <option value="30">30 minutes</option>
              <option value="60">1 hour</option>
              <option value="120">2 hours</option>
            </select>
          </div>
        </div>

        <div class="flex justify-between items-start gap-4 p-4 bg-gray-700 rounded border border-gray-600">
          <div class="flex-1">
            <div class="text-sm font-medium text-gray-100 mb-1">Require authentication for profile access</div>
            <div class="text-xs text-gray-400">
              Require master password verification before accessing encrypted SSH profiles
            </div>
          </div>
          <div class="flex-shrink-0">
            <Checkbox
              id="require-auth-access"
              v-model="requireAuthForAccess"
              :disabled="!isUnlocked || isLoading"
              @change="handleAuthRequirementToggle"
            />
          </div>
        </div>
      </div>
    </Card>

    <!-- Actions -->
    <Card title="Security Actions" class="bg-gray-800 border border-gray-700">
      <div class="flex flex-wrap gap-3">
        <Button
          variant="primary"
          @click="$emit('change-password')"
          :disabled="!isUnlocked || isLoading"
          class="flex items-center gap-2"
        >
          <Key :size="16" />
          Change Master Password
        </Button>

        <Button
          variant="secondary"
          @click="handleLock"
          :disabled="!isUnlocked || isLoading"
          class="flex items-center gap-2"
        >
          <Lock :size="16" />
          Lock Session
        </Button>

        <Button
          variant="danger"
          @click="$emit('reset-password')"
          :disabled="isLoading"
          class="flex items-center gap-2 ml-auto"
        >
          <Trash2 :size="16" />
          Reset Master Password
        </Button>
      </div>
    </Card>

    <!-- Device Information -->
    <Card title="Device Information" class="bg-gray-800 border border-gray-700" v-if="currentDevice">
      <div class="flex flex-col gap-4">
        <div class="flex justify-between items-center p-3 bg-gray-700 rounded">
          <div class="text-sm font-medium text-gray-400">Device ID</div>
          <div class="text-sm text-gray-100 font-mono">{{ currentDevice.id }}</div>
        </div>

        <div class="flex justify-between items-center p-3 bg-gray-700 rounded">
          <div class="text-sm font-medium text-gray-400">Device Name</div>
          <div class="text-sm text-gray-100 font-mono">{{ currentDevice.name }}</div>
        </div>

        <div class="flex justify-between items-center p-3 bg-gray-700 rounded">
          <div class="text-sm font-medium text-gray-400">Operating System</div>
          <div class="text-sm text-gray-100 font-mono">{{ 'Linux' }}</div>
        </div>

        <div class="flex justify-between items-center p-3 bg-gray-700 rounded">
          <div class="text-sm font-medium text-gray-400">First Seen</div>
          <div class="text-sm text-gray-100">
            {{ currentDevice.created ? formatDateTime(currentDevice.created) : 'Unknown' }}
          </div>
        </div>

        <div class="flex justify-between items-center p-3 bg-gray-700 rounded">
          <div class="text-sm font-medium text-gray-400">Last Accessed</div>
          <div class="text-sm text-gray-100">
            {{ currentDevice.lastVerified ? formatDateTime(currentDevice.lastVerified) : 'Never' }}
          </div>
        </div>
      </div>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Shield, Key, Lock, Trash2 } from 'lucide-vue-next'
import Card from './ui/Card.vue'
import Button from './ui/Button.vue'
import Checkbox from './ui/Checkbox.vue'

// State
const isUnlocked = ref(false)
const isLoading = ref(false)
const error = ref<string | null>(null)
const status = ref<any>(null)
const currentDevice = ref<any>(null)
const autoUnlockEnabled = ref(false)
const sessionTimeoutMinutes = ref(0)
const requireAuthForAccess = ref(false)

</script>

