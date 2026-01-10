# SFTP File Browser

Transfer files securely with Kerminal's built-in SFTP file browser.

## Opening SFTP

1. Connect to an SSH profile
2. Press `Ctrl+Shift+F` or click the folder icon in the toolbar

::: tip
SFTP is only available for active SSH connections.
:::

## Interface Overview

The file browser provides a familiar interface:
- **Path bar** - Current directory with editable path
- **File list** - Files and folders with details
- **Toolbar** - Navigation, upload, and view options

## File Operations

| Action | How |
|--------|-----|
| Navigate | Click folder or edit path bar |
| Go back | `←` button or `Backspace` |
| Go up | Click parent folder in path |
| Refresh | Click refresh button |
| Upload | Drag & drop or click Upload |
| Download | Right-click → Download |
| New folder | Right-click → New Folder |
| Rename | Right-click → Rename |
| Delete | Right-click → Delete |

## Path Bar

- **Edit**: Click the path bar to type manually
- **Navigate**: Type path and press `Enter`
- **Autocomplete**: Use `Tab` for suggestions
- **Cancel**: Press `Escape`

## Keyboard Navigation

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate files |
| `Enter` | Open folder / Preview file |
| `Backspace` | Go to parent folder |
| `Ctrl+F` | Search in directory |
| `Delete` | Delete selected |
| `F2` | Rename selected |
| `Ctrl+Shift+N` | New folder |

## Drag and Drop

### Upload
- Drag files from your computer into the browser
- Progress indicator shows upload status
- Multiple files supported

### Download
- Drag files from browser to your desktop
- Or right-click → Download

## File Preview

Double-click files to preview:
- **Text files** - Syntax-highlighted view
- **Images** - Preview with zoom controls
- **Others** - Download prompt

## Transfer Queue

When transferring multiple files:
- View progress in the transfer queue
- Cancel individual transfers
- Resume interrupted transfers

## Best Practices

1. **Use drag and drop** for quick uploads
2. **Keyboard navigation** for power users
3. **Preview before download** for text files
4. **Create folders** to organize remote files

