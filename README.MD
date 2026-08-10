# Obsidian Server Panel

![Obsidian Server Panel](resources/Screenshot%202025-09-28%20at%2019-28-08%20Obsidian%20Server%20Panel.png)  
A modern, feature-rich, and user-friendly web-based control panel for managing Minecraft servers. Built with a powerful Rust backend and a responsive React frontend, Obsidian provides a seamless experience for server administrators.

## ✨ Features

-   **🖥️ Comprehensive Server Management:**
    -   Create, configure, and manage multiple Minecraft servers.
    -   Support for Vanilla, Fabric, Forge, NeoForge, Quilt, Spigot, CraftBukkit, and custom server jars.
    -   Easy installation of different Minecraft and loader versions.
    -   Control server lifecycle: Start, Stop, Restart, and Kill operations.

-   **📦 One-Click Modpack Installation:**
    -   Browse and install server packs from Modrinth, CurseForge, ATLauncher, and Technic.
    -   Automatic mod, dependency, and override handling from pack manifests.
    -   Discovery is filtered to server-compatible packs, so client-only packs are clearly flagged.

-   **🔒 Robust User & Permission System:**
    -   Multi-user support with a role-based permission system.
    -   Secure authentication with password and token-based sessions.
    -   Optional email verification and two-factor authentication, enabled automatically when SMTP is configured and fully bypassed when it is not.
    -   Admin panel for user management: create, edit, delete, and disable users.

-   **📊 Real-time Monitoring & Console:**
    -   Live server console with command input.
    -   On Linux, servers run inside named GNU `screen` sessions, so they're always reachable with `screen -ls` and survive a panel restart.
    -   Real-time host resource monitoring for CPU, memory, disk, and network usage.
    -   Live player count and list.

-   **🗂️ Integrated File Manager:**
    -   Full-featured web-based file browser for each server instance.
    -   Standard file operations: Upload, Download, Move, Copy, Rename, and Delete.
    -   Built-in tools for archiving (`.zip`) and extracting files directly on the server.
    -   In-browser text editor for easy configuration file changes with syntax highlighting.

-   **🧩 Effortless Mod & Content Management:**
    -   Browse and install mods directly from Modrinth & CurseForge.
    -   View installed mods and check for available updates.
    -   Detailed mod view with descriptions, changelogs, and version history.

-   **💾 Automated Backup System:**
    -   Create on-demand or scheduled server backups.
    -   Supports full, incremental, and world-only backup types.
    -   Flexible cron-based scheduling for automation.
    -   Configurable backup retention policies to manage disk space.

-   **☕ Java Version Management:**
    -   Install and manage multiple Java runtimes required for different Minecraft versions.
    -   Easily switch between Java versions for each server.

-   **♿ Accessibility Focused:**
    -   Multiple themes designed for accessibility, including modes for Deuteranopia, Tritanopia, and a high-contrast Monochrome theme.

## 🛠️ Technology Stack

-   **Backend:**
    -   **Language:** Rust (edition 2024)
    -   **Framework:** Actix Web
    -   **Database:** SQLite, MySQL, or PostgreSQL (via SQLx)
    -   **Real-time Communication:** Server-Sent Events (SSE)
-   **Frontend:**
    -   **Framework:** React 19 (with TypeScript)
    -   **Build Tool:** Vite
    -   **UI Components:** [HeroUI](https://heroui.com/) v3
    -   **Styling:** Tailwind CSS v4
    -   **Animations:** Framer Motion
    -   **Charting:** Recharts
    -   **Editor:** Monaco

## 🚀 Getting Started

### Installation (Linux)

The easiest way to install the Obsidian Server Panel is by using the provided installation script.

```bash
curl -sSL https://raw.githubusercontent.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/main/install-obsidian.sh | sudo bash
```

The script will automatically:
1.  Check for necessary dependencies.
2.  Download the latest release for your architecture.
3.  Set up the binary in `/usr/local/bin`.
4.  Create a configuration directory at `/etc/obsidian`.
5.  Set up and enable a `systemd` service to run the panel on boot.

You will be prompted to set the port for the web UI during installation.

> **Note:** For live server consoles on Linux, install GNU `screen` (4.06 or newer). The panel creates a `screen` session per server automatically.

### Installation (Windows)

Download the installer (`obsidian-server-windows-x86_64-setup.exe`) or the standalone zip from the [latest release](../../releases). The installer creates Start Menu shortcuts and an optional start-at-login entry.

### Docker

A Compose stack with MySQL is included:

```bash
docker compose -f composer.yml up --build
```

The panel will be available at `http://localhost:3698`. **Change the default passwords in `composer.yml` before deploying to production.**

Prebuilt images are published to the GitHub Container Registry:

```bash
docker pull ghcr.io/obsidian-minecraft-server-portal/server:latest
```

### Configuration

The panel listens on **port 80** by default; pass `--port <PORT>` to change it. Key environment variables:

| Variable | Description |
|----------|-------------|
| `DATABASE_CONNECTION_STRING` | Database connection string (defaults to a local SQLite file) |
| `SMTP_HOST` | Enables email verification and 2FA when set; both are disabled when unset |
| `RUST_LOG` | Log level filter |

See [CONTRIBUTING.md](CONTRIBUTING.md#environment-variables) for the complete list, including the full SMTP configuration.

### Building from Source

Build instructions and development setup live in **[CONTRIBUTING.md](CONTRIBUTING.md)**.

## 📄 API Documentation

This project includes an OpenAPI 3.0 specification for its backend API in [`openapi-v3.json`](openapi-v3.json). It can be used with tools like Swagger UI or Postman to explore and interact with the API endpoints.

## 🤝 Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for build instructions, code style, testing, and the pull request process.

## 📜 License

This project is provided under the **Obsidian Private License**.

-   **You are permitted to:**
    -   Use, copy, modify, and distribute the software for private, non-commercial purposes.
    -   Run the software on your own hardware for personal use.
    -   Share the software with friends and family for personal use.

-   **You are NOT permitted to:**
    -   Sell, rent, lease, or distribute this software for profit.
    -   Use this software in a commercial service or product.

Please see the [LICENSE](LICENSE) file for the full license text.

Copyright (c) 2025 Obsidian Minecraft Server Panel Project. All rights reserved.
