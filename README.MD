# Obsidian Server Panel

![Obsidian Server Panel](resources/Screenshot%202025-09-28%20at%2019-28-08%20Obsidian%20Server%20Panel.png)  
A modern, feature-rich, and user-friendly web-based control panel for managing Minecraft servers. Built with a powerful Rust backend and a responsive React frontend, Obsidian provides a seamless experience for server administrators.

## ✨ Features

-   **🖥️ Comprehensive Server Management:**
    -   Create, configure, and manage multiple Minecraft servers.
    -   Support for various server types: Vanilla, Fabric, Forge, NeoForge, and Quilt.
    -   Easy installation of different Minecraft and loader versions.
    -   Control server lifecycle: Start, Stop, Restart, and Kill operations.

-   **🔒 Robust User & Permission System:**
    -   Multi-user support with a role-based permission system.
    -   Secure authentication with password and token-based sessions.
    -   Admin panel for user management: create, edit, delete, and disable users.
    -   Force password resets for enhanced security.

-   **📊 Real-time Monitoring & Console:**
    -   Live server console with command input.
    -   Real-time host resource monitoring for CPU, memory, disk, and network usage.
    -   Graphical representation of resource history.
    -   Live player count and list.

-   **🗂️ Integrated File Manager:**
    -   Full-featured web-based file browser for each server instance.
    -   Standard file operations: Upload, Download, Move, Copy, Rename, and Delete.
    -   Built-in tools for archiving (`.zip`) and extracting files directly on the server.
    -   In-browser text editor for easy configuration file changes with syntax highlighting.

-   **🧩 Effortless Mod & Content Management:**
    -   Browse and install mods, modpacks, and more directly from Modrinth & CurseForge.
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
    -   **Language:** Rust
    -   **Framework:** Actix Web
    -   **Database:** SQLite, MySQL, or PostgreSQL (via SQLx)
    -   **Real-time Communication:** Server-Sent Events (SSE)
-   **Frontend:**
    -   **Framework:** React (with TypeScript)
    -   **Build Tool:** Vite
    -   **UI Components:** [HeroUI](https://heroui.net/)
    -   **Styling:** Tailwind CSS
    -   **Animations:** Framer Motion
    -   **Charting:** Recharts
-   **API:**
    -   Fully documented with an `openapi.yaml` specification.

## 🚀 Getting Started

### Prerequisites

-   For running the installation script: `curl` or `wget`, and `unzip` or `bsdtar`.
-   For building from source:
    -   [Rust toolchain](https://www.rust-lang.org/tools/install)
    -   [Node.js](https://nodejs.org/) (with pnpm enabled: `npm install -g pnpm`)
    -   [just](https://github.com/casey/just) command runner (recommended)

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

### Building from Source

This project uses [`just`](https://github.com/casey/just) as its command runner. Run `just --list` to see all available recipes.

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel.git
    cd obsidian-server-panel
    ```

2.  **Install frontend dependencies:**
    ```bash
    pnpm install
    ```

3.  **Build and package (frontend + backend):**
    ```bash
    just build              # SQLite (default)
    just build mysql        # MySQL
    just build postgres     # PostgreSQL
    ```

4.  **Or build components individually:**
    ```bash
    just build-frontend          # Frontend only
    just build-backend           # Backend debug build (SQLite default)
    just build-backend postgres  # Backend debug build (PostgreSQL)
    ```

5.  **Run the application:**
    ```bash
    just run                # SQLite (default)
    just run postgres       # PostgreSQL
    ```

The server will be available at `http://localhost:8080` by default.

For more details on development workflows, testing, and all available recipes, see the [Wiki](../../wiki).

## 📄 API Documentation

This project includes a comprehensive OpenAPI 3.0 specification for its backend API. You can find the full details in the `openapi.yaml` file. This can be used with tools like Swagger UI or Postman to explore and interact with the API endpoints.

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