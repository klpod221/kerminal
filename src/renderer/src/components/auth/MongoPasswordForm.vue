<template>
  <div class="space-y-2">
    <!-- Description -->
    <div class="text-sm text-gray-300 leading-relaxed">
      <p class="mb-3">
        <template v-if="mongoHasMasterPassword === null">
          Connect to your MongoDB database. After testing the connection, you can either import
          existing encrypted data or create a new master password.
        </template>
        <template v-else-if="mongoHasMasterPassword === true">
          Connect to your existing MongoDB database and import your master password. This will
          synchronize your encrypted data across devices.
        </template>
        <template v-else>
          Connect to your MongoDB database and create a new master password. Your data will be
          encrypted and stored securely.
        </template>
      </p>
    </div>

    <!-- Form -->
    <Form @submit="isTest ? testConnection : handleSubmit">
      <!-- MongoDB URI -->
      <Input
        id="mongo-uri"
        v-model="form.mongoUri"
        label="MongoDB URI"
        rules="required"
        placeholder="mongodb://username:password@hostname:port"
      />

      <!-- Database Name -->
      <Input
        id="database-name"
        v-model="form.databaseName"
        label="Database Name"
        rules="required"
        placeholder="kerminal"
      />

      <!-- Connection Status -->

      <!-- MongoDB Master Password (only show if connection successful) -->
      <div v-if="connectionTestResult === true">
        <Input
          id="master-password"
          v-model="form.masterPassword"
          rules="required|min:8"
          :label="
            mongoHasMasterPassword
              ? 'Existing MongoDB Master Password'
              : 'Create New Master Password'
          "
          type="password"
          :placeholder="
            mongoHasMasterPassword
              ? 'Enter existing MongoDB master password'
              : 'Create a secure master password'
          "
        />
        <p class="text-xs text-yellow-400">
          <template v-if="mongoHasMasterPassword">
            <strong>Important:</strong> This is the master password that was used to encrypt data in
            your MongoDB database.
          </template>
          <template v-else>
            <strong>Important:</strong> This password will be used to encrypt your data. Make sure
            to remember it as it cannot be recovered.
          </template>
        </p>
      </div>

      <!-- Submit Button -->
      <div class="flex justify-end w-full pt-4">
        <Button
          v-if="!connectionTestResult"
          type="submit"
          variant="primary"
          :icon="Monitor"
          size="sm"
          :loading="isTesting"
          @click="testConnection"
        >
          Test Connection
        </Button>

        <Button
          v-else
          variant="primary"
          size="sm"
          :loading="loading"
          :icon="Database"
          type="submit"
          @click="handleSubmit"
        >
          {{ mongoHasMasterPassword ? 'Import from MongoDB' : 'Create & Connect' }}
        </Button>
      </div>
    </Form>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { Database, Monitor } from 'lucide-vue-next'
import Form from '../ui/Form.vue'
import Input from '../ui/Input.vue'
import Button from '../ui/Button.vue'
import type { MongoConnectionConfig } from '../../types/auth'
import { message } from '@renderer/utils/message'

const emit = defineEmits<{
  submit: [config: MongoConnectionConfig, hasExistingMasterPassword: boolean]
}>()

defineProps<{
  loading: boolean
}>()

// State
const isTest = ref(true)
const isTesting = ref(false)
const connectionTestResult = ref<boolean | null>(null)
const mongoHasMasterPassword = ref<boolean | null>(null)

// Form data
const form = ref({
  mongoUri: '',
  databaseName: 'kerminal',
  masterPassword: ''
})

// Methods
const testConnection = async (): Promise<void> => {
  isTesting.value = true
  connectionTestResult.value = null
  mongoHasMasterPassword.value = null

  try {
    const success = await window.api.invoke(
      'sync.testConnection',
      form.value.mongoUri,
      form.value.databaseName
    )
    connectionTestResult.value = Boolean(success)

    // If connection successful, check if MongoDB has master password data
    if (success) {
      mongoHasMasterPassword.value = (await window.api.invoke(
        'auth:check-mongo-master-password-exists',
        form.value.mongoUri,
        form.value.databaseName
      )) as boolean
      isTest.value = false
    } else {
      message.error('Failed to connect to MongoDB. Please check your URI and database name.')
    }
  } catch (error) {
    console.error('Connection test failed:', error)
    connectionTestResult.value = false
    mongoHasMasterPassword.value = null
  } finally {
    isTesting.value = false
  }
}

const handleSubmit = async (): Promise<void> => {
  emit(
    'submit',
    {
      mongoUri: form.value.mongoUri,
      databaseName: form.value.databaseName,
      masterPassword: form.value.masterPassword
    },
    mongoHasMasterPassword.value ?? false
  )
}

// Reset form function (exposed for parent component)
defineExpose({
  resetForm: () => {
    form.value = {
      mongoUri: '',
      databaseName: 'kerminal',
      masterPassword: ''
    }
    connectionTestResult.value = null
    mongoHasMasterPassword.value = null
  }
})

// If connection details change, reset connection test status
watch(
  () => [form.value.mongoUri, form.value.databaseName],
  () => {
    connectionTestResult.value = null
    mongoHasMasterPassword.value = null
    form.value.masterPassword = ''
    isTest.value = true
  }
)
</script>
