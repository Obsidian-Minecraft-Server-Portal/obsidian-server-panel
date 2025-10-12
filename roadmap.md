# Obsidian Server Panel - Beta Release Roadmap

This document outlines features that need to be implemented or completed before the beta release.

---

## 🔴 Critical Priority - Must Implement Before Beta

These features are either completely non-functional or have significant frontend/backend mismatches that block core functionality.

### 1. Backup System Implementation ⚠️ CRITICAL
**Status:** Frontend fully implemented (666 lines), backend completely missing
**Location:** `src-actix/server/backups/backup_endpoint.rs` (currently just a stub)

**Required Backend Endpoints:**
- `GET /api/server/:id/backups` - List all backups for a server
- `POST /api/server/:id/backups` - Create a new backup
- `DELETE /api/server/:id/backups/:backupId` - Delete a backup
- `POST /api/server/:id/backups/:backupId/restore` - Restore from backup
- `GET /api/server/:id/backups/:backupId/download` - Download backup archive
- `GET /api/server/:id/backups/settings` - Get backup configuration
- `PUT /api/server/:id/backups/settings` - Update backup settings

**Features to Implement:**
- Full backup creation (entire server directory)
- World-only backups (just world folders)
- Git-based incremental backups (using obsidian-backups crate)
- Cron-based scheduling with retention policies
- Backup metadata storage (description, type, size, date, git commit)
- Compression and archiving

**Database Changes:**
- Add `backups` table to track backup metadata
- Add `backup_schedules` table for cron jobs

**Frontend Reference:** `src/components/server-components/server-page/backups/ServerBackups.tsx`

---

### 2. Modpack Installation System
**Status:** Discovery fully implemented, installation completely missing
**Location:** Frontend in `src/pages/DiscoverPage.tsx`, no backend implementation

**Required Implementation:**

#### A. Modpack Installation from Discovery Page
**Frontend Complete:**
- Full modpack browsing for Modrinth, CurseForge, ATLauncher, Technic
- Detailed modpack viewer with version selection
- API integration in `src/ts/api/modpack-api.ts`

**Missing Backend:**
- `POST /api/server/from-modpack` - Create server from modpack
  - Parameters: platform, modpack ID, version, server name, minecraft version
- Modpack manifest parsing:
  - CurseForge `manifest.json` format
  - Modrinth `modrinth.index.json` format
  - ATLauncher pack format
- Automatic mod dependency downloading
- Overrides/custom files extraction
- Server configuration based on modpack settings

#### B. Modpack Installation from Zip/Archive Upload
**Status:** Half-implemented - UI allows upload but no processing
**Location:** `src/components/server-components/new-server-modal/NewServerModal.tsx:364`

**Current Issue:**
- Frontend accepts `.jar,.zip,.tar.gz,.tar` files
- Backend receives file but doesn't process modpack structure
- File is treated as simple jar, not a modpack archive

**Required Backend Logic:**
1. Detect if uploaded file is a modpack (check for manifest files)
2. Parse modpack manifest
3. Extract modpack structure
4. Download required mods from APIs
5. Configure server based on modpack specifications
6. Handle overrides/custom configurations

---

### 3. Server Version Update System
**Status:** No implementation on either side
**Current Behavior:** Users must manually download and replace server jars

**Required Implementation:**
- `GET /api/server/:id/updates/check` - Check if newer version available
- `POST /api/server/:id/updates/apply` - Download and apply update
- Version comparison logic for each server type
- Safe update process:
  1. Stop server if running
  2. Backup current jar
  3. Download new version
  4. Replace jar
  5. Update database record
- Rollback capability if update fails

**Frontend Addition:**
- Update button in ServerOptions.tsx
- Update available indicator
- Update progress tracking

---

### 4. Create Server from Zip Archive
**Status:** Not implemented
**User Story:** User has existing server folder as zip, wants to import it

**Required Implementation:**
- `POST /api/server/from-zip` - Create server from uploaded zip
- Zip validation and structure detection
- Server type auto-detection from jar filename
- Minecraft version detection from version.json or server jar
- Import configuration (server.properties, world folders)
- Handle incomplete uploads with resume capability

**Frontend Addition:**
- New tab in NewServerModal for "Import from Zip"
- Drag-and-drop zip upload area
- Configuration override options

---

## 🟠 High Priority - Core Features Incomplete

These features are partially implemented or have significant functionality gaps that impact user experience.

### 5. Mod Update Checking
**Status:** Frontend logic exists, no backend integration
**Location:** `src/ts/mod-updates.ts` (227 lines), incomplete API key handling

**Current Issues:**
- Frontend can check updates but needs proper API keys
- No backend persistence of mod versions
- No bulk update functionality

**Required Implementation:**
- Backend mod version tracking in database
- `GET /api/server/:id/mods/check-updates` - Check all mods for updates
- `POST /api/server/:id/mods/:modId/update` - Update specific mod
- `POST /api/server/:id/mods/update-all` - Batch update all outdated mods
- Rate limiting and API key management for CurseForge/Modrinth
- Dependency compatibility checking before updates

---

### 6. World Discovery & Management System
**Status:** Navigation exists, no implementation
**Location:** `DiscoverPage.tsx:36` accepts "worlds" type, but no content

**Required Implementation:**

#### A. World Discovery
- Integration with world hosting platforms (e.g., Planet Minecraft, CurseForge world downloads)
- `GET /api/worlds/search` - Search for downloadable worlds
- `GET /api/worlds/:id` - Get world details
- `POST /api/server/:id/worlds/install` - Download and install world

#### B. World Management
- `GET /api/server/:id/worlds` - List all worlds in server
- `POST /api/server/:id/worlds/:worldName/backup` - Backup specific world
- `POST /api/server/:id/worlds/:worldName/restore` - Restore world
- `DELETE /api/server/:id/worlds/:worldName` - Delete world
- `POST /api/server/:id/worlds/upload` - Upload custom world
- Multi-world support (Multiverse detection)
- World reset/regeneration

**Frontend Components:**
- WorldsDiscoverPage variant
- World installation modal
- World management in ServerPage (new tab?)

---

### 7. Custom Jar Upload Processing
**Status:** Half-implemented - uploads but doesn't validate or process properly
**Location:** `NewServerModal.tsx:137-174`

**Current Issues:**
- Accepts any jar/zip but doesn't validate
- Doesn't detect if file is a modpack archive
- No extraction of packaged modpacks
- No jar integrity checking

**Required Fixes:**
- Server jar validation (check for main class)
- Automatic modpack detection and parsing
- Minecraft version extraction from jar manifests
- Proper error messages for invalid jars
- Support for universal/installer jars (Forge)

---

### 8. Server Statistics & Performance Tracking
**Status:** Real-time monitoring exists, no historical data
**Location:** `HostInfoProvider.tsx` streams SSE data, but nothing persists

**Required Implementation:**
- `server_stats` database table with time-series data
- Background task to periodically save stats
- `GET /api/server/:id/stats?period=24h` - Get historical stats
- Metrics to track:
  - CPU usage over time
  - RAM usage over time
  - Player count (parse from console logs)
  - TPS (Ticks Per Second) - parse from server console
  - Disk I/O
  - Network traffic
- Configurable retention policy (keep 24h, 7d, 30d aggregates)

**Frontend Additions:**
- Performance graphs in ServerPage
- Historical data visualization
- Performance alerts configuration

---

### 9. Quilt Version Selection
**Status:** Component exists but incomplete
**Location:** `src/components/server-components/version-selectors/QuiltVersionSelector.tsx`

**Issue:** No API calls to fetch Quilt versions (unlike Fabric/Forge selectors)

**Required Implementation:**
- Integrate with Quilt Meta API (https://meta.quiltmc.org)
- Fetch loader versions
- Fetch compatible Minecraft versions
- Match UI/UX of Fabric version selector

---

## 🟡 Medium Priority - Quality of Life Improvements

Features that would significantly improve usability but aren't blockers for beta release.

### 10. Server Templates & Presets
**Status:** Not implemented
**Benefit:** Faster server creation for common use cases

**Implementation:**
- Predefined templates:
  - Vanilla Survival (optimized settings)
  - Creative Build Server
  - Modded Kitchen Sink
  - Lightweight Modded
  - PvP Arena
- Template storage in database
- `GET /api/templates` - List available templates
- `POST /api/server/from-template` - Create from template
- Allow users to save custom templates from existing servers

**Frontend:**
- New tab in NewServerModal: "From Template"
- Template cards with descriptions
- Template customization before creation

---

### 11. Server Icon Upload UI
**Status:** Backend endpoint exists, no frontend UI
**Location:** Backend at `server_endpoint.rs:263-291`, missing in `ServerOptions.tsx`

**Required Frontend:**
- Icon upload area in ServerOptions
- Image preview
- 64x64 PNG validation
- Crop/resize tool

**Backend Already Complete:** `POST /api/server/:id/icon`

---

### 12. Server Cloning/Duplication
**Status:** Not implemented
**Use Case:** Testing configurations, creating similar servers

**Implementation:**
- `POST /api/server/:id/clone` - Duplicate entire server
- Options:
  - Include/exclude world data
  - Include/exclude mods
  - Include/exclude configurations
- Clone with new name and port
- Copy entire directory structure

**Frontend:**
- Clone button in ServerOptions
- Clone configuration modal

---

### 13. Automatic Mod Dependency Resolution
**Status:** Manual mod installation only
**Current Behavior:** User must manually download dependencies

**Implementation:**
- When installing mod, check dependencies via API
- Recursively resolve dependency tree
- Show dependency confirmation modal
- Batch install all required mods
- Detect and warn about dependency conflicts

**Enhancement to Existing:** Modify `POST /api/server/:id/mods/install` endpoint

---

### 14. File Editor External Modification Detection Improvements
**Status:** Basic polling exists, could be better
**Location:** `ServerFiles.tsx:190-208` polls every 3 seconds

**Improvements:**
- Websocket-based file watching (use `notify` crate on backend)
- SSE stream for file change events
- Auto-refresh option
- Diff view for external changes
- Conflict resolution for simultaneous edits

---

### 15. Resource Pack & Data Pack Management
**Status:** Navigation exists, minimal implementation
**Location:** `DiscoverPage.tsx:36-40` accepts types, no specialized UI

**Implementation:**
- Discover resource packs from CurseForge/Modrinth/Planet Minecraft
- `POST /api/server/:id/resourcepacks/install` - Auto-install to resourcepacks folder
- `POST /api/server/:id/datapacks/install` - Auto-install to world/datapacks
- Preview resource pack textures
- Data pack validation
- In-game data pack commands (enable/disable)

---

### 16. Scheduled Server Tasks
**Status:** Cron scheduling UI exists for backups, no general system
**Location:** `ServerBackups.tsx:461-496` has cron UI

**Implementation:**
- General task scheduler in backend (using cron expressions)
- Schedulable tasks:
  - Automatic backups (already in UI)
  - Automatic restarts
  - Automatic mod updates
  - Automatic server updates
  - Custom commands execution
- `GET /api/server/:id/tasks` - List scheduled tasks
- `POST /api/server/:id/tasks` - Create scheduled task

**Frontend:**
- New "Scheduled Tasks" tab in ServerPage
- Task list with enable/disable toggles
- Task execution history

---

### 17. Global Settings Page
**Status:** Not implemented
**Current Issue:** Settings scattered across localStorage, no persistence

**Required Settings:**
- Default Java version preference
- Default RAM allocation
- Auto-update preferences
- Theme selection (currently localStorage only)
- API keys for CurseForge/Modrinth
- Notification preferences
- Default backup retention policies

**Implementation:**
- `GET /api/user/settings` - Get user preferences
- `PUT /api/user/settings` - Update preferences
- New Settings page in navigation
- Setting categories: Appearance, Defaults, Integrations, Notifications

---

## 🟢 Low Priority - Nice to Have

Features that would be beneficial but can be deferred to post-beta releases.

### 18. Server Import/Export
**Status:** Not implemented

**Features:**
- Export server configuration as JSON
- Export entire server as portable archive
- Import server from configuration file
- Server migration tool (move between directories)

---

### 19. Multi-User Server Sharing
**Status:** User permissions exist, no sharing mechanism
**Current:** `UserPermissions` in backend, but no per-server access control

**Implementation:**
- Share server access with other users
- Per-server permission levels (Owner, Admin, Viewer)
- Invite system
- Collaborative server management
- Activity log (who did what)

---

### 20. Server Properties Editor UI
**Status:** Must use file browser currently

**Implementation:**
- Dedicated Server Properties tab in ServerPage
- Form-based editor with descriptions
- Input validation (e.g., port ranges, boolean values)
- Preset configurations (Peaceful, Hard, Ultra-Hardcore)
- Apply changes with server restart prompt

---

### 21. Enhanced Error Boundaries
**Status:** ErrorBoundary component exists but inconsistently used
**Location:** Used in `ServerPage.tsx:83-87`, missing elsewhere

**Improvement:**
- Wrap all major page components
- Add error reporting/logging
- User-friendly error messages
- Retry mechanisms
- Fallback UI

---

### 22. Forge Installation Progress Feedback
**Status:** Forge installer runs but no progress indication
**Issue:** Forge uses installer jars that take time to run

**Implementation:**
- Track Forge installer process
- Stream installer output to UI
- Progress indicator during installation
- Clear completion/error messages

---

### 23. Server Console Command Shortcuts
**Status:** Manual command typing only

**Enhancement:**
- Quick action buttons:
  - `/stop` (graceful stop)
  - `/save-all`
  - `/whitelist add [player]`
  - `/op [player]`
- Command history (up/down arrows)
- Command autocomplete
- Saved custom commands

---

### 24. Performance Alerts & Notifications
**Status:** No alerting system

**Implementation:**
- Configurable thresholds (CPU > 90%, RAM > 95%)
- Alert channels: In-app notifications, email, webhook
- Server crash detection
- Automatic actions (restart on crash, backup before restart)

---

### 25. World Import from Single-Player
**Status:** Not implemented

**Feature:**
- Upload world folder from user's .minecraft/saves
- Validate world structure
- Import to server
- Handle Nether/End dimension folders

---

## Database Schema Additions Required

### New Tables Needed:

```sql
-- For backup system
CREATE TABLE backups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id INTEGER NOT NULL,
    backup_type TEXT NOT NULL, -- 'full', 'incremental', 'world_only'
    description TEXT,
    file_path TEXT NOT NULL,
    size_bytes INTEGER,
    git_commit TEXT, -- for incremental backups
    created_at TEXT NOT NULL,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

CREATE TABLE backup_schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id INTEGER NOT NULL,
    cron_expression TEXT NOT NULL,
    backup_type TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    retention_days INTEGER,
    created_at TEXT NOT NULL,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

-- For statistics tracking
CREATE TABLE server_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id INTEGER NOT NULL,
    timestamp TEXT NOT NULL,
    cpu_percent REAL,
    ram_used_mb INTEGER,
    ram_total_mb INTEGER,
    disk_read_bps INTEGER,
    disk_write_bps INTEGER,
    network_sent_bps INTEGER,
    network_recv_bps INTEGER,
    player_count INTEGER,
    tps REAL, -- ticks per second
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

-- For scheduled tasks
CREATE TABLE scheduled_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id INTEGER NOT NULL,
    task_type TEXT NOT NULL, -- 'backup', 'restart', 'command', 'update'
    cron_expression TEXT NOT NULL,
    parameters TEXT, -- JSON for task-specific data
    enabled BOOLEAN NOT NULL DEFAULT 1,
    last_run TEXT,
    next_run TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

-- For user settings
CREATE TABLE user_settings (
    user_id INTEGER PRIMARY KEY,
    settings_json TEXT NOT NULL, -- JSON blob of all settings
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- For server templates
CREATE TABLE server_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT 0, -- system vs user template
    creator_id INTEGER,
    template_json TEXT NOT NULL, -- server config as JSON
    created_at TEXT NOT NULL,
    FOREIGN KEY (creator_id) REFERENCES users(id) ON DELETE SET NULL
);
```

---

## API Endpoints Summary

### Critical Endpoints Needed (17):
1. `GET /api/server/:id/backups`
2. `POST /api/server/:id/backups`
3. `DELETE /api/server/:id/backups/:backupId`
4. `POST /api/server/:id/backups/:backupId/restore`
5. `GET /api/server/:id/backups/:backupId/download`
6. `GET /api/server/:id/backups/settings`
7. `PUT /api/server/:id/backups/settings`
8. `POST /api/server/from-modpack`
9. `POST /api/server/from-zip`
10. `GET /api/server/:id/updates/check`
11. `POST /api/server/:id/updates/apply`
12. `GET /api/server/:id/mods/check-updates`
13. `POST /api/server/:id/mods/:modId/update`
14. `GET /api/worlds/search`
15. `POST /api/server/:id/worlds/install`
16. `GET /api/server/:id/worlds`
17. `GET /api/server/:id/stats`

### High Priority Endpoints (8):
18. `POST /api/server/:id/worlds/:worldName/backup`
19. `POST /api/server/:id/worlds/:worldName/restore`
20. `POST /api/server/:id/worlds/upload`
21. `POST /api/server/:id/mods/update-all`
22. `GET /api/templates`
23. `POST /api/server/from-template`
24. `POST /api/server/:id/clone`
25. `GET /api/server/:id/tasks`

---

## Implementation Priority Order

### Phase 1: Critical Features (Beta Blockers)
1. **Backup System** - Most critical, frontend complete
2. **Modpack Installation** - Core feature, discovery already done
3. **Server from Zip Import** - Essential for user onboarding
4. **Server Updates** - Basic server maintenance

### Phase 2: Core Completeness (Beta Quality)
5. **Mod Update Checking** - Already 80% done
6. **World Management** - Basic world operations
7. **Custom Jar Processing** - Fix existing feature
8. **Statistics Persistence** - Infrastructure already exists

### Phase 3: UX Polish (Beta+)
9. **Server Templates** - Significantly improves UX
10. **Quilt Support Completion** - Finish what's started
11. **Server Cloning** - Nice for power users
12. **Server Icon Upload** - Quick win, backend done

### Phase 4: Advanced Features (Post-Beta)
13. **Scheduled Tasks System**
14. **Resource Pack Management**
15. **Global Settings Page**
16. Everything else in Low Priority section

---

## Testing Requirements

Before beta release, ensure comprehensive testing of:

- [ ] Backup creation, restoration, and deletion
- [ ] Modpack installation from all supported platforms
- [ ] Server creation from zip archives
- [ ] Server version updates for all loader types
- [ ] File upload handling for large files (>1GB)
- [ ] Concurrent server operations
- [ ] Database migrations
- [ ] SSE connection stability
- [ ] Authentication and permission enforcement
- [ ] Cross-browser compatibility (Chrome, Firefox, Edge)
- [ ] Error handling and user feedback
- [ ] Resource cleanup on server deletion

---

## Documentation Requirements

Before beta:

- [ ] Installation guide (Windows, Linux, macOS)
- [ ] User manual covering all implemented features
- [ ] API documentation (OpenAPI spec update)
- [ ] Development setup guide
- [ ] Architecture documentation
- [ ] Troubleshooting guide
- [ ] Migration guide (if upgrading from alpha)

---

## Estimated Implementation Effort

**Critical Priority:** ~120-160 hours
- Backup system: 40-50h
- Modpack installation: 40-50h
- Server updates: 20-30h
- Server from zip: 20-30h

**High Priority:** ~80-100 hours

**Medium Priority:** ~60-80 hours

**Low Priority:** ~40-60 hours

**Total for Beta-Ready State:** ~200-260 hours (5-6.5 weeks full-time)

---

## Next Steps

1. Prioritize and assign tasks from Phase 1 (Critical Features)
2. Set up project tracking (GitHub Issues/Projects)
3. Create feature branches for each major feature
4. Implement database migrations
5. Write integration tests alongside implementation
6. Update OpenAPI spec as endpoints are added
7. Begin documentation writing
8. Schedule beta testing period

---

*Last updated: 2025-10-11*
*Version: 1.0*