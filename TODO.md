# 🚀 Kerminal Todo List

This is a list of todos for the Kerminal application. Please check the [README.md](README.md) for the current features and progress. Feel free to contribute to the project by adding new features or improving the existing ones.

---

## SFTP Browser

- [ ] **File search**
  - Search files in current directory
  - Filter by name, extension, size, date
  - Search by content, regex filters
  - Hotkey: Ctrl+F to focus search

- [ ] **Batch selection improvements**
  - Select All / Deselect All
  - Invert Selection
  - Filter selection (select all files, select all directories)
  - Keyboard shortcuts for selection

- [ ] **Quick actions toolbar**
  - Quick download/upload buttons for selected files
  - Context menu improvements with more actions
  - Bulk operations (rename, delete, move multiple files)

- [ ] **Transfer queue management**
  - Pause/Resume individual transfers
  - Retry failed transfers
  - Priority queue (prioritize certain transfers)
  - Queue reordering
  - Auto-retry with exponential backoff

- [ ] **File history/Bookmarks**
  - Bookmark frequently used folders
  - Recent paths history
  - Quick navigation sidebar

- [ ] **Image preview improvements**
  - Zoom, pan for images
  - Image gallery view (next/previous)
  - Fullscreen mode

---

## Terminal & Editor

- [ ] **Smarter terminal buffer management**
  - Smartly load only the necessary lines of the terminal buffer
  - Load only the lines that are visible in the terminal
  - Load only the lines that are needed

- [ ] **Command palette**
  - Ctrl+P / Ctrl+K to open command palette
  - Quick access to all features and commands
  - Fuzzy search

- [ ] **Multi-file editing**
  - Tabbed editor interface for multiple files
  - Close tabs, unsaved changes warning
  - Tab switching shortcuts

- [ ] **Connection indicator in terminal tabs**
  - Visual indicator for SSH connection status
  - Show connection info (host, latency, etc.)

- [ ] **Syntax highlighting improvements**
  - Add more languages
  - Custom syntax themes
  - Language auto-detection improvements

- [ ] **File watcher**
  - Reload when file changes externally
  - Warning when file modified externally
  - Option to disable watcher

- [ ] **Open file with default application**
  - Open file with default application (System file picker)

- [ ] **Split terminal shortcuts**
  - Keyboard shortcuts to split quickly
  - Split preset configurations
  - Quick split templates

- [ ] **Terminal profiles**
  - Save preset terminals
  - Working directory presets
  - Shell configuration presets
  - Environment variables presets

- [ ] **Command history search**
  - Search in terminal session history
  - Filter history by command
  - Export history

- [ ] **Multiple cursor selection**
  - Multi-cursor support for copy/paste
  - Column selection mode

- [ ] **Compare files**
  - Side-by-side diff view
  - Merge files
  - Visual diff highlighting

---

## User Experience

- [ ] **Customizable keyboard shortcuts**
  - Settings page for keyboard shortcuts
  - Remap shortcuts according to preference
  - Conflict detection

- [ ] **Auto-save for file editor**
  - Auto-save after delay (configurable)
  - Auto-save before disconnect
  - Visual indicator for unsaved changes

- [ ] **Tooltips and onboarding**
  - Contextual tooltips for features
  - First-time user tour
  - Interactive tutorials

- [ ] **Progress indicators improvements**
  - Better loading states for long operations
  - Skeleton loaders
  - Cancellable operations

- [ ] **Notification system**
  - Desktop notifications for transfers
  - Sync completion notifications
  - Error notifications
  - Notification preferences

---

## SSH Management

- [ ] **Connection groups with tags**
  - Tag-based filtering instead of just groups
  - Multiple tags per profile
  - Tag colors and organization

- [ ] **Recent connections**
  - Quick access menu with recently used connections
  - Connection history tracking

- [ ] **SSH config import**
  - Import profiles from `~/.ssh/config`
  - Batch import with conflict resolution

- [ ] **Connection health monitoring**
  - Periodic connection testing
  - Connection status indicators
  - Auto-reconnect on failure

- [ ] **Connection templates**
  - Create SSH profiles from templates
  - Template library
  - Share templates

- [ ] **More SSH Authentication Methods**
  - Additional authentication methods support
  - Enhanced security options

---

## Sync & Security

- [ ] **Sync filters**
  - Choose sync by profile/group
  - Include/exclude specific data types
  - Sync scheduling

- [ ] **Sync history**
  - Log all sync operations
  - View sync conflicts history
  - Export sync log

- [ ] **Backup/Restore**
  - Export encrypted data
  - Import backup
  - Scheduled backups

- [ ] **Cloud backup integration**
  - Cloud storage providers integration
  - Automatic cloud backups
  - Cloud sync options

- [ ] **Session timeout warnings**
  - Warning before auto-lock
  - Extend session option
  - Configurable timeout duration

- [ ] **Two-factor authentication**
  - 2FA for master password
  - TOTP support
  - Backup codes

- [ ] **Password strength indicator**
  - Real-time password strength checking
  - Password requirements display
  - Suggestions for stronger passwords

---

## Advanced Features

- [ ] **File synchronization (rsync-like)**
  - Sync specific folders automatically
  - Two-way sync with conflict resolution
  - Sync scheduling and rules

- [ ] **Script execution**
  - Run shell scripts on remote servers
  - Script library
  - Script templates

- [ ] **Remote terminal**
  - Integrated terminal in SFTP view
  - Execute commands on remote server
  - Terminal history per session

- [ ] **Activity log**
  - View log of all operations
  - Filter log by type, date
  - Export log

- [ ] **Export reports**
  - Transfer reports
  - Sync reports
  - Usage statistics

---

## Developer Features

- [ ] **Plugin system**
  - Plugin architecture
  - Plugin marketplace
  - Plugin API documentation

- [ ] **Custom themes (for whole application)**
  - User-generated themes
  - Theme marketplace
  - Theme editor

---

## Platform & Distribution

- [ ] **Web-based version**
  - Browser-compatible version
  - Progressive Web App (PWA) support
  - Cloud-based deployment

- [ ] **Mobile app companion**
  - Mobile application for iOS/Android
  - Sync with desktop version
  - Mobile-optimized UI

---

## Accessibility

- [ ] **Screen reader support**
  - ARIA labels
  - Keyboard navigation improvements
  - Screen reader testing

- [ ] **Keyboard-only navigation**
  - Full keyboard navigation
  - Focus indicators
  - Tab order optimization

- [ ] **Font size adjustments**
  - Configurable font sizes
  - Zoom levels
  - Per-component font scaling

---
