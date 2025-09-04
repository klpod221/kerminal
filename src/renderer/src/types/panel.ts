export interface Tab {
  id: string
  title: string
  color?: string
  lastConnected?: Date
  profileId?: string // SSH Profile ID if this is an SSH connection
  groupId?: string // SSH Group ID if this SSH connection belongs to a group
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
}
