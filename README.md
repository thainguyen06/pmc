# Open Process Management (OPM)

## Overview

OPM (Open Process Management) is a simple PM2 alternative written in Rust. It provides a command-line/api/web interface to start, stop, restart, and manage fork processes

## Features

- Start, stop, restart, and reload processes.
- Watch for file changes and auto-reload processes.
- Set memory limits for processes.
- List all running processes with customizable output formats.
- Retrieve detailed information about a specific process.
- Get startup commands for processes.
- Use HTTP/rust api to control processes.
- **Web UI for visual process management** (NEW)
- **Integrated API server with daemon** (NEW)

## Web UI

OPM now includes a built-in web interface for managing processes through a browser. The web UI provides:

- Real-time process monitoring with auto-refresh
- Visual process list with status indicators
- Easy process creation with a user-friendly form
- Process control buttons (start, stop, restart, remove)
- Log viewer with filtering and follow mode
- Process details display (CPU, memory, uptime, etc.)

### Accessing the Web UI

Once the OPM daemon is running, the web UI is automatically available at:

```
http://localhost:8080/app
```

The daemon starts automatically with the integrated API server on port 8080.

### Screenshots

**Main Process List:**
![OPM Web UI - Main View](https://github.com/user-attachments/assets/e2af6365-6eeb-4c67-9c05-4d99812465c4)

**New Process Form:**
![OPM Web UI - New Process](https://github.com/user-attachments/assets/b4976ed8-d0b3-41ff-a80a-007884e9beee)

## Usage

```bash
# Start/Restart a process
opm start <id/name> or <script> [--name <name>] [--watch <path>] [--max-memory <limit>]

# Restart a process
opm restart <id/name>

# Reload a process (alias for restart)
opm reload <id/name>

# Stop/Kill a process
opm stop <id/name>

# Remove a process
opm remove <id/name>

# Get process info
opm info <id/name>

# Get process env
opm env <id/name>

# Get startup command for a process
opm cstart <id/name>

# Save all processes to dumpfile
opm save

# Restore all processes
opm restore

# List all processes
opm list [--format <raw|json|default>]

# Get process logs
opm logs <id/name> [--lines <num_lines>]

# Reset process index
opm daemon reset

# Stop daemon
opm daemon stop

# Start/Restart daemon
opm daemon start

# Check daemon health
opm daemon health

# Setup systemd service (autostart with system)
opm daemon setup
```

### Advanced Features

#### Configuration Import/Export
Export and import process configurations to HCL files, allowing you to save and restore multiple process configurations easily.

```bash
# Export a single process
opm export 0 process.hcl
opm export myapp myapp.hcl

# Export multiple processes by ID
opm export 1,4,7 multi_config.hcl

# Export multiple processes by name
opm export app1,app2,app3 apps.hcl

# Export all processes
opm export all all_processes.hcl

# Import processes from a configuration file
opm import config.hcl
```

The exported configuration includes:
- Process script/command
- Environment variables (only those different from system environment)
- Watch path (if enabled)
- Memory limits (if set)
- All metadata needed to recreate the process

#### Watch Mode
Automatically reload your process when files change:
```bash
opm start app.js --watch .
```

#### Memory Limits
Set a maximum memory limit for a process:
```bash
opm start app.js --max-memory 500M
opm start app.py --max-memory 1G
```

#### Get Startup Command
Get the exact command used to start a process:
```bash
opm cstart 0
# or
opm get-command myapp
```

For more command information, check out `opm --help`

### Remote Server Management

OPM supports managing processes on remote OPM instances. The web UI and API can connect to remote servers.

#### API Endpoints for Remote Management

All CLI commands support the `--server` flag to operate on remote instances:

```bash
# List processes on a remote server
opm list --server remote-name

# Start a process on a remote server
opm start "node app.js" --name "remote-app" --server remote-name

# View logs from a remote server
opm logs 0 --server remote-name
```

#### Configuring Remote Servers

Remote servers are configured in `~/.opm/servers.toml`:

```toml
[servers.production]
address = "http://production-server:8080"
token = "your-auth-token"  # Optional

[servers.staging]
address = "http://staging-server:8080"
```

#### API Access

The integrated API server (automatically started with the daemon) provides:

- **REST API** on `http://localhost:8080` for programmatic access
- **Web UI** on `http://localhost:8080/app` for browser-based management
- **Remote API** endpoints at `/remote/{server-name}/` for managing remote instances
- **OpenAPI Documentation** at `http://localhost:8080/docs/embed`

The API supports all process management operations:
- `/list` - List all processes
- `/process/{id}/info` - Get process details
- `/process/{id}/action` - Control processes (start, stop, restart, etc.)
- `/process/{id}/logs/{type}` - View process logs
- `/process/create` - Create new processes
- `/daemon/metrics` - Get daemon metrics and health

## Troubleshooting

### Process Fails to Start or Restart

If you encounter errors when starting or restarting processes:

1. **Check the logs**: OPM provides detailed error messages in the process logs
   ```bash
   opm logs <id/name>
   ```

2. **Common Issues**:
   - **"Command not found"**: The shell or interpreter (e.g., `node`, `python`) is not in your PATH
     - Solution: Use the full path to the interpreter or add it to your PATH
     - Check your configuration: `~/.opm/config.toml`
   
   - **"Permission denied"**: The script or shell doesn't have execute permissions
     - Solution: `chmod +x your-script.sh`
   
   - **"Failed to open log file"**: The log directory doesn't exist or lacks write permissions
     - Solution: Create the directory or check permissions: `~/.opm/logs`
   
   - **"Failed to set working directory"**: The process path doesn't exist
     - Solution: Verify the path exists before starting the process

3. **Node.js Module Not Found**: If you get "module not found" errors in command prompt but it works in bash:
   - The PATH environment may differ between shells
   - Solution: Use the full path to `node` in your config or ensure PATH is consistent
   - Check your shell's environment: `opm env <id>`

### Restore Command Issues

If `opm restore` doesn't work as expected:

1. **Check daemon status**: `opm daemon health`
2. **Review restore output**: OPM now provides detailed progress during restore
3. **Check individual process logs**: Some processes may fail while others succeed

### Daemon Not Restarting Processes

If the daemon doesn't restart crashed processes:

1. **Check crash limit**: By default, processes that crash too many times (10) are stopped
   - Edit `~/.opm/config.toml` to adjust the `restarts` limit under `[daemon]`
   
2. **Review daemon logs**: The daemon now logs detailed information about restart attempts
   
3. **Reset counters**: Use `opm daemon reset` to reset process IDs if needed

### Environment Variables

OPM automatically loads `.env` files from the process working directory. If environment variables aren't being set:

1. **Check `.env` file location**: Must be in the process working directory
2. **View current environment**: `opm env <id>`
3. **Clear and reload**: `opm restart <id> --reset-env`

### Getting Help

- View detailed process information: `opm info <id>`
- Check daemon health: `opm daemon health`
- View all processes: `opm list`
- Check logs with errors only: `opm logs <id> --errors-only`

### System Integration

OPM can be configured to automatically start with your system using systemd:

```bash
# Setup systemd service (run as root for system-wide, or as user for user service)
opm daemon setup

# After setup, enable and start the service:
# For system-wide (root):
sudo systemctl daemon-reload
sudo systemctl enable opm.service
sudo systemctl start opm.service

# For user service:
systemctl --user daemon-reload
systemctl --user enable opm.service
systemctl --user start opm.service
loginctl enable-linger $USER  # Enable starting at boot even when not logged in
```

This ensures that:
- The OPM daemon starts automatically when your system boots
- All processes configured to run are automatically restored after system restart
- Process restart counters are reset on restore, giving each process a fresh start

## Installation

Pre-built binaries for Linux, MacOS, and WSL can be found on the [releases](releases) page.

There is no windows support yet. Install from crates.io using `cargo install opm` (requires clang++)

#### Building

- Clone the project
- Open a terminal in the project folder
- Check if you have cargo (Rust's package manager) installed, just type in `cargo`
- If cargo is installed, run `cargo build --release`
- Put the executable into one of your PATH entries, usually `/bin/` or `/usr/bin/`
