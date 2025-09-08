export interface SecuritySettings {
  requirePasswordOnStart: boolean
  autoLockTimeout: number // in minutes (0 = never)
  useBiometrics?: boolean // for future implementation
}

export interface MasterPasswordModalProps {
  visible: boolean
  mode: 'create' | 'unlock'
  error?: string
}

export interface SecuritySettingsModalProps {
  visible: boolean
}
