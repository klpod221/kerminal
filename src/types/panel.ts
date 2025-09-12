export interface Tab {
  id: string
  title: string
  color?: string
  lastConnected?: Date
  profileId?: string
  groupId?: string
}

export interface Panel {
  id: string
  activeTabId: string
  tabs: Tab[]
}

export interface PanelLayout {
  type: 'panel' | 'split'
  id: string
  panel?: Panel // if type = 'panel'
  children?: PanelLayout[] // if type = 'split'
  direction?: 'horizontal' | 'vertical' // if type = 'split'
  sizes?: number[] // size ratios for children
}

export interface TerminalInstance {
  id: string
  ready: boolean
  isSSHConnecting?: boolean
}
