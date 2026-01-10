# Session Recording

Record and replay terminal sessions for documentation, training, or debugging.

## Recording Sessions

### Start Recording

1. Click the **Record** button (●) in the terminal toolbar
2. A red indicator confirms recording is active
3. All terminal input and output is captured

### Stop Recording

1. Click the **Stop** button (■)
2. Enter a name for the recording
3. Recording is saved locally

::: tip
Give recordings descriptive names like "nginx-setup-2024" for easy identification.
:::

## Playing Recordings

### Playback Controls

1. Open the **Recordings** panel from the drawer
2. Click a recording to play

| Control | Action |
|---------|--------|
| **Play/Pause** | Start or pause playback |
| **Timeline** | Click to seek to any point |
| **Speed** | Adjust from 0.5x to 4x |
| **Restart** | Jump to beginning |

### Keyboard Shortcuts (during playback)

| Key | Action |
|-----|--------|
| `Space` | Play/Pause |
| `←` / `→` | Seek backward/forward |
| `Home` | Jump to start |
| `End` | Jump to end |

## Managing Recordings

### Organize

- **Rename**: Right-click → Rename
- **Delete**: Right-click → Delete
- **Search**: Use the search bar to filter

### Export

Recordings use **asciicast v2** format:

1. Right-click a recording → Export
2. Save the `.cast` file

Compatible with:
- [asciinema.org](https://asciinema.org) for sharing
- `asciinema play` command-line tool
- Any asciicast-compatible player

## Use Cases

### Documentation
Record setup procedures to share with your team.

### Training
Create tutorials showing command sequences and expected output.

### Debugging
Capture problematic sessions for later analysis.

### Demonstrations
Record demos for presentations or READMEs.

## Tips

1. **Plan your recording** - Know what you want to demonstrate
2. **Keep it short** - Focus on specific tasks
3. **Use clear commands** - Avoid typos and corrections
4. **Add descriptions** - Name recordings descriptively
5. **Export for sharing** - asciicast format works everywhere

