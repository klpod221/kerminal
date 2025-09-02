import { ref, computed, type Ref } from 'vue'
import {
  isRequired,
  hasMinLength,
  hasMaxLength,
  isValidHostname,
  isValidIP,
  isValidPort,
  isValidSSHKeyPath
} from '../utils/validation'

export type ValidationValue = string | number | undefined

export interface FieldValidation {
  value: Ref<ValidationValue>
  rules: ValidationRule[]
  touched: Ref<boolean>
  error: Ref<string | null>
}

export interface ValidationRule {
  name: string
  validate: (value: ValidationValue) => boolean
  message: string
}

export interface UseValidationReturn {
  fields: Ref<Record<string, FieldValidation>>
  registerField: (
    name: string,
    value: Ref<ValidationValue>,
    rules: ValidationRule[]
  ) => FieldValidation
  validateField: (name: string) => boolean
  validateAll: () => boolean
  resetValidation: () => void
  isFormValid: Ref<boolean>
  hasBeenTouched: Ref<boolean>
}

/**
 * Composable for form validation
 */
export function useValidation(): UseValidationReturn {
  const fields = ref<Record<string, FieldValidation>>({})

  /**
   * Register a field for validation
   */
  const registerField = (
    name: string,
    value: Ref<string | number | undefined>,
    rules: ValidationRule[]
  ): FieldValidation => {
    const touched = ref(false)
    const error = ref<string | null>(null)

    const field: FieldValidation = {
      value,
      rules,
      touched,
      error
    }

    fields.value[name] = field
    return field
  }

  /**
   * Validate a single field
   */
  const validateField = (name: string): boolean => {
    const field = fields.value[name]
    if (!field) return true

    field.touched.value = true
    field.error.value = null

    const value = field.value.value

    for (const rule of field.rules) {
      if (!rule.validate(value)) {
        field.error.value = rule.message
        return false
      }
    }

    return true
  }

  /**
   * Validate all fields
   */
  const validateAll = (): boolean => {
    let isValid = true

    for (const fieldName in fields.value) {
      if (!validateField(fieldName)) {
        isValid = false
      }
    }

    return isValid
  }

  /**
   * Reset all field states
   */
  const resetValidation = (): void => {
    for (const field of Object.values(fields.value)) {
      field.touched.value = false
      field.error.value = null
    }
  }

  /**
   * Check if form is valid
   */
  const isFormValid = computed(() => {
    return Object.values(fields.value).every(
      (field) => !field.error.value && (field.touched.value || field.value.value)
    )
  })

  /**
   * Check if any field has been touched
   */
  const hasBeenTouched = computed(() => {
    return Object.values(fields.value).some((field) => field.touched.value)
  })

  return {
    fields,
    registerField,
    validateField,
    validateAll,
    resetValidation,
    isFormValid,
    hasBeenTouched
  }
}

/**
 * Common validation rules
 */
export const validationRules = {
  required: (message = 'This field is required'): ValidationRule => ({
    name: 'required',
    validate: isRequired,
    message
  }),

  minLength: (min: number, message?: string): ValidationRule => ({
    name: 'minLength',
    validate: (value: ValidationValue) => hasMinLength(String(value || ''), min),
    message: message || `Minimum length is ${min} characters`
  }),

  maxLength: (max: number, message?: string): ValidationRule => ({
    name: 'maxLength',
    validate: (value: ValidationValue) => hasMaxLength(String(value || ''), max),
    message: message || `Maximum length is ${max} characters`
  }),

  hostname: (message = 'Please enter a valid hostname or IP address'): ValidationRule => ({
    name: 'hostname',
    validate: (value: ValidationValue) => {
      if (!value) return true // Optional field
      const str = String(value)
      return isValidHostname(str) || isValidIP(str)
    },
    message
  }),

  port: (message = 'Please enter a valid port number (1-65535)'): ValidationRule => ({
    name: 'port',
    validate: (value: ValidationValue) => {
      if (!value) return true // Optional field
      return isValidPort(value)
    },
    message
  }),

  sshKeyPath: (message = 'Please enter a valid SSH key file path'): ValidationRule => ({
    name: 'sshKeyPath',
    validate: (value: ValidationValue) => {
      if (!value) return true // Optional field
      return isValidSSHKeyPath(String(value))
    },
    message
  }),

  profileName: (message = 'Profile name must be 1-50 characters'): ValidationRule => ({
    name: 'profileName',
    validate: (value: ValidationValue) => {
      if (!value) return false
      const str = String(value).trim()
      return str.length >= 1 && str.length <= 50
    },
    message
  }),

  groupName: (message = 'Group name must be 1-30 characters'): ValidationRule => ({
    name: 'groupName',
    validate: (value: ValidationValue) => {
      if (!value) return false
      const str = String(value).trim()
      return str.length >= 1 && str.length <= 30
    },
    message
  }),

  username: (message = 'Please enter a valid username'): ValidationRule => ({
    name: 'username',
    validate: (value: ValidationValue) => {
      if (!value) return false
      const str = String(value).trim()
      // Basic username validation: alphanumeric, underscore, hyphen, dot
      const usernameRegex = /^[a-zA-Z0-9._-]+$/
      return usernameRegex.test(str) && str.length <= 32
    },
    message
  }),

  custom: (
    validate: (value: ValidationValue) => boolean,
    message: string,
    name = 'custom'
  ): ValidationRule => ({
    name,
    validate,
    message
  })
}
