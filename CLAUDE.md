# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Obsidian Server Panel is a modern web-based Minecraft server management system. The application uses a Rust backend (Actix Web) with a React frontend (TypeScript + Vite), designed to manage multiple Minecraft servers with support for various mod loaders (Vanilla, Fabric, Forge, NeoForge, Quilt).

## Build Commands

### Development
```bash
# Start frontend dev server with HMR (port 3000)
pnpm run dev

# Run backend in dev mode (auto-reloads on changes)
pnpm run run-api
# or: cargo run --bin obsidian_server_panel

# Watch mode - rebuilds backend on file changes
pnpm run watch
```

### Building
```bash
# Build frontend only (outputs to target/wwwroot)
pnpm run build-frontend

# Build backend only (release mode)
pnpm run build-api
# or: cargo build --release

# Build both frontend and backend
pnpm run build
```

### Testing
```bash
# Run Rust tests
cargo test

# Frontend tests (if applicable)
pnpm test
```

## Architecture

### Backend Structure (Rust - Actix Web)

The backend follows a modular architecture with the entry point in `src-actix/lib.rs`:

- **Entry Point**: `src-actix/main.rs` → `src-actix/lib.rs::run()`
- **Modules**:
  - `authentication/` - User authentication, permissions (bcrypt + token-based sessions), middleware
  - `server/` - Core server management logic
    - `server_data.rs` - Server state and lifecycle management
    - `server_actions.rs` - Start/stop/restart/kill operations
    - `server_endpoint.rs` - HTTP API endpoints
    - `server_db.rs` - Database operations for servers
    - `installed_mods/` - Mod tracking and management
    - `filesystem/` - File operations, archive/extract functionality
    - `backups/` - Backup system (full, incremental, world-only)
  - `host_info/` - System resource monitoring (CPU, RAM, disk, network via sysinfo)
  - `java/` - Java runtime management and version mapping
  - `actions/` - Generic action tracking system
  - `updater/` - Application update system
  - `actix_util/` - HTTP utilities, error handling, asset serving
  - `forge_endpoint.rs` - Forge-specific API endpoints

- **Database**: SQLite via SQLx (initialized in `app_db.rs`)
- **Real-time Updates**: Server-Sent Events (SSE) for console logs and resource monitoring
- **Dev Mode**: In debug builds, runs Vite dev server on separate thread and uses proxy
- **Working Directory**: Debug mode sets `./target/dev-env` as working directory

### Frontend Structure (React + TypeScript)

The frontend is in `src/` with this organization:

- **Entry Point**: `src/main.tsx` - Sets up React Router and provider hierarchy
- **Provider Hierarchy** (nested in order):
  - `WindowProvider` - Window state management
  - `PersistentActionProvider` - Long-running action tracking
  - `NotificationProvider` - Toast notifications
  - `ThemeProvider` - Accessibility-focused theming
  - `MessageProvider` - Global messaging system
  - `HostInfoProvider` - System resource data (SSE connection)
  - `AuthenticationProvider` - User session management
  - `MinecraftVersionsProvider` - Minecraft version data
  - `ServerProvider` - Server list and state management
  - `JavaVersionProvider` - Java runtime data

- **Pages** (`src/pages/`):
  - `Dashboard.tsx` - Home page with server list
  - `ServerPage.tsx` - Individual server management (console, files, options, backups)
  - `ContentPage.tsx` - Mod/modpack browser (Modrinth + CurseForge)
  - `DiscoverPage.tsx` - Content discovery
  - `Login.tsx` / `Signup.tsx` - Authentication

- **Components** (`src/components/`):
  - `extended/` - Wrapped HeroUI components with custom defaults (no rounded corners, Minecraft fonts)
  - `server-components/` - Server-specific UI components
  - `authentication/` - User management modals
  - `navigation/` - Top navbar and navigation
  - `icons/` - Custom SVG icons (NeoForge, Quilt)

### Key Architectural Patterns

**Backend:**
- Each feature module has `mod.rs`, `*_data.rs` (types), `*_db.rs` (database), `*_endpoint.rs` (HTTP handlers)
- Endpoints are registered via `configure()` functions in `lib.rs`
- Authentication middleware wraps protected routes
- SSE streams for real-time updates (console logs, resource monitoring)

**Frontend:**
- Context providers for global state management (no Redux/Zustand)
- SSE subscriptions in providers for real-time data
- React Router for navigation with AnimatePresence for transitions
- HeroUI component library with custom theme system
- Monaco Editor for in-browser file editing

### Database Schema

SQLite database initialized in `app_db::initialize_databases()`:
- Users table (authentication, permissions as bitflags via `enumflags2`)
- Servers table (configuration, paths, versions, Java settings)
- Installed mods tracking (auto-detected via filesystem watcher using `notify` crate)
- Actions table (persistent action tracking)

## Development Guidelines

### Backend (Rust)

**Code Style:**
- Use `rustfmt.toml` configuration (should exist in root)
- Error handling: Use `anyhow::Result` for main logic, `thiserror` for custom errors
- Logging: `log::info!`, `log::debug!`, `log::error!` (filtered by DEBUG const)

**Adding New Endpoints:**
1. Create types in `*_data.rs`
2. Add database operations in `*_db.rs` (use SQLx compile-time verification)
3. Implement handlers in `*_endpoint.rs`
4. Register in `lib.rs` via `configure()` function
5. Wrap in auth middleware if needed

**Server Lifecycle:**
- Servers are spawned as child processes (via `tokio::process::Command`)
- Console I/O uses `tokio_interactive` for interactive process management
- Server state tracked in `ServerData` static map
- Resource monitoring via `sysinfo` crate with periodic polling

### Frontend (React + TypeScript)

**Code Style:**
- Follow `style-guide.md` for UX consistency
- All HeroUI components must use `radius="none"` (sharp corners)
- Typography: Use `font-minecraft-header` for headings, `font-minecraft-body` for UI
- Icons: Prefer pixel art style (`pixelarticons:*`, `pixel:*`)
- No rounded corners anywhere in the UI

**Theming:**
- Support 4 themes: default dark, deuteranopia-friendly, tritanopia-friendly, monochrome
- Use HeroUI theme variables, never hardcode colors
- Test all changes across all accessibility themes

**Adding New Pages:**
1. Create page component in `src/pages/`
2. Add route in `src/main.tsx`
3. Update navigation in `src/components/navigation/Navigation.tsx`
4. Use Framer Motion for page transitions

**State Management:**
- Use React Context for global state (no external state library)
- SSE connections managed in provider components
- Prefer composition over prop drilling

### API Integration

- OpenAPI specification in `openapi.yaml` (reference for endpoints)
- Base API path: `/api`
- Protected routes require authentication token in headers
- SSE endpoints for real-time data: `/api/host/resources`, `/api/server/:id/console`

## Important Notes

**File Paths:**
- Backend working directory changes in debug mode to `./target/dev-env`
- Frontend build output: `target/wwwroot` (served by backend in production)
- Server instances stored in user-specified directories

**Development Mode Detection:**
- `DEBUG` const in Rust: `cfg!(debug_assertions)`
- Vite proxy only runs in debug builds
- Frontend connects to different ports: dev (localhost:3000) vs production (same as backend)

**Permission System:**
- Uses bitflags (`enumflags2` crate) for user permissions
- Checked via middleware on protected routes
- Admin users have full access, regular users have restricted permissions

**Mod Loader Support:**
- Each loader has version provider: `ForgeVersionsProvider`, `FabricVersionsProvider`, etc.
- Version selectors in `src/components/server-components/version-selectors/`
- Backend handles loader-specific installation logic in `forge_server.rs` and similar

**Backup System:**
- Powered by `obsidian-backups` crate
- Supports full, incremental, and world-only backups
- Cron-based scheduling with retention policies
- UI in `src/components/server-components/server-page/backups/ServerBackups.tsx`

## Configuration

**Command Line Args** (see `command_line_args.rs`):
- `--port` or `-p`: Web server port (default: 8080)
- `--forward-webpanel`: Enable UPnP port forwarding

**Environment:**
- `RUST_LOG`: Control log level (debug builds default to Debug, release to Info)

## Common Tasks

**Add New Server Type:**
1. Add variant to `ServerType` enum in `server/server_type.rs`
2. Implement installation logic in backend
3. Add version provider in `src/providers/LoaderVersionProviders/`
4. Create version selector component
5. Update `NewServerModal.tsx` with new option

**Add New Permission:**
1. Add to `UserPermissions` bitflags in `authentication/user_permissions.rs`
2. Update middleware checks in `authentication/auth_middleware.rs`
3. Update UI permission checks in `AuthenticationProvider.tsx`

**Add Real-time Monitoring:**
1. Create SSE endpoint in backend (return `EventSource` response)
2. Create provider in `src/providers/` with SSE connection
3. Subscribe to updates in relevant components

**Modify Database Schema:**
1. Update table definitions in `*_db.rs` `initialize()` function
2. Write migration if needed (manual SQLite schema updates)
3. Update corresponding data types in `*_data.rs`
