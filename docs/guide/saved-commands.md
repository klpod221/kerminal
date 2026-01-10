# Saved Commands

Build a library of frequently used commands for quick access and reuse.

## Creating Commands

### Add a New Command

1. Open Saved Commands with `Ctrl+Alt+C`
2. Click **New Command**
3. Fill in the details:

| Field | Description |
|-------|-------------|
| **Name** | Descriptive name (e.g., "Deploy to Production") |
| **Command** | The actual command to run |
| **Group** | Category for organization |
| **Description** | Optional notes |

### Quick Save

Right-click in the terminal and select **Save Command** to save the last executed command.

## Using Variables

Commands support dynamic variable substitution using `{{variable}}` syntax:

```bash
ssh {{user}}@{{host}} -p {{port}}
```

When executed, you'll be prompted to fill in each variable.

### Common Variable Patterns

```bash
# SSH connection
ssh {{user}}@{{server}}

# Docker commands
docker exec -it {{container}} bash

# Git operations
git clone git@github.com:{{org}}/{{repo}}.git

# File operations
scp {{file}} {{user}}@{{host}}:{{path}}
```

## Executing Commands

| Action | Result |
|--------|--------|
| **Single click** | Insert into active terminal |
| **Double click** | Insert and execute |
| **Right-click** | Show context menu |

### Context Menu Options

- Execute in current terminal
- Execute in new tab
- Edit command
- Copy to clipboard
- Delete

## Organizing Commands

### Groups

- Create groups to categorize commands
- Drag commands between groups
- Collapse/expand groups

### Favorites

- Star commands for quick access
- Favorites appear at the top

### Search

- Use the search bar to filter commands
- Searches name, command, and description

## Usage Statistics

Kerminal tracks how often you use each command:
- View usage count on each command
- Sort by most used
- Identify candidates for favorites

## Import & Export

### Export Commands

1. Right-click a group → Export
2. Save as JSON file

### Import Commands

1. Click Import in the panel
2. Select JSON file
3. Choose merge or replace

## Tips

1. **Use groups** - Organize by project, environment, or purpose
2. **Variables for flexibility** - Make commands reusable
3. **Descriptive names** - Easy to find later
4. **Star favorites** - Quick access to common commands
5. **Keep it current** - Remove outdated commands

