<template>
  <div class="space-y-6">
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
      <p class="text-xs text-blue-400">
        <strong>Note:</strong> Your data will be encrypted with the master password.
      </p>
    </div>

    <!-- Form -->
    <form class="space-y-4" @submit.prevent="handleSubmit">
      <!-- MongoDB URI -->
      <Input
        v-model="form.mongoUri"
        label="MongoDB URI"
        placeholder="mongodb://username:password@hostname:port"
        :error-message="mongoUriError"
        helper-text="Include credentials in the URI"
        @blur="validation.validateField('mongoUri')"
      />

      <!-- Database Name -->
      <Input
        v-model="form.databaseName"
        label="Database Name"
        placeholder="kerminal"
        :error-message="databaseNameError"
        @blur="validation.validateField('databaseName')"
      />

      <!-- Test Connection Button -->
      <div class="flex justify-start">
        <Button
          type="button"
          variant="secondary"
          size="sm"
          :loading="isTestingConnection"
          :disabled="!canTestConnection"
          @click="testConnection"
        >
          {{
            connectionTestResult === null
              ? 'Test Connection'
              : connectionTestResult
                ? 'Connection OK'
                : 'Connection Failed'
          }}
        </Button>
      </div>

      <!-- Connection Test Status -->
      <Message
        v-if="connectionTestResult === true"
        type="success"
        title="Connection Successful"
        content="MongoDB connection established successfully."
        :closable="false"
      />
      <Message
        v-else-if="connectionTestResult === false"
        type="error"
        title="Connection Failed"
        content="Failed to connect to MongoDB. Please check your connection details."
        :closable="false"
      />

      <!-- MongoDB Master Password (only show if connection successful) -->
      <div v-if="connectionTestResult === true">
        <Input
          v-model="form.masterPassword"
          :label="
            mongoHasMasterPassword
              ? 'Existing MongoDB Master Password'
              : 'Create New Master Password'
          "
          :type="showMasterPassword ? 'text' : 'password'"
          :placeholder="
            mongoHasMasterPassword
              ? 'Enter existing MongoDB master password'
              : 'Create a secure master password'
          "
          :error-message="masterPasswordError"
          :right-icon="showMasterPassword ? EyeOff : Eye"
          @right-icon-click="showMasterPassword = !showMasterPassword"
          @blur="validation.validateField('masterPassword')"
        />
        <p class="text-xs text-gray-400 mt-1">
          <template v-if="mongoHasMasterPassword">
            This is the master password that was used to encrypt data in your MongoDB database.
          </template>
          <template v-else>
            This password will be used to encrypt your data. Make sure to remember it as it cannot
            be recovered.
          </template>
        </p>
      </div>

      <!-- Loading State -->
      <Message
        v-if="isProcessing"
        type="loading"
        :title="mongoHasMasterPassword ? 'Importing from MongoDB' : 'Creating MongoDB Connection'"
        :content="
          mongoHasMasterPassword
            ? 'Verifying credentials and importing existing data...'
            : 'Creating master password and setting up MongoDB sync...'
        "
        :closable="false"
      />

      <!-- Submit Button -->
      <div class="flex justify-end w-full pt-4">
        <Button
          variant="primary"
          size="sm"
          :disabled="!canSubmit || isProcessing"
          :loading="isProcessing"
          :icon="Database"
          type="submit"
        >
          {{ mongoHasMasterPassword ? 'Import from MongoDB' : 'Create & Connect' }}
        </Button>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Database, Eye, EyeOff } from 'lucide-vue-next'
import Input from '../ui/Input.vue'
import Button from '../ui/Button.vue'
import Message from '../ui/Message.vue'
import { useValidation, validationRules } from '../../composables/useValidation'
import type { MongoConnectionConfig } from '../../types/auth'

const emit = defineEmits<{
  submit: [config: MongoConnectionConfig, hasExistingMasterPassword: boolean]
}>()

defineProps<{
  isProcessing: boolean
}>()

// State
const showMasterPassword = ref(false)
const isTestingConnection = ref(false)
const connectionTestResult = ref<boolean | null>(null)
const mongoHasMasterPassword = ref<boolean | null>(null)

// Form data
const form = ref({
  mongoUri: '',
  databaseName: 'kerminal',
  masterPassword: ''
})

// Validation setup
const validation = useValidation()

// Create reactive refs for validation
const mongoUriRef = computed({
  get: () => form.value.mongoUri,
  set: (value) => {
    form.value.mongoUri = value
  }
})

const databaseNameRef = computed({
  get: () => form.value.databaseName,
  set: (value) => {
    form.value.databaseName = value
  }
})

const masterPasswordRef = computed({
  get: () => form.value.masterPassword,
  set: (value) => {
    form.value.masterPassword = value
  }
})

// Register validation fields
validation.registerField('mongoUri', mongoUriRef, [
  validationRules.required('MongoDB URI is required'),
  validationRules.custom((value) => {
    // Basic MongoDB URI validation
    if (typeof value !== 'string') return false
    return value.includes('mongodb://') || value.includes('mongodb+srv://')
  }, 'Invalid MongoDB URI format')
])

validation.registerField('databaseName', databaseNameRef, [
  validationRules.required('Database name is required'),
  validationRules.minLength(1, 'Database name cannot be empty')
])

validation.registerField('masterPassword', masterPasswordRef, [
  validationRules.custom((value) => {
    // Only validate master password if connection test passed
    if (connectionTestResult.value === true) {
      if (typeof value !== 'string') return false
      const trimmedValue = value.trim()

      if (mongoHasMasterPassword.value === true) {
        // Existing password - just check it's not empty
        return trimmedValue.length > 0
      } else if (mongoHasMasterPassword.value === false) {
        // New password - must be at least 8 characters
        return trimmedValue.length >= 8
      } else {
        // Status unknown, require basic length
        return trimmedValue.length >= 8
      }
    }
    return true
  }, 'Master password is required')
])

// Error message computed properties
const mongoUriError = computed(() => validation.fields.value?.mongoUri?.error?.value || undefined)
const databaseNameError = computed(
  () => validation.fields.value?.databaseName?.error?.value || undefined
)
const masterPasswordError = computed(
  () => validation.fields.value?.masterPassword?.error?.value || undefined
)

// Computed
const canTestConnection = computed(() => {
  return form.value.mongoUri.trim().length > 0 && form.value.databaseName.trim().length > 0
})

const canSubmit = computed(() => {
  const hasUri = form.value.mongoUri.trim().length > 0
  const hasDbName = form.value.databaseName.trim().length > 0
  const hasMasterPassword = form.value.masterPassword.trim().length > 0
  const connectionTested = connectionTestResult.value === true
  const isValid = !mongoUriError.value && !databaseNameError.value && !masterPasswordError.value

  return hasUri && hasDbName && hasMasterPassword && connectionTested && isValid
})

// Methods
const testConnection = async (): Promise<void> => {
  if (!canTestConnection.value) {
    validation.validateField('mongoUri')
    validation.validateField('databaseName')
    return
  }

  isTestingConnection.value = true
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
    }
  } catch (error) {
    console.error('Connection test failed:', error)
    connectionTestResult.value = false
    mongoHasMasterPassword.value = null
  } finally {
    isTestingConnection.value = false
  }
}

const handleSubmit = async (): Promise<void> => {
  if (!canSubmit.value) {
    validation.validateAll()
    return
  }

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
    showMasterPassword.value = false
    connectionTestResult.value = null
    mongoHasMasterPassword.value = null
    if (validation.fields.value && Object.keys(validation.fields.value).length > 0) {
      validation.resetValidation()
    }
  }
})
</script>
