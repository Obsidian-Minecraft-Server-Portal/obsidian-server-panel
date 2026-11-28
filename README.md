# Obsidian

A self-hostable **Minecraft server panel**. Obsidian is a single native binary
that serves a web UI and an open REST API for creating, running and managing
Minecraft servers.

- Vanilla, Fabric, Quilt, Forge, NeoForge, Paper, Purpur, Spigot, CraftBukkit
  and custom jars.
- Mods, plugins and modpacks from Modrinth, CurseForge, Hangar and Spiget.
- A live console, a file manager with a code editor, and a built-in SFTP server.
- An optional world map for each instance, with a top-down view and a free-fly
  3D view. Live player positions come from the console or, if you opt in, RCON.
- Web maps from BlueMap, Dynmap, Squaremap and Pl3xmap, shown in the Map tab.
- Deduplicated, incremental backups with single-file restore.
- A plain-English scheduler for backups, restarts and commands.
- Instance sharing through preset roles with granular permissions, including
  separate permissions for the map and for player positions.
- TOTP two-factor login, new-IP verification and invite-only accounts.
- Java is downloaded automatically for each Minecraft version, so you don't
  need to install it yourself.

Minecraft servers run as detached processes. Restarting or updating the panel
does not stop them.

## Contents

- [Quick start](#quick-start)
- [Docker](#docker)
- [Windows](#windows)
- [macOS](#macos)
- [Linux](#linux)
- [Configuration](#configuration)
- [Ports](#ports)
- [Data directory](#data-directory)
- [Running behind a reverse proxy](#running-behind-a-reverse-proxy)
- [Command line](#command-line)
- [Updates](#updates)
- [API](#api)
- [Building from source](#building-from-source)

## Quick start

1. Install Obsidian using one of the methods below.
2. Open `http://<host>:1421` in a browser.
3. Complete the setup wizard. It creates the admin account and asks for the
   optional SMTP settings, the timezone and the default UPnP behaviour.

After setup, new accounts come only from one-time invite links or from an admin.
There is no open signup.

## Docker

The image is published for `linux/amd64` and `linux/arm64`:

```
ghcr.io/obsidian-minecraft-server-portal/obsidian-server-panel
```

Tags are `latest`, the full version (for example `0.1.1`) and the minor
version (for example `0.1`).

Minecraft servers run as plain processes inside the same container. Java
runtimes are downloaded into the data volume the first time they are needed.

### Docker Compose

There are two ways to set up networking:

| Option                                   | Use it when                                                                 |
|------------------------------------------|-----------------------------------------------------------------------------|
| [Published ports](#option-1-published-ports) | You want the container isolated and know which ports to expose.         |
| [Host network](#option-2-host-network)   | You want UPnP, or don't want to manage port ranges. Linux hosts only.       |

#### Option 1: published ports

```yaml
services:
  obsidian:
    image: ghcr.io/obsidian-minecraft-server-portal/obsidian-server-panel:latest
    container_name: obsidian
    restart: unless-stopped
    ports:
      - "1421:1421"               # web panel and API
      - "2022:2022"               # SFTP
      - "25565-25575:25565-25575" # Minecraft servers, one port per instance
    volumes:
      - obsidian-data:/data
    environment:
      OBSIDIAN_TIMEZONE: "America/New_York"
      # OBSIDIAN_PUBLIC_URL: "https://panel.example.com"

      # Optional: create the first admin without the setup wizard.
      # OBSIDIAN_ADMIN_USERNAME: "admin"
      # OBSIDIAN_ADMIN_EMAIL: "admin@example.com"
      # OBSIDIAN_ADMIN_PASSWORD: "change-me"

      # Optional: SMTP enables password resets and emailed login codes.
      # OBSIDIAN_SMTP_HOST: "smtp.example.com"
      # OBSIDIAN_SMTP_PORT: "587"
      # OBSIDIAN_SMTP_USERNAME: "panel@example.com"
      # OBSIDIAN_SMTP_PASSWORD: "secret"
      # OBSIDIAN_SMTP_FROM: "Obsidian <panel@example.com>"
      # OBSIDIAN_SMTP_TLS: "starttls"
    stop_grace_period: 90s

volumes:
  obsidian-data:
```

#### Option 2: host network

The container shares the host's network stack, so there are no port mappings.
The panel listens on the host's ports `1421` and `2022`, every Minecraft server
is reachable on whatever port it's given, and UPnP port forwarding works.

```yaml
services:
  obsidian:
    image: ghcr.io/obsidian-minecraft-server-portal/obsidian-server-panel:latest
    container_name: obsidian
    restart: unless-stopped
    network_mode: host
    volumes:
      - obsidian-data:/data
    environment:
      OBSIDIAN_TIMEZONE: "America/New_York"
      # OBSIDIAN_PORT: "1421"
      # OBSIDIAN_SFTP_PORT: "2022"
      # OBSIDIAN_BIND: "0.0.0.0"
      # OBSIDIAN_UPNP_DEFAULT: "true"
      # OBSIDIAN_PUBLIC_URL: "https://panel.example.com"

      # Optional: create the first admin without the setup wizard.
      # OBSIDIAN_ADMIN_USERNAME: "admin"
      # OBSIDIAN_ADMIN_EMAIL: "admin@example.com"
      # OBSIDIAN_ADMIN_PASSWORD: "change-me"

      # Optional: SMTP enables password resets and emailed login codes.
      # OBSIDIAN_SMTP_HOST: "smtp.example.com"
      # OBSIDIAN_SMTP_PORT: "587"
      # OBSIDIAN_SMTP_USERNAME: "panel@example.com"
      # OBSIDIAN_SMTP_PASSWORD: "secret"
      # OBSIDIAN_SMTP_FROM: "Obsidian <panel@example.com>"
      # OBSIDIAN_SMTP_TLS: "starttls"
    stop_grace_period: 90s

volumes:
  obsidian-data:
```

With host networking, change the panel or SFTP port through `OBSIDIAN_PORT`
and `OBSIDIAN_SFTP_PORT` instead of a port mapping, and make sure nothing else
on the host already uses them. Host networking only behaves this way on Linux;
Docker Desktop on Windows and macOS runs containers inside a VM, so use
option 1 there.

Start the stack:

```bash
docker compose up -d
```

When the container stops, the panel sends `stop` to every running server and
waits for them to save. `stop_grace_period` gives it time to do that before
Docker kills the container. Keep it above the panel's stop timeout, which is 30
seconds by default.

### docker run

```bash
docker run -d \
  --name obsidian \
  --restart unless-stopped \
  --stop-timeout 90 \
  -p 1421:1421 \
  -p 2022:2022 \
  -p 25565-25575:25565-25575 \
  -v obsidian-data:/data \
  -e OBSIDIAN_TIMEZONE=America/New_York \
  ghcr.io/obsidian-minecraft-server-portal/obsidian-server-panel:latest
```

The host-network equivalent:

```bash
docker run -d \
  --name obsidian \
  --restart unless-stopped \
  --stop-timeout 90 \
  --network host \
  -v obsidian-data:/data \
  -e OBSIDIAN_TIMEZONE=America/New_York \
  ghcr.io/obsidian-minecraft-server-portal/obsidian-server-panel:latest
```

### Using a host folder instead of a named volume

The container runs as the unprivileged user `obsidian` with UID and GID
`10001`. A bind-mounted folder must be writable by that user:

```bash
mkdir -p ./obsidian-data
```

```bash
sudo chown -R 10001:10001 ./obsidian-data
```

Then mount it with `-v ./obsidian-data:/data`, or in Compose with
`- ./obsidian-data:/data`.

### Docker notes

- **Minecraft ports:** each new instance takes the next free port from 25565
  upwards. With published ports, publish a range that covers as many instances
  as you plan to run. You can change an instance's port in its settings.
- **UPnP:** UPnP discovery needs the host's network, so the per-instance UPnP
  toggle only works with the [host network](#option-2-host-network) option.
- **Updates:** Docker builds never update themselves. Pull a new image and
  recreate the container. Because the servers live in the same container,
  recreating it stops them cleanly; the panel restarts instances marked
  *auto start* when it boots.

## Windows

Download from the
[latest release](https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/releases/latest):

| File                                         | What it is                     |
|----------------------------------------------|--------------------------------|
| `obsidian-<version>-x86_64-pc-windows-msvc.msi` | Installer (recommended)     |
| `obsidian-x86_64-pc-windows-msvc.zip`        | Portable `obsidian.exe`        |

### Installer

Run the `.msi`. It installs Obsidian to `C:\Program Files\Obsidian`, adds it to
`PATH`, and creates a Start Menu shortcut.

The installer has an optional feature, **Run as a system service**. Pick the
one that fits:

- **Tray app (default):** launch *Obsidian* from the Start Menu. A tray icon
  appears; click it to open the panel. Data lives in
  `%APPDATA%\Obsidian`.
- **System service:** enable the feature during install. The panel starts on
  boot without anyone logged in, and data lives in `C:\ProgramData\Obsidian`.
  Don't also start the tray app from the Start Menu.

### Portable

Unzip `obsidian.exe` anywhere and run it. To install it as a service later,
run this from an elevated terminal:

```powershell
obsidian service install
```

```powershell
obsidian service start
```

Allow ports `1421`, `2022` and your Minecraft ports through Windows Firewall if
other machines need to reach them.

## macOS

Download from the
[latest release](https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/releases/latest).
Pick `aarch64` for Apple silicon or `x86_64` for Intel Macs.

| File                                     | What it is                       |
|------------------------------------------|----------------------------------|
| `obsidian-<version>-<arch>-apple-darwin.dmg` | `Obsidian.app` with a menu bar icon |
| `obsidian-<arch>-apple-darwin.tar.gz`    | The bare `obsidian` binary       |

### App

Open the `.dmg` and drag **Obsidian** into **Applications**. Launch it, then
click the menu bar icon to open the panel. Data lives in
`~/Library/Application Support/Obsidian`.

If macOS refuses to open the app because it can't verify the developer,
right-click the app and choose **Open**, or clear the quarantine flag:

```bash
xattr -dr com.apple.quarantine /Applications/Obsidian.app
```

### Install script

The script installs the binary to `/usr/local/bin`:

```bash
curl -fsSL https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/releases/latest/download/install.sh | sh
```

To run the panel as a launchd service that starts on boot:

```bash
sudo obsidian service install
```

```bash
sudo obsidian service start
```

The service keeps its data in `/Library/Application Support/Obsidian`.

## Linux

Release binaries are built for `x86_64` and `aarch64` (glibc).

### Install script

```bash
curl -fsSL https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/releases/latest/download/install.sh | sh
```

The script detects your architecture, verifies the download against the
published `SHA256SUMS`, and installs `obsidian` to `/usr/local/bin`. Set these
variables to change what it does:

| Variable             | Default             | Effect                                                                 |
|----------------------|---------------------|------------------------------------------------------------------------|
| `OBSIDIAN_VERSION`   | `latest`            | Release tag to install, for example `v0.1.1`.                          |
| `OBSIDIAN_PREFIX`    | `/usr/local/bin`    | Where the `obsidian` command goes.                                     |
| `OBSIDIAN_SERVICE`   | `0`                 | `1` installs a systemd service running as the `obsidian` user from `/opt/obsidian`. |
| `OBSIDIAN_DATA_DIR`  | `/var/lib/obsidian` | Data directory for the service.                                        |
| `OBSIDIAN_DESKTOP`   | `0`                 | `1` installs the build with a tray icon. It needs GTK 3 and an AppIndicator library. |

Pass variables to `sh`, not to `curl`.

### Recommended: systemd service

```bash
curl -fsSL https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/releases/latest/download/install.sh | OBSIDIAN_SERVICE=1 sh
```

This creates the `obsidian` system user, installs the binary to
`/opt/obsidian`, writes `/etc/systemd/system/obsidian.service` and starts it.
Manage it with `systemctl`:

```bash
sudo systemctl status obsidian
```

```bash
journalctl -u obsidian -f
```

Set environment variables in a drop-in:

```bash
sudo systemctl edit obsidian
```

```ini
[Service]
Environment=OBSIDIAN_PORT=8080
Environment=OBSIDIAN_PUBLIC_URL=https://panel.example.com
```

The unit uses `KillMode=process`, so restarting or updating the panel leaves
running Minecraft servers alone.

### Manual install

Download `obsidian-<arch>-unknown-linux-gnu.tar.gz` (headless) or
`obsidian-desktop-<arch>-unknown-linux-gnu.tar.gz` (tray icon) from the
[latest release](https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/releases/latest),
then:

```bash
tar -xzf obsidian-x86_64-unknown-linux-gnu.tar.gz
```

```bash
sudo install -m 0755 obsidian /usr/local/bin/obsidian
```

```bash
obsidian serve --headless
```

The headless build keeps its data in `./data` under the current directory
unless `OBSIDIAN_DATA_DIR` is set. The desktop build uses
`~/.local/share/Obsidian`. You can also register either build as a service with
`sudo obsidian service install`.

Open the panel and SFTP ports in your firewall if needed, for example:

```bash
sudo ufw allow 1421/tcp && sudo ufw allow 2022/tcp && sudo ufw allow 25565:25575/tcp
```

## Configuration

Settings are read in this order, and later sources win:

1. Built-in defaults.
2. `config.toml` in the [data directory](#data-directory).
3. `OBSIDIAN_*` environment variables.

Everything the setup wizard asks for can also be set through environment
variables, so a server can be provisioned without opening the UI.

### Environment variables

| Variable                     | Default    | Description |
|------------------------------|------------|-------------|
| `OBSIDIAN_DATA_DIR`          | see [Data directory](#data-directory) | Where the database, instances, backups and Java runtimes are stored. Only settable through the environment. |
| `OBSIDIAN_BIND`              | `0.0.0.0`  | Address the panel and SFTP server listen on. |
| `OBSIDIAN_PORT`              | `1421`     | Web panel and API port. |
| `OBSIDIAN_SFTP_PORT`         | `2022`     | SFTP port. `0` disables SFTP. |
| `OBSIDIAN_PUBLIC_URL`        | *unset*    | The URL users reach the panel at, for example `https://panel.example.com`. Used for email links, CSRF origin checks and the tray icon. When set, it can't be changed in the UI. |
| `OBSIDIAN_TIMEZONE`          | `UTC`      | IANA timezone for schedules, for example `Europe/Berlin`. A timezone saved in the panel takes precedence. |
| `OBSIDIAN_TRUSTED_PROXIES`   | *empty*    | Comma-separated IPs or CIDRs of reverse proxies whose `X-Forwarded-*` headers are trusted, for example `127.0.0.1,10.0.0.0/8`. |
| `OBSIDIAN_UPNP_DEFAULT`      | *unset*    | `true` or `false`: whether new instances have UPnP port forwarding on. Overrides the panel setting on every boot. |
| `OBSIDIAN_STOP_TIMEOUT_SECS` | `30`       | Seconds to wait after sending `stop` before killing a server, from 5 to 600. Overrides the panel setting on every boot. |
| `OBSIDIAN_ADMIN_USERNAME`    | *unset*    | Creates the first admin on boot. Used only while no account exists. Requires `OBSIDIAN_ADMIN_PASSWORD`. |
| `OBSIDIAN_ADMIN_EMAIL`       | *unset*    | Email for the bootstrapped admin. |
| `OBSIDIAN_ADMIN_PASSWORD`    | *unset*    | Password for the bootstrapped admin. |
| `OBSIDIAN_SMTP_HOST`         | *unset*    | SMTP server. Setting it together with `OBSIDIAN_SMTP_FROM` enables email. |
| `OBSIDIAN_SMTP_PORT`         | `587`      | SMTP port. |
| `OBSIDIAN_SMTP_USERNAME`     | *unset*    | SMTP login. |
| `OBSIDIAN_SMTP_PASSWORD`     | *unset*    | SMTP password. |
| `OBSIDIAN_SMTP_FROM`         | *unset*    | Sender address, for example `Obsidian <panel@example.com>`. |
| `OBSIDIAN_SMTP_TLS`          | `starttls` | `none`, `starttls` or `tls`. |

SMTP is optional. With it, users can reset forgotten passwords and receive
emailed codes when they sign in from a new IP. Without it, a login from a new
IP is allowed and the user and admins get an in-panel security notification,
unless the user has TOTP enabled.

### config.toml

The same settings, in `config.toml` inside the data directory:

```toml
bind = "0.0.0.0"
port = 1421
sftp_port = 2022
public_url = "https://panel.example.com"
timezone = "Europe/Berlin"
trusted_proxies = ["127.0.0.1", "10.0.0.0/8"]
upnp_default = false
stop_timeout_secs = 30

[smtp]
host = "smtp.example.com"
port = 587
username = "panel@example.com"
password = "secret"
from = "Obsidian <panel@example.com>"
tls = "starttls"

[admin]
username = "admin"
email = "admin@example.com"
password = "change-me"
```

## Ports

| Port            | Protocol | Purpose                                   | Setting              |
|-----------------|----------|-------------------------------------------|----------------------|
| `1421`          | TCP      | Web panel, REST API and WebSocket         | `OBSIDIAN_PORT`      |
| `2022`          | TCP      | SFTP                                      | `OBSIDIAN_SFTP_PORT` |
| `25565` and up  | TCP      | Minecraft servers, one per instance       | per instance         |
| `25575` and up  | TCP      | RCON for player positions (opt-in)        | Map tab              |

RCON listens on every network interface, so only open its port to networks you
trust. Some mods and plugins open extra ports, for example voice chat. Forward
those as well. The Map tab reaches a web map through the panel, so that map's
port needs forwarding only if players should open it directly.

## Data directory

| How Obsidian runs                          | Default data directory                     |
|--------------------------------------------|--------------------------------------------|
| Docker                                     | `/data`                                    |
| Windows tray app                           | `%APPDATA%\Obsidian`                       |
| Windows service                            | `C:\ProgramData\Obsidian`                  |
| macOS app                                  | `~/Library/Application Support/Obsidian`   |
| macOS service                              | `/Library/Application Support/Obsidian`    |
| Linux desktop build                        | `~/.local/share/Obsidian`                  |
| Linux service (install script or `service install`) | `/var/lib/obsidian`               |
| Linux headless build                       | `./data` in the working directory          |

`OBSIDIAN_DATA_DIR` overrides all of them. Inside it:

| Path            | Contents                                                       |
|-----------------|----------------------------------------------------------------|
| `obsidian.db`   | The database                                                   |
| `secret.key`    | Key that encrypts TOTP secrets. Back it up with the database.  |
| `sftp_host_key` | SFTP host key                                                  |
| `config.toml`   | Optional configuration                                         |
| `instances/`    | One folder per Minecraft server                                |
| `backups/`      | Backup chunk store and snapshots                               |
| `java/`         | Downloaded Java runtimes                                       |
| `cache/`        | Cached server jars and downloads                               |
| `runners/`      | Runner pidfiles and IPC endpoints                              |
| `logs/`         | Panel and runner logs                                          |

The database lives in memory and is written to `obsidian.db` every 30 seconds
when something changed, and on a clean shutdown. A hard crash or power loss
can lose up to 30 seconds of panel changes. Minecraft worlds and backups are
written straight to disk and are not affected.

## Running behind a reverse proxy

Put Obsidian behind a proxy such as Caddy or nginx to serve it over HTTPS.

1. Set `OBSIDIAN_PUBLIC_URL` to the public HTTPS address.
2. Set `OBSIDIAN_TRUSTED_PROXIES` to the proxy's address, so client IPs and the
   scheme come from `X-Forwarded-*` headers.
3. Proxy WebSocket upgrades for `/api/v1/ws`.

Caddy:

```
panel.example.com {
    reverse_proxy 127.0.0.1:1421
}
```

nginx:

```nginx
server {
    listen 443 ssl;
    server_name panel.example.com;

    client_max_body_size 0;

    location / {
        proxy_pass http://127.0.0.1:1421;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

SFTP and Minecraft traffic do not go through the HTTP proxy. Expose those ports
directly.

## Command line

```
obsidian [COMMAND]
```

| Command                          | Description                                                              |
|----------------------------------|--------------------------------------------------------------------------|
| `serve`                          | Run the panel. This is the default. Shows a tray icon on desktop builds. |
| `serve --headless`               | Run the panel without a tray icon.                                       |
| `service install`                | Register the panel as an OS service that starts on boot.                 |
| `service start` / `service stop` | Start or stop the installed service.                                     |
| `service uninstall`              | Remove the service.                                                      |
| `reset-password <username>`      | Replace a user's password with a printed temporary one.                  |
| `--version`                      | Print the version.                                                       |

`service` commands need administrator or root rights. Services use
systemd on Linux, launchd on macOS and the Service Control Manager on Windows.
They are not available in the Docker image.

### Locked out?

Stop the panel, run `reset-password`, then start it again and sign in with the
printed password. Run it with the same `OBSIDIAN_DATA_DIR` as the panel.

Linux service:

```bash
sudo systemctl stop obsidian
```

```bash
sudo -u obsidian env OBSIDIAN_DATA_DIR=/var/lib/obsidian /opt/obsidian/obsidian reset-password admin
```

```bash
sudo systemctl start obsidian
```

Docker Compose:

```bash
docker compose stop obsidian
```

```bash
docker compose run --rm obsidian reset-password admin
```

```bash
docker compose start obsidian
```

## Updates

Desktop and standalone builds check GitHub Releases every six hours and can
update in place from the panel. Running Minecraft servers keep running during
the update.

Docker images never update themselves. Pull the new image instead:

```bash
docker compose pull && docker compose up -d
```

## API

- REST API under `/api/v1`.
- Interactive docs at `/api/docs`, and the OpenAPI 3.1 spec at
  `/api/docs/openapi.json`.
- Health check at `/api/v1/health`.
- One multiplexed WebSocket at `/api/v1/ws` for console output, status,
  stats, tasks and notifications.

Create personal API tokens under your account settings. A token can do only
what its creator can do.

## Building from source

Requirements:

- Rust (stable)
- Node.js 22 and pnpm
- [`just`](https://github.com/casey/just)
- On Linux, for the tray build: `libgtk-3-dev`, `libxdo-dev` and
  `libayatana-appindicator3-dev`

| Recipe                | What it does                                                       |
|-----------------------|--------------------------------------------------------------------|
| `just dev`            | Debug run with Vite hot reload, data in `target/dev-env`           |
| `just dev-tray`       | The same, with the tray icon                                       |
| `just build`          | Release desktop binary (tray, service, updater)                    |
| `just build-headless` | Release binary with the Docker feature set (no tray, no updater)   |
| `just docker-build`   | Build the Docker image as `obsidian-server-panel:dev`              |
| `just docker-run`     | Build and run the image on ports 1421 and 2022                     |
| `just ci`             | Clippy, tests, type check, frontend build and the line limit       |

Builds go to `target/release/obsidian`. The Vite dev server for `just dev`
needs port 3000 free.

CurseForge features need an API key at **build time**:

```bash
CURSEFORGE_API_KEY=your-key just build
```

Without a key, CurseForge is disabled and every other source still works.

Cargo features:

| Feature   | Includes                                            |
|-----------|-----------------------------------------------------|
| `desktop` | Default. `tray`, `service` and `updater`.           |
| `tray`    | System tray icon that opens the panel.              |
| `service` | `obsidian service …` commands.                      |
| `updater` | In-place updates from GitHub Releases.              |
| `docker`  | Used by the Docker image. Build with `--no-default-features --features docker`. |

---

Obsidian is not affiliated with Mojang or Microsoft.
