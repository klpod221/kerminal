export interface SecuritySettings {
  requirePasswordOnStart: boolean
  autoLockTimeout: number // in minutes (0 = never)
  useBiometrics?: boolean // for future implementation
}

export interface MongoConnectionConfig {
  mongoUri: string
  databaseName: string
  masterPassword: string
}

export type MasterPasswordCreateMode = 'local' | 'mongodb'

export interface MasterPasswordModalProps {
  visible: boolean
  error?: string
}

export interface MasterPasswordModalEmits {
  localCreated: [password: string, settings: SecuritySettings]
  mongoConnected: [config: MongoConnectionConfig]
  unlocked: []
}

export interface SecuritySettingsModalProps {
  visible: boolean
}
