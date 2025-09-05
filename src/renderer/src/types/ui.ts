import type { Component } from 'vue'

/**
 * Common UI size variants
 */
export type UISize = 'sm' | 'md' | 'lg'

/**
 * Button variant types
 */
export type ButtonVariant =
  | 'primary'
  | 'secondary'
  | 'success'
  | 'warning'
  | 'danger'
  | 'ghost'
  | 'outline'

/**
 * Input types
 */
export type InputType = 'text' | 'password' | 'email' | 'number' | 'tel' | 'url' | 'search'

/**
 * Modal size variants
 */
export type ModalSize = 'sm' | 'md' | 'lg' | 'xl' | '2xl'

/**
 * Message types
 */
export type MessageType = 'success' | 'error' | 'warning' | 'info'

/**
 * Message positions
 */
export type MessagePosition =
  | 'top'
  | 'top-right'
  | 'top-left'
  | 'bottom'
  | 'bottom-right'
  | 'bottom-left'

/**
 * PopConfirm placements
 */
export type PopConfirmPlacement =
  | 'top'
  | 'bottom'
  | 'left'
  | 'right'
  | 'topLeft'
  | 'topRight'
  | 'bottomLeft'
  | 'bottomRight'

/**
 * Drawer positions
 */
export type DrawerPosition = 'left' | 'right'

/**
 * Drawer widths
 */
export type DrawerWidth = 'sm' | 'md' | 'lg' | 'xl' | '2xl'

/**
 * Props interface for Button component
 */
export interface ButtonProps {
  variant?: ButtonVariant
  size?: UISize
  icon?: Component
  iconRight?: boolean
  text?: string
  loading?: boolean
  title?: string
  disabled?: boolean
}

/**
 * Props interface for Card component
 */
export interface CardProps {
  /**
   * Card title displayed in header
   */
  title?: string
  /**
   * Icon component to display in header
   */
  icon?: Component
  /**
   * Background color class for icon container
   */
  iconBackground?: string
  /**
   * Color class for icon
   */
  iconColor?: string
  /**
   * Card size variant
   */
  size?: UISize
  /**
   * Enable hover effects
   */
  hover?: boolean
  /**
   * Enable scale transform on hover
   */
  scale?: boolean
  /**
   * Remove default padding
   */
  noPadding?: boolean
  /**
   * Add default spacing between content elements
   */
  spacing?: boolean
  /**
   * Center align content (for action buttons)
   */
  center?: boolean
  /**
   * Custom CSS classes to apply
   */
  customClass?: string
}

/**
 * Emits interface for Card component
 */
export interface CardEmits {
  /**
   * Emitted when card is clicked
   */
  click: [event: MouseEvent]
}

/**
 * Props interface for Input component
 */
export interface InputProps {
  modelValue?: string | number
  type?: InputType
  label?: string
  placeholder?: string
  rules?: Array<string | ((value: string) => boolean)>
  helperText?: string
  errorMessage?: string
  size?: UISize
  leftIcon?: Component
  rightIcon?: Component
  disabled?: boolean
  readonly?: boolean
  autocomplete?: string
  helper?: boolean
  id?: string
}

/**
 * Props interface for Modal component
 */
export interface ModalProps {
  visible?: boolean
  title?: string
  icon?: Component
  iconBackground?: string
  iconColor?: string
  showCloseButton?: boolean
  closeOnBackdrop?: boolean
  size?: ModalSize
}

/**
 * Props interface for Drawer component
 */
export interface DrawerProps {
  visible?: boolean
  title?: string
  icon?: Component
  iconBackground?: string
  iconColor?: string
  position?: DrawerPosition
  width?: DrawerWidth
  closeOnOverlay?: boolean
}

/**
 * Props interface for Select component
 */
export interface SelectProps {
  modelValue?: string | number
  label?: string
  placeholder?: string
  rules?: Array<string | ((value: string) => boolean)>
  helperText?: string
  errorMessage?: string
  size?: UISize
  disabled?: boolean
  helper?: boolean
  id?: string
}

/**
 * Props interface for Textarea component
 */
export interface TextareaProps {
  modelValue?: string
  label?: string
  placeholder?: string
  rules?: Array<string | ((value: string) => boolean)>
  helperText?: string
  errorMessage?: string
  size?: UISize
  disabled?: boolean
  readonly?: boolean
  helper?: boolean
  rows?: number
  id?: string
}

/**
 * Props interface for ColorPicker component
 */
export interface ColorPickerProps {
  modelValue?: string
  label?: string
  rules?: Array<string | ((value: string) => boolean)>
  helperText?: string
  errorMessage?: string
  size?: UISize
  disabled?: boolean
  readonly?: boolean
  helper?: boolean
  id?: string
}

/**
 * Props interface for Checkbox component
 */
export interface CheckboxProps {
  modelValue?: boolean
  label?: string
  rules?: Array<string | ((value: boolean) => boolean)>
  helperText?: string
  errorMessage?: string
  size?: UISize
  disabled?: boolean
  helper?: boolean
  labelPosition?: 'top' | 'right'
  id?: string
}

/**
 * Props interface for Message component
 */
export interface MessageProps {
  type?: MessageType | 'loading'
  title?: string
  content: string
  duration?: number
  closable?: boolean
  onClose?: () => void
}

/**
 * Props interface for PopConfirm component
 */
export interface PopConfirmProps {
  title?: string
  content: string
  okText?: string
  cancelText?: string
  okType?: 'primary' | 'danger'
  trigger?: 'click' | 'hover'
  placement?: PopConfirmPlacement
  disabled?: boolean
}

/**
 * Props interface for Tab component
 */
export interface TabProps {
  tab: import('./panel').Tab
  isActive: boolean
  isConnecting?: boolean
  minWidth: number
  maxWidth: number
  panelId: string
}

/**
 * Tab component emits
 */
export interface TabEmits {
  select: []
  close: []
  dragStart: [tab: import('./panel').Tab]
  drop: [draggedTab: import('./panel').Tab, targetTab: import('./panel').Tab]
  duplicate: [tab: import('./panel').Tab]
  closeOthers: [tab: import('./panel').Tab]
  closeToRight: [tab: import('./panel').Tab]
  moveToNewPanel: [tab: import('./panel').Tab]
}

/**
 * Context menu item interface
 */
export interface ContextMenuItem {
  id: string
  label?: string // Optional for dividers
  icon?: unknown // Accept any icon component (Lucide, etc.)
  shortcut?: string
  danger?: boolean
  disabled?: boolean
  type?: 'item' | 'divider'
  action?: string
}

/**
 * Props interface for ContextMenu component
 */
export interface ContextMenuProps {
  items: ContextMenuItem[]
}

/**
 * ContextMenu component emits
 */
export interface ContextMenuEmits {
  itemClick: [item: ContextMenuItem]
}
