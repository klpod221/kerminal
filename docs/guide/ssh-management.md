# SSH Management

Manage SSH connections with Kerminal's powerful profile system.

## Creating Profiles

1. Open SSH Profiles panel (`Ctrl+Shift+S`)
2. Click **New Profile**
3. Fill in connection details:

| Field | Description |
|-------|-------------|
| **Name** | Profile display name |
| **Host** | Hostname or IP |
| **Port** | SSH port (default: 22) |
| **Username** | SSH username |
| **Auth** | Password or SSH Key |

## Organizing Profiles

### Groups
1. Click **New Group**
2. Drag profiles into groups
3. Collapse/expand as needed

### Colors
1. Edit profile → **Terminal** tab
2. Select **Tab Color**
3. Color shows in sidebar and tabs

## Authentication

### Password
Enter password when creating profile. Encrypted with AES-256-GCM.

::: tip
Consider SSH keys for better security.
:::

### SSH Keys
1. Select **SSH Key** as auth method
2. Choose existing key or click **Manage SSH Keys**

### Key Manager
- **Generate**: RSA, Ed25519, ECDSA
- **Import**: From file or clipboard
- **Export**: Public key for servers

## Network Options

### Proxy
1. Edit profile → **Network** tab
2. Enable **Use Proxy**
3. Select type: HTTP, SOCKS4, SOCKS5
4. Enter proxy details

### Jump Hosts

Connect through bastion servers:

```
Local → Bastion → Target
```

1. Create bastion profile first
2. In target profile → **Network** tab
3. Enable **Use Jump Host**
4. Select bastion profile

**Chained jumps**: Add multiple jump hosts in order.

## Port Forwarding

### Local Forward
Access remote services locally:
```
localhost:8080 → remote:80
```
Use case: Access web UIs, databases

### Remote Forward
Expose local services remotely:
```
remote:8080 → localhost:3000
```
Use case: Share dev server, webhook testing

### Dynamic (SOCKS)
Create SOCKS proxy through SSH:
```
Local port 1080 → SOCKS5 proxy
```
Use case: Browse through remote network

Enable **Auto-start** to establish tunnels on connect.

## Import & Export

### From ~/.ssh/config
1. Look for **From .ssh/config** section
2. Click **Import** button
3. Select hosts to import

### Backup & Restore
1. Click **Backup & Restore** in top bar
2. **Export**: Save profiles as JSON (password protected optional)
3. **Import**: Restore from backup file

## Testing Connections

Before saving:
1. Fill in all details
2. Click **Test Connection**
3. View results: success, auth error, or network error

## Best Practices

### Security
- Use SSH keys over passwords
- Protect keys with passphrases
- Rotate keys periodically

### Organization
- Group by environment (prod, staging, dev)
- Use descriptive names
- Color-code for quick identification

