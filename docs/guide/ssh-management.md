# SSH Management

Learn how to manage SSH connections effectively with Kerminal's powerful profile management system.

## SSH Profiles

### Creating a Profile

1. Open the SSH Profiles panel (sidebar)
2. Click the **New Profile** button at the bottom of the panel
3. In the **Basic** tab, fill in the connection details:
   - **Profile Name**: A descriptive name
   - **Host**: Hostname or IP address
   - **Port**: SSH port (default: 22)
   - **Username**: SSH username
   - **Authentication Method**: Choose Password or SSH Key

### Profile Groups

Organize your connections by creating groups:

1. Click the **New Group** button at the bottom of the SSH Profiles panel
2. Enter a group name
3. Drag profiles into the group or use the **Add profile to group** button on the group header

### Profile Colors

Assign colors to profiles for visual organization:

1. In the profile editor, go to the **Terminal** tab
2. Use the **Tab Color** picker to select a color
3. Colors appear as indicators in the sidebar and tab headers

## Authentication Methods

### Password Authentication

Simply enter your password when creating the profile. The password is encrypted using AES-256-GCM.

::: tip Security Note
Consider using key-based authentication for better security.
:::

### SSH Key Authentication

1. In the profile editor (**Basic** tab), select **SSH Key** as the Authentication Method
2. Select an existing key from the dropdown
3. Or click **Manage SSH Keys** to import/generate keys

### SSH Key Manager

Access the key manager via the **Manage SSH Keys** link in the profile editor:

#### Generate New Keys
```bash
# Kerminal uses russh for key generation
# Supports RSA, Ed25519, and ECDSA
```

#### Import Keys
- Import from file
- Paste key content directly
- Import from clipboard

#### Export Keys
- Export public key for server configuration
- Backup private keys (encrypted)

## Proxy Configuration

Connect through proxies when direct SSH is not available.

1. In the profile editor, go to the **Network** tab
2. Check **Use Proxy**
3. Select **Proxy Type** (HTTP, SOCKS4, SOCKS5)

### HTTP Proxy

```
Host: proxy.example.com
Port: 8080
Username: (optional)
Password: (optional)
```

### SOCKS4/5 Proxy

```
Host: socks.example.com
Port: 1080
Username: (optional)
Password: (optional)
```

## Jump Hosts (Bastion)

Connect through one or more bastion hosts to reach your target server.

### Simple Jump

```
Local → Bastion → Target
```

1. Create a profile for the bastion host first
2. In the target profile's **Network** tab:
3. Check **Use Jump Host**
4. Select the bastion profile from the **Jump Host Profile** dropdown

### Chained Jumps

```
Local → Bastion1 → Bastion2 → Target
```

Add multiple jump hosts in order. Kerminal will automatically:
- Authenticate at each hop
- Forward connections through the chain
- Show the connection path visually

## Port Forwarding

### Local Port Forwarding

Access remote services on your local machine:

```
Local Port: 8080
Remote Host: localhost
Remote Port: 80
```

Now `localhost:8080` connects to port 80 on the remote server.

**Use cases:**
- Access web interfaces behind firewalls
- Connect to remote databases
- Use remote development servers

### Remote Port Forwarding

Expose local services to the remote network:

```
Remote Port: 8080
Local Host: localhost
Local Port: 3000
```

Now port 8080 on the remote server connects to your local port 3000.

**Use cases:**
- Share local development server
- Webhook testing
- Temporary service exposure

### Dynamic Port Forwarding (SOCKS)

Create a SOCKS proxy through SSH:

```
Local Port: 1080
```

Configure applications to use `localhost:1080` as SOCKS5 proxy.

**Use cases:**
- Browse the web through the remote network
- Access internal resources
- Bypass network restrictions

### Auto-Start Forwarding

Enable "Auto-start" on port forwarding rules to automatically establish tunnels when connecting.

## Connection Testing

Before saving a profile, test the connection:

1. Fill in all connection details
2. Click **Test Connection**
3. Kerminal will attempt to connect and report:
   - Success with server fingerprint
   - Authentication failure details
   - Network errors

## Import from SSH Config

Kerminal automatically parses your local SSH config file (`~/.ssh/config`) and allows you to import hosts as profiles.

1. Open the **SSH Profiles** panel
2. Look for the **From .ssh/config** section
3. Click the **Import** button (Download icon)
4. Select the hosts you want to import
5. Click **Import** to create profiles for selected hosts

## Backup & Restore

Protect your data by creating full backups of your profiles, keys, and settings.

### Create Backup

1. Click the **Backup & Restore** button (Archive icon) in the top bar
2. In the **Export Backup** section:
3. (Optional) Check **Password Protection** to encrypt your backup with AES-256-GCM
4. Click **Export Backup**
5. Save the `.json` (or `.kbak` if encrypted) file

### Restore Backup

1. Click the **Backup & Restore** button (Archive icon) in the top bar
2. In the **Import Backup** section:
3. Click **Select Backup File**
4. Choose your backup file
5. If encrypted, enter the password when prompted
6. The application will reload with restored data

::: warning
Restoring a backup will update existing data matching the IDs in the backup.
:::

## Best Practices

### Security

1. **Use key-based authentication** when possible
2. **Protect your keys** with passphrases
3. **Rotate keys** periodically
4. **Don't share profiles** containing credentials

### Organization

1. **Group by environment** (prod, staging, dev)
2. **Use descriptive names** that include purpose
3. **Color-code** for quick identification
4. **Add notes** for connection-specific information

### Backup

1. **Create backups** regularly
2. **Encrypt backups** with a strong password
3. **Store backups** securely (encrypted cloud, password manager)
