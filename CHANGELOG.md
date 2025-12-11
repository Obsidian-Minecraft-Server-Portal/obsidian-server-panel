# Changelog

## [Unreleased]

### Fixed
- Fixed Java directory migration when changing paths in settings
  - Now supports merging directories when destination already exists (source takes priority)
  - Prevents migration when servers are running with Java from the old directory
  - Uses efficient directory renaming instead of recursive copying to avoid "Access denied" errors
  - Provides detailed error messages for common Windows permission issues
- Fixed Java versions tab not refreshing when navigating to it or after changing Java directory
  - Java installations list now refreshes automatically when the Java tab is opened
  - Java versions refresh after saving settings when Java directory is changed
  - Ensures users always see current Java installations from the active storage location
  - Optimized to only call the API once per tab navigation (removed duplicate refresh calls)

## [1.0.0-alpha.1] - 2025-10-28

This is the first alpha release of Obsidian Server Panel, a modern web-based Minecraft server management system. This release includes the core functionality for managing multiple Minecraft servers with support for various mod loaders (Vanilla, Fabric, Forge, NeoForge, Quilt).

### Features

#### Server Management
- Multi-server management with support for Vanilla, Fabric, Forge, NeoForge, and Quilt
- Auto-select Java executable based on Minecraft version
- Java version map persistence with daily scheduler and API integration
- Java version map expiration check with enhanced database schema
- Server process management with improved readability and error handling
- UPnP port forwarding support with automatic cleanup on server stop
- Server creation status messaging improvements

#### Backup System
- Full backup and restore functionality
- Incremental backup support
- World-only backup option
- Streamlined backup download and deletion logic
- Cron-based scheduling with retention policies

#### File Management
- File download capability with refactored browser layout
- Folder upload with recursive file handling
- In-browser file editor with unsaved changes detection
- External file modification handling
- Archive/extract functionality with bug fixes

#### Content Discovery & Mod Management
- Multi-platform modpack discovery (Modrinth, CurseForge, ATLauncher)
- ATLauncher API integration with GraphQL
- Enhanced DiscoverPage with multi-platform state management and animations
- Platform-specific filtering capabilities
- Mod version fetching and display
- ModItemContentDrawer component with tabs for description, changelog, and versions
- Strongly typed loader support in InstalledModList

#### User Interface
- Notification system with management capabilities (mark as read, delete, delete all)
- Persistent action tracking system with API endpoints and frontend polling
- Collapsing header logic and motion animations on ServerPage
- Improved form validation and user feedback for server creation
- Accessibility improvements (tabIndex for non-focusable elements)
- Relative date/time formatting for better UX
- Enhanced responsiveness and usability across components

#### Installation & Deployment
- Linux installer script with interactive prompts
- Uninstall functionality
- Web UI port configuration
- Systemd service integration
- Service restart on deployment

### Improvements

- Replaced `obsidian-upnp` with `easy-upnp` for improved UPnP handling
- Optimized release build settings for smaller binary size and improved performance
- Improved Vite server integration and updated `vite-actix` dependency
- Enhanced installer script robustness with better error handling
- Updated dependencies to latest compatible versions
- Refactored server action logic for improved clarity
- Improved mod-related components for better reusability and maintainability

### Bug Fixes

- Fixed archiving bug for paths inside directories
- Fixed typos in `PersistentActionProvider` interfaces
- Fixed installer config directory creation and systemd service variable usage
- Fixed installer variable syntax in extraction log messages
- Close all UPnP ports when server stops to ensure clean teardown

### Documentation

- Added `CLAUDE.md` with comprehensive project architecture and tooling details
- Added UX Style Guide for consistent user experience
- Added beta release roadmap with critical feature breakdown
- Added Terms of Service and Privacy Policy documents
- Updated README with feature list, installation steps, and license details
- Added OpenAPI specification and JetClient configuration files

### Refactoring & Code Cleanup

- Removed unused code, modules, and dependencies
- Removed unused upload and archive logic, streamlined editor operations
- Removed unused `description`, `logo`, and `developers` fields from modpack API
- Updated `obsidian-scheduler` features in `Cargo.toml`
- Reordered imports for consistency
- Moved assets from subdirectory to frontend root
- Removed platform-specific icon setup in `build.rs`
- Asset organization improvements

### Infrastructure

- Added `.cargo/config.toml` with cross-compilation target configurations
- Added IntelliJ project configuration files
- Updated `.gitignore` to exclude IntelliJ Copilot configuration files
- Added WSL target support in IDE configuration
- Enhanced data source setup with library references

---

**Full Changelog**: https://github.com/YOUR_USERNAME/YOUR_REPO/compare/0.0.0...1.0.0-alpha.1