import { Ref, Component } from 'vue'

/**
 * Common UI size variants
 */
export type UISize = 'sm' | 'md' | 'lg'

/**
 * Input types
 */
export type InputType = 'text' | 'password' | 'email' | 'number' | 'tel' | 'url' | 'search'

/**
 * Props interface for Form component
 */
export interface FormContext {
  register: (field: FormField) => void
  unregister: (id: string) => void
  getFieldValue: (id: string) => unknown
  getAllFieldValues: () => { [key: string]: unknown }
}

/**
 * Interface representing a form field
 */
export interface FormField {
  id: string
  value: Ref<unknown>
  validate: () => string
}

/**
 * Props interface for Input component
 */
export interface InputProps {
  id: string
  modelValue?: string | number
  type?: InputType
  label?: string
  placeholder?: string
  rules?: string
  helperText?: string
  errorMessage?: string
  size?: UISize
  leftIcon?: Component
  rightIcon?: Component
  disabled?: boolean
  readonly?: boolean
  autocomplete?: string
  helper?: boolean
}

/**
 * Props interface for Input component
 */
export interface SelectProps {
  id: string
  modelValue?: string | number
  label?: string
  placeholder?: string
  rules?: string
  helperText?: string
  errorMessage?: string
  size?: UISize
  leftIcon?: Component
  rightIcon?: Component
  disabled?: boolean
  readonly?: boolean
  autocomplete?: string
  helper?: boolean
}

/**
 * Props interface for Checkbox component
 */
export interface CheckboxProps {
  id: string
  modelValue?: boolean
  label?: string
  rules?: string
  helperText?: string
  errorMessage?: string
  size?: UISize
  disabled?: boolean
  readonly?: boolean
  helper?: boolean
}

/**
 * Props interface for Text Area component
 */
export interface TextareaProps {
  id: string
  modelValue?: string | number
  rows?: number
  label?: string
  placeholder?: string
  rules?: string
  helperText?: string
  errorMessage?: string
  size?: UISize
  disabled?: boolean
  readonly?: boolean
  autocomplete?: string
  helper?: boolean
}

/**
 * Props interface for Color Picker component
 */
export interface ColorPickerProps {
  id: string
  modelValue?: string
  label?: string
  rules?: string
  helperText?: string
  errorMessage?: string
  size?: UISize
  disabled?: boolean
  readonly?: boolean
  helper?: boolean
}
