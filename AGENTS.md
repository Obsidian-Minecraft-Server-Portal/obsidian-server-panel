# AI Agent Role

You act as a senior
software engineer with deep expertise in performance-critical, cross-platform
architecture. You think in dependency graphs and always reason about side effects.

## The TAVRN Stack

Most projects here use the **TAVRN** stack. Assume this unless the codebase shows
otherwise:

- **A** — **Actix**: Rust backend / web framework
- **V** — **Vite**: frontend build tool and dev server
- **R** — **React**: UI library, written in **TypeScript**
- **N** — **Next-UI**, now **HeroUI**: React component library

**This project does not use Tauri.** Obsidian ships as a single native binary
(Docker image, standalone binary with a tray icon, or an OS service) that serves
the web UI from Actix. Never add Tauri.

Languages: **TypeScript** (frontend), **Rust** (Actix), and **PHP** for
server-side pieces where present.

Build & tooling: tasks are orchestrated through a **`justfile`** (run recipes with
`just <recipe>`), and builds run in **Docker** for reproducible, cross-platform
output. Prefer `just` recipes and the Docker-based build over ad-hoc commands.

Notes for HeroUI (v3): requires Tailwind CSS v4, needs no provider, uses compound
components (e.g., `Card.Header`, `Card.Content`), and prefers `onPress` over
`onClick`.

## Core Principle

Every change has consequences. When advising on a change, your job is to find ALL
of them before they become bugs.

## Analysis Protocol

When reasoning about a change or answering a question about impact, work through:

1. **Dependency Mapping**: Search the entire project for every file, function,
   type, trait, interface, and module that references the code in question. Use
   grep, ripgrep, or file-search tools aggressively. Do NOT rely on memory or
   assumptions.

2. **Impact Classification**: For each dependent, classify the impact:
   - **Direct**: Code that calls, imports, or implements the interface
   - **Indirect**: Code that depends on behavior or side effects (event ordering,
     state mutations, timing)
   - **Cross-platform**: Platform-specific code paths that may need parallel
     updates (Windows/macOS/Linux/iOS/Android variants)

3. **Side Effect Analysis**: Explicitly enumerate:
   - Type signature changes and their propagation
   - Behavioral changes (return values, error conditions, timing)
   - State management impacts (shared state, caches, persistence)
   - Threading/concurrency implications (race conditions, deadlocks)
   - Performance implications (allocation patterns, hot paths, UI thread blocking)
   - Platform-specific behavior differences
   - Serialization/deserialization compatibility (config files, IPC, network)

## Cross-Platform Considerations

- Check for platform-conditional compilation (`#[cfg]`, platform folders,
  `.ios.ts`/`.android.ts` variants)
- Consider file path separators, line endings, and OS-specific APIs
- Consider screen density, input methods (touch vs mouse vs keyboard), and
  platform UI conventions
- Consider memory constraints on mobile vs desktop

## Performance Mindset

- Flag allocations introduced in hot paths without justification
- Prefer zero-copy and borrowing over cloning
- Heavy work belongs on background threads, not the UI thread
- Recommend profiling before and after performance-critical changes

## Communication Style

- Present dependency analysis first: what you found, what is affected, and the plan
- If you discover risks that cannot be fully mitigated, explicitly flag them
- If a question is too ambiguous to answer confidently, ask for clarification
  rather than guessing

## Tooling Rules

- Prefer the project's `just` recipes and Docker-based build over ad-hoc commands;
  check the `justfile` for the canonical way to build, test, and lint
- Verify correctness with the stack's tooling: `cargo clippy` for Rust
  (Actix), `tsc` / the project's linter for the TypeScript frontend, and the
  configured PHP linter where PHP is used
- Never use Python for any tooling or scripts

## Commits

You may create commits. When you do, follow this rule exactly:

> Commit all uncommitted changes, separate into sections and create multiple
> commits when it makes sense. Never add any AI attributions.

Do not add "Co-Authored-By", "Generated with", or any other AI attribution to
commit messages — write only the message content.

## Changelogs and Release Notes

When asked to write a changelog or release notes, update the associated
release with the text. The associated release is the GitHub release for the
version being documented (its `vX.Y.Z` tag); write the text with
`gh release edit <tag> --notes-file <file>`. Always use the
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) standard.

## Code Rules

These apply to every file written in this repo, by humans or agents:

- **No unnecessary code comments.** Code should explain itself through names
  and structure. Comment only non-obvious *why*, never *what*.
- **No single file longer than 750 lines.** Split into modules first.
- **Readable and performant.** Prefer clear, direct code. Be concise; be
  verbose only when the problem demands it.
- **Never reinvent the wheel.** Use well-maintained libraries and crates
  whenever one fits, instead of hand-rolling (HTTP clients, hashing, archive
  handling, cron-like time math, image processing, SSH/SFTP, UPnP, TOTP,
  cropping UIs, etc.).

## Agent Memory

Update your persistent agent memory as you discover codepaths, dependency
relationships, architectural patterns, platform-specific implementations, and
performance-critical sections. This builds institutional knowledge across
conversations. Write concise notes about what you found and where.

Examples of what to record:

- Key dependency chains (e.g., "a shared config struct is used in 14 files across 3 crates")
- Platform-specific code locations and patterns
- Performance-critical hot paths and their constraints
- Shared state and synchronization mechanisms
- Build configuration and feature flag relationships

### Recorded notes

- **Scrollbars:** `src/scrollbars.ts` attaches OverlayScrollbars to every
  scroll area automatically:
  - It runs a `MutationObserver` on `document.body` and rescans on resize.
  - It uses the element itself as the viewport, so DOM, refs, Lenis and
    console auto-follow are untouched.
  - It skips Monaco and anything with `scrollbar-width: none`.
  - The theme is `.os-scrollbar.os-theme-obsidian` in `src/css/index.css`. The
    double class is required, because the library's own defaults load later.
- **Smooth scroll:** `src/components/layout/SmoothScroll.tsx` (Lenis).
  - Areas that must keep native wheel behaviour carry `data-native-scroll`.
  - Programmatic scrolls inside Lenis wrappers use `lenis.scrollTo`.
  - Instant writes use `scrollTo({ behavior: "instant" })`.
- **HeroUI v3 Checkbox:** only `Checkbox.Content` is clickable, so
  `Checkbox.Control` must sit inside it.
- **React Aria menus:** a menu item is focused on hover. Highlight with
  `data-hovered` and `data-focus-visible`, never `data-focused`.
- **Server migration:**
  - The `obsidian-migration` crate holds the orchestration. Compatibility
    lives in `obsidian-content/src/manager/compat.rs`, and jar Minecraft
    ranges in `metadata.rs` and `range.rs`.
  - `InstanceManager::begin_migration` blocks `start()` while a migration
    runs.
  - `update_and_wait` awaits the install task.
  - Rollback restores the snapshot, the software fields and the `mods` rows.
- **Dev server quirks:**
  - `just dev` proxies the page to a Vite it starts itself on port 3000. If
    something else holds 3000, every page returns 500.
  - After a dependency change, Vite's cold optimise can take more than 60s,
    which exceeds the proxy timeout, so the first load fails. Wait and reload.
  - To verify UI without touching a running `just dev`, run a second Vite on
    another port. Give it its own `cacheDir`, because a shared
    `node_modules/.vite` makes the two servers re-optimise each other. Give it
    an `/api` proxy that rewrites `Origin`.
- **Windows tests:** `obsidian-runner::runner_round_trip` (needs `/bin/sh`)
  fails on Windows. `obsidian-backups::snapshot_deduplicates_and_restores` can
  fail intermittently with "Access is denied". Neither is a regression.

---

# Obsidian — Project Specification

Obsidian is a self-hostable **Minecraft server panel**: a webserver with an open
API and a web UI for creating, running, and managing Minecraft server instances.
The name refers to obsidian, the block nether portals are built from, which is
why the brand colour is **purple**.

It is free to distribute. Drew also runs a small hosting company that sells VPSs
running Obsidian, so headless, env-var-driven provisioning matters as much as
the desktop experience.

This is a **rewrite**. The previous attempt lives at the git remote
(`Obsidian-Minecraft-Server-Portal/obsidian-server-panel`). Study it for domain
knowledge, but do **not** copy its structure. It was abandoned because of:

- 1–2k-line files (`backup_manager.rs`, `ServerFiles.tsx`, `filesystem_endpoint.rs`).
- A ~2.5k-line HeroUI v2-compat shim layered over HeroUI v3.
- SQL DDL duplicated across sqlite/mysql/postgres, with no migrations.
- Java downloads reimplemented instead of using `piston-mc`.
- Server-process logic duplicated in two places, plus a GNU `screen` dependency.
- Realtime split across SSE and a separate WebSocket.
- Coarse, global-only permission flags.

## 1. Product & Distribution

- **One binary, three ways to run it:**
  1. **Docker image**, configured entirely through `OBSIDIAN_*` env vars.
  2. **Standalone binary** for Windows, macOS, and Linux. It shows a system tray
     icon (`tray-icon`) that opens the panel in the default browser.
  3. **OS service**: Windows Service, launchd, or systemd via `service-manager`,
     installed with `obsidian service install`.
- **Hosting model:** one Obsidian panel per customer VPS. There is no
  multi-node / node-agent system. Everything the setup wizard asks for must also
  be settable through env vars, so VPS images provision headlessly.
- **Minecraft servers run raw** as native Java processes. Never run Minecraft
  inside Docker. Inside the Docker image, the servers are plain processes in the
  same container.
- **Open API:** REST under `/api/v1`, an OpenAPI 3.1 spec generated with
  `utoipa`, and docs served at `/api/docs` (Scalar). Personal API tokens are
  limited to the permissions of the user who created them.
- **piston-mc** (Drew's crate) handles all Mojang downloads:
  - `manifest_v2::ManifestV2::fetch()` → `.version(id)` → `VersionManifest::download_server(..)` for vanilla jars.
  - `java::JavaManifest::fetch()` → per-platform `Runtimes` → `JavaRuntime::install(..)` for Java.
  - It does **not** cover mod loaders. Those live in `obsidian-loaders`.

## 2. Repository Layout

`src-actix/` is a **thin** binary: HTTP routes, middleware, the WebSocket hub,
the CLI entry point (`serve`, `runner`, `service`, `reset-password`), and the
tray. No business logic lives there.

Workspace crates in `crates/`:

| Crate                 | Responsibility                                                                                                                     |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------|
| `obsidian-common`     | Shared types: data paths, `Loader` enum, `EventBus`, `TaskRegistry`, common errors                                                 |
| `obsidian-db`         | sqlx SQLite pool, `sqlx::migrate!` migrations, in-memory DB + flusher, repositories                                                |
| `obsidian-auth`       | Users, argon2id, sessions, TOTP (`totp-rs`), trusted IPs, invites, password reset, the permission model                            |
| `obsidian-java`       | Java runtimes via piston-mc, plus detection of Java already installed on the system                                                |
| `obsidian-loaders`    | One `ServerLoader` trait. Implementations: Vanilla (piston-mc), Fabric, Quilt, Forge, NeoForge, Paper, Purpur, Spigot/CraftBukkit (BuildTools), Custom Jar |
| `obsidian-content`    | One `ContentPlatform` trait. Implementations: Modrinth, CurseForge, Hangar, Spiget. Covers mods, plugins, and modpacks             |
| `obsidian-runner`     | The detached per-instance supervisor process (§5)                                                                                  |
| `obsidian-instances`  | Instance lifecycle, config, `server.properties`, runner IPC client, event bus                                                      |
| `obsidian-backups`    | Content-addressed incremental backup store (§7)                                                                                    |
| `obsidian-scheduler`  | User-friendly schedules and event triggers (§8)                                                                                    |
| `obsidian-net`        | UPnP port forwarding (`igd-next`)                                                                                                  |
| `obsidian-sftp`       | Embedded SFTP server (`russh` + `russh-sftp`)                                                                                      |
| `obsidian-migration`  | Server software migration: compatibility scan, guarded upgrade with optional backup, rollback                                      |

The frontend lives in `src/` (Vite root at the repo root) and builds into
`target/wwwroot`. Actix embeds it with `include_dir`. In debug builds,
`vite-actix` proxies to the Vite dev server.

**Hard rules:**

- No source file may exceed **750 lines**. Split modules before they grow past
  that.
- One SQL dialect: SQLite only.
- No duplicate implementations. Java always comes through piston-mc, every
  loader implements `ServerLoader`, and every content source implements
  `ContentPlatform`.
- One realtime channel (§9).
- Use HeroUI v3 directly. Never write a v2-compat shim.

## 3. Database (SQLite + sqlx)

### Storage model

- **No WAL journaling.** The live database is **in-memory**, opened through a
  shared in-process memdb URI (`file:/obsidian?vfs=memdb`) so every pooled
  connection sees the same data.
- **On boot:** if `data/obsidian.db` exists, load it into memory with the
  SQLite backup API (`libsqlite3-sys`, via the connection's raw handle). Then
  run migrations.
- **Persistence:** a **single writer task** dumps the database every **30
  seconds**, but only when a dirty flag is set. It runs
  `VACUUM INTO 'obsidian.db.tmp'`, fsyncs, then atomically renames the file to
  `obsidian.db`.
- The writer also flushes on graceful shutdown and on SIGTERM/Ctrl-C, so
  `docker stop` is safe.
- **Known risk:** a hard crash or power loss loses up to 30 s of writes. Keep
  anything that must not be lost (for example backup snapshot manifests) on
  disk in its own store; the DB only indexes it.
- Migrations live in `crates/obsidian-db/migrations/*.sql` and are applied on
  boot.

### Tables

- **Instances**
  - `instances`: owner_id, name, loader, mc_version, loader_version,
    java_runtime_id, jvm_args, memory_min/max, port, upnp_enabled, auto_start,
    icon path, directory, created_at, updated_at.
  - `instance_members`: instance_id, user_id, role, permission overrides
    (bitset). This drives sharing.
- **Installed Minecraft versions**
  - `minecraft_versions`: loader, mc_version, loader_version, jar/cache path,
    installed_at.
- **Java versions**
  - `java_runtimes`: source (`piston` | `system`), component (e.g.
    `java-runtime-delta`), version, path, arch, installed_at.
- **Mods and plugins**
  - `mods`: Drew's schema, extended so plugins can share the table:

```sql
CREATE TABLE IF NOT EXISTS mods
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    filename    TEXT    NOT NULL,
    project_id  TEXT    DEFAULT NULL, -- CurseForge project id or Modrinth slug; NULL for manual uploads
    version_id  TEXT    DEFAULT NULL, -- NULL for manual uploads
    icon        BLOB    DEFAULT NULL, -- PNG icon extracted from the jar or fetched from the platform
    platform    INTEGER DEFAULT NULL, -- 0 = CurseForge, 1 = Modrinth, 2 = Hangar, 3 = Spiget, NULL = manual upload
    kind        INTEGER NOT NULL DEFAULT 0, -- 0 = mod, 1 = plugin
    instance_id INTEGER NOT NULL REFERENCES instances (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_mods_instance ON mods (instance_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_mods_instance_file ON mods (instance_id, filename);
```

- **Users and auth**
  - `users`: username, email, password_hash (argon2id), is_admin,
    instance_limit, totp_secret (encrypted at rest), totp_enabled, created_at.
  - `user_recovery_codes`: hashed TOTP recovery codes.
  - `user_trusted_ips`: user_id, ip, first_seen, last_seen, user_agent. This
    backs the "authorized IPs" system.
  - `sessions`, `api_tokens` (hashed), `invites` (hashed token, expiry,
    created_by).
  - `password_resets`: hashed token, expiry, used_at.
  - `email_verification_codes`: codes for new-IP verification when SMTP is set.
  - `security_events`: logins, new IPs, MFA changes.
- **Other**
  - `backup_snapshots`, `backup_policies`, `schedules`, `schedule_runs`,
    `notifications`, `settings` (key/value).

## 4. Authentication & Users

- **Account creation:** the setup wizard creates the first admin. After that,
  accounts come only from **one-time invite links** or are created by an admin.
  There is no open signup.
- **Global roles:** `Admin` and `User`. Admins set a per-user limit on how many
  instances that user may create.
- **Passwords:** argon2id. Sessions are an HttpOnly, Secure, SameSite=Lax
  cookie, with CSRF protection on mutating requests.
- **MFA:** TOTP (authenticator apps) plus single-use recovery codes.
- **Authorized IPs (new-IP verification):** when a login comes from an IP not
  in `user_trusted_ips`:
  1. If TOTP is enabled, require the TOTP code.
  2. Else, if SMTP is configured, require an emailed 6-digit code.
  3. Else, **allow the login and notify**: record the IP and send an in-panel
     security notification to the user and to admins.

  On success the IP is remembered as trusted. Users can view and revoke their
  trusted IPs.
- **Password reset:** an emailed reset link. It is enabled only when SMTP is
  configured, either in the setup wizard (self-hosted) or through
  `OBSIDIAN_SMTP_*` env vars (VPS). For lockout recovery there is a CLI command:
  `obsidian reset-password <username>`.
- **Instance sharing:** instances have an owner. Owners share them with other
  users through **preset roles that expand to granular permissions**, which the
  owner can then tweak per member.
  - Roles: `Viewer`, `Operator`, `Moderator`, `Admin`, `Co-owner`.
  - Permissions (`group.action`):
    - `console.read`, `console.send`
    - `power.start`, `power.stop`, `power.restart`, `power.kill`
    - `files.read`, `files.write`, `files.delete`, `files.sftp`
    - `content.install`, `content.remove`
    - `backups.create`, `backups.restore`, `backups.download`, `backups.delete`
    - `schedules.manage`, `players.manage`, `settings.edit`, `members.manage`
  - Permissions are stored as a bitset. The role is kept only for display.
  - Every endpoint, WebSocket topic, and SFTP operation is checked against these
    permissions.

## 5. Instance Process Model (detach + reattach)

Minecraft servers must **keep running when the panel restarts or updates**.
GNU `screen` is not allowed.

- The panel spawns `obsidian runner --instance <id>`, the same binary in
  supervisor mode, as a **detached** process: `setsid` on Unix,
  `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP` on Windows.
- The runner:
  - Launches Java and owns the server's stdin/stdout.
  - Keeps a ring buffer of recent console output.
  - Writes a pidfile.
  - Serves a local IPC endpoint (Unix domain socket / Windows named pipe, via
    the `interprocess` crate) for console streaming, commands, and status.
- On boot, the panel scans pidfiles and IPC endpoints and **reattaches** to
  every live runner. A stale pidfile marks the instance as crashed.
- Graceful stop sends `stop`, waits for a configurable timeout, then kills.
- Instances with `auto_start` are started when the panel boots and no runner is
  alive for them, for example after a host reboot.
- Stats come from:
  - `sysinfo` on the Java PID: CPU, RAM.
  - Server List Ping: players, MOTD.
  - Log parsing: TPS (where the loader reports it), crash detection.

## 6. Instances & Content

- **Instance creation wizard:** server type → Minecraft version → loader
  version → Java. Java is picked automatically from the version manifest's
  `java_version` and can be overridden.
  - Alternatively, **search CurseForge or Modrinth for a modpack** and create
    the instance from it, auto-selecting the right loader.
- **Supported server types:** Vanilla, Fabric, Quilt, Forge, NeoForge, Paper,
  Purpur, Spigot, CraftBukkit, and Custom Jar.
- **Spigot/CraftBukkit** are compiled with **BuildTools** using a piston-mc Java
  runtime. The resulting jar is cached for reuse, and progress is streamed as a
  task. Never download Spigot/CraftBukkit from third-party mirrors.
- **Content sources:**
  - Mods: Modrinth, CurseForge.
  - Plugins: Modrinth, Hangar (PaperMC), CurseForge (Bukkit), Spiget
    (SpigotMC). Premium or external-download Spiget resources are shown as
    unavailable for automatic install.
- **Project pages:** every Discover result opens a platform page with the
  rendered README, gallery, changelog and versions. Modrinth and CurseForge
  provide the full page; Hangar and Spiget show what their APIs return.
  **Quick add** installs the newest build that fits the target instance with
  its required dependencies. Descriptions and changelogs are sanitized on the
  server before they reach the browser.
- **CurseForge API key:** injected at **build time** through an env var for
  official builds. Never commit the key to the repo. Source builds without a key
  must disable CurseForge features cleanly.
- **Mod icons:** extracted from the jar (`fabric.mod.json`,
  `quilt.mod.json`, `META-INF/mods.toml`, `neoforge.mods.toml`, `plugin.yml`,
  `paper-plugin.yml`) or fetched from the platform, then stored as a PNG BLOB.
- **Instance pages:**
  - **Console** (primary): live output with ANSI colours, command input with
    history and autocomplete.
  - **Content**: one tab with two sections. *Mods & plugins* lists installed
    content with updates, enable/disable and removal, and browses and installs.
    *Files* is a complete file browser with a Monaco editor, chunked and
    resumable uploads of any size, multi-select, zip/unzip, rename, move, and
    delete. The `files` tab redirects to `content?section=files`.
  - **Backups**, **Schedules**.
  - **Players**: ops, whitelist, bans.
  - **Properties**: a `server.properties` form editor with descriptions.
  - **Settings**: Java, memory, JVM args, port, UPnP, members/sharing.
- **Server icon:** the user uploads an image of **any size**. An in-browser
  crop/edit tool (`react-easy-crop`) produces a square crop. The backend
  resizes it to **64×64** with the `image` crate (Lanczos3) and writes
  `server-icon.png` into the instance directory.
- **UPnP:** a per-instance toggle that fully port-forwards the server port
  through `igd-next`. It renews the lease while the server runs, removes the
  mapping on stop, and shows mapping status and the external IP in the UI.
- **SFTP:** a built-in SFTP server. Users authenticate with their panel
  credentials or an API token. Each accessible instance appears as a top-level
  folder. Access requires `files.sftp`, and reads and writes are gated by
  `files.*`.

## 7. Backups (full incremental, content-addressed)

- **Storage:** restic-style.
  - Files are split into chunks with **FastCDC**, hashed with **BLAKE3**,
    compressed with **zstd**, and stored once in a per-instance chunk store.
  - Each backup is a snapshot manifest (a tree of files to chunk hashes). Every
    backup is incremental, and every snapshot can be restored on its own.
- **Consistency:** while the server is running, send `save-off` and
  `save-all flush` before reading files, and `save-on` afterwards.
- **Retention:** tiered (keep N hourly / daily / weekly / monthly). Pruning is
  mark-and-sweep garbage collection of unreferenced chunks.
- **Restore options:**
  - Full instance.
  - A single file or folder.
  - Download a snapshot as a zip.
- **Other:**
  - `.gitignore`-style exclude patterns.
  - The UI shows snapshot sizes, total store size, and the dedup ratio.
  - Backups are triggered from the scheduler (§8) or manually.

## 8. Scheduler (user friendly, **not** cron)

- Schedules are stored as structured JSON, never as cron strings. Supported
  kinds:
  - Every N minutes or hours.
  - Daily at a time.
  - Weekly on chosen weekdays at a time.
  - Monthly on a day.
  - One-off at a date/time.
  - Event triggers: on start, on stop, on crash, server empty for N minutes,
    player joined.
- Times are timezone-aware (`chrono-tz`), using the panel's configured timezone.
- **Actions** are a chainable list of steps, with optional delays and a
  "only if running" condition:
  - Backup.
  - Restart with in-game countdown broadcasts (e.g. 5m / 1m / 10s).
  - Send a command.
  - Start, stop.
  - Update content.
- **UI:**
  - A plain-English preview, e.g. "Every Mon & Wed at 4:00 AM".
  - The next few run times.
  - Run history with results.

## 9. API & Realtime

- REST lives under `/api/v1`. Every route is annotated for `utoipa`.
- There is exactly **one** WebSocket, `/api/v1/ws`, multiplexed by topic:
  - `instance.{id}.console`, `instance.{id}.status`, `instance.{id}.stats`.
  - `tasks`, `notifications`, `backups`.
- Subscriptions are permission-checked per topic. Do not add SSE endpoints.
- Long-running operations are tracked as **tasks** with progress published on
  `tasks`. These include downloads, Java installs, loader installs, BuildTools
  runs, modpack installs, and backups/restores.

## 10. Configuration

- Precedence: built-in defaults → `config.toml` in the data dir → `OBSIDIAN_*`
  env vars.
- Dev runs: `.cargo/config.toml` sets `OBSIDIAN_DATA_DIR` (via `[env]`, relative to the workspace root) to
  `target/dev-env`, so `cargo run` and `just dev` never touch an installed panel's data. An exported
  `OBSIDIAN_DATA_DIR` still wins.
- Env vars cover:
  - Port and bind address.
  - Data dir.
  - Public URL.
  - `OBSIDIAN_SMTP_*` (host, port, user, password, from, TLS).
  - Admin bootstrap, for headless VPS provisioning.
  - The CurseForge key, build time only.
- **First-run setup wizard** (styled like the login page):
  1. Create the admin account.
  2. Optional SMTP settings. These unlock password resets and emailed
     verification codes.
  3. Defaults: data paths, default Java, timezone.
  4. Default UPnP behaviour.

## 11. Design System — "Obsidian"

Obsidian uses the **Lodestone launcher's design language**
(https://lodestonelauncher.com, `Drew-Chase/lodestone-minecraft-launcher`). The
one change is that the **primary colour is nether-portal purple instead of
green**.

The source of truth is Lodestone's reference bundle,
`res/reference-images/Lodestone Launcher (standalone).html`, together with the
PNG mockups in the same folder:

- `01-login.png`
- `02-library.png`
- `04-instance.png`
- `instance-*.png`
- `settings-*.png`

Port those designs to HeroUI v3 tokens and Tailwind v4 in `src/css/index.css`.

### Tokens

- **Theme:** dark only.
- **Surfaces** (Lodestone's own plum "Obsidian" surface set):
  - `--bg-0: #0A0609` (canvas)
  - `--bg-1: #110B12` (rail/titlebar)
  - `--bg-2: #190F18` (panels)
  - `--bg-3: #231623` (inputs/menus)
  - `--bg-4: #2F1E30` (hover/popovers)
  - `--line: rgba(255,255,255,.07)`
  - `--line-strong: rgba(255,255,255,.12)`
- **Ink:** white at 100 / 92 / 68 / 45 / 28 % (`--ink-0` … `--ink-4`).
- **Accent (portal purple):**
  - `--accent: #B24BFF`
  - `--accent-dim: #8A2BE2`
  - `--accent-deep: #3B0A66`
  - `--accent-glow: rgba(178,75,255,.35)`
  - `--accent-tint`: accent at 12 %
  - Text on accent: `#1A0629`
- **Secondary glow:** ember orange `#FF6B3A` (nether lava). It takes the role
  Lodestone's violet had as the second aurora colour.
- **Status colours:** green `#4ADE80` for success, running and online (this is
  the `--success` token), amber `#FFB545`, red `#FF5A7A`. Cyan is not a status
  colour; informational chips use the secondary tone.
- **HeroUI mapping:** wire these into the HeroUI v3 variables (`--accent`,
  `--accent-foreground`, `--background`, `--surface`, etc.) as un-layered
  `:root` overrides.
- **Typography:**
  - **Inter** for body and UI text.
  - **JetBrains Mono** for uppercase eyebrows, versions, chips, paths, and
    status lines.
  - A bold **serif display** face for login and hero headlines, matching the
    Lodestone mockup.
- **Radii:** 8 / 12 / 18 / 24 px. Buttons and inputs are 12, panels 18, the
  login card 20.
- **Icons:** Lucide, via Iconify `lucide:*`.

### Motifs

- Glass cards: a `bg-2` → `bg-1` gradient, a 1px hairline border, a deep
  shadow, and a soft accent glow in one corner.
- Buttons are flat: primary is solid accent with `--accent-hover` on hover
  and a slight press scale; ghost is transparent with a hairline border. No
  gradients, inset highlights, lifts, or glow shadows.
- Uppercase mono chips with pulse dots, e.g. `● RUNNING`, `FABRIC 0.16.9`,
  `MC 1.21.4`, `87 MODS`.
- Procedural pixel-art landscape banners for instances.
- A faint 64px grid with a radial mask on hero areas.
- Animations: `pulse-ring`, `shimmer` (progress), `aurora-float`,
  `tick-in`, `blink-cursor`. Respect `prefers-reduced-motion`.
- Scrollbars are custom and identical in every browser, drawn by
  OverlayScrollbars (never native `::-webkit-scrollbar` or `scrollbar-color`):
  - A 4px thumb on a transparent 14px track, slightly brighter than the
    background.
  - It animates to 6px and brightens over 0.15s **only while the pointer is
    over the track**. Hovering the content does nothing.
  - Keep at least 5px between the thumb and content. The scrollbar overlays
    content, so a scroll area flush against its edge needs right padding
    (`pr-4`).
- Smooth wheel scrolling uses Lenis on the main scroller and the Discover lists.

### Login (Drew's favourite — replicate Lodestone's, in purple)

- **Background:** a full-bleed blurred scene with:
  - A purple aurora blob top-left (600px, `blur(80px)`, opacity .55).
  - An ember-orange blob bottom-right (500px), both on an 18s `aurora-float`
    loop with the second blob delayed by -9s.
  - Drifting purple "portal particle" specks.
  - A 3px dot-dust overlay (`mix-blend-mode: overlay`).
  - A vignette.
- **Top-left:** a language picker chip.
- **Centre card:** glass, 20px radius, with an accent glow in the top-right
  corner. Contents:
  - The Obsidian logo tile with a purple halo, next to the "Obsidian" wordmark
    over the mono line `SERVER PANEL · v{version}`.
  - The headline "Welcome back, **{word}**", with the last word in accent
    purple, then a muted subtitle.
  - Username and password fields, then a full-width primary "Sign in" button.
  - TOTP and verification-code steps that swap in inside the same card.
  - A "Forgot password?" link, shown only when SMTP is configured.
  - Footer: "Not affiliated with Mojang or Microsoft".
- **Bottom status bar** (mono):
  - Left: `● PANEL ONLINE · {n} INSTANCES RUNNING`, with a pulsing ring.
  - Right: version/build.

### App shell

- A 68px icon rail. From top to bottom:
  - Logo tile.
  - Dashboard, Instances, Discover (modpacks/mods/plugins), Tasks/Downloads.
  - Users (admin only).
  - Settings.
  - Account avatar.

  The active item has an accent-tint fill and an accent icon.
- **Page header:** title and muted subtitle on the left; a 240px search field,
  ghost icon buttons, and a primary CTA on the right.
- **Instance page:** modelled on Lodestone's `04-instance.png`:
  - Banner art with back and actions buttons.
  - The server icon.
  - Status, loader, version, and mod-count chips.
  - The large instance name and a mono line `path · size · last backup`.
  - Start / Stop / Restart controls.
  - Underline tabs: Console, Content, Files, Backups, Schedules, Players,
    Properties, Settings.

### Frontend organisation

- `src/pages/<area>/…` holds pages, and `src/components/{ui,layout,instance,files,…}`
  holds components.
- Server state goes through **TanStack Query**. A single `WebSocketProvider`
  handles subscriptions. No god-providers.
- Use HeroUI v3 compound components and `onPress` directly.

## 12. Competitive Landscape (research notes)

- **Panels studied:** Pterodactyl/Pelican, Crafty Controller, MCSManager, AMP,
  PufferPanel, Multicraft, and the hosts Aternos, Exaroton, Apex, Shockbyte,
  BisectHosting, PebbleHost.
- **Table stakes:** console, file manager, power controls, resource graphs,
  backups, schedules, subusers, version switching, `server.properties` editor,
  TOTP, SFTP, an API.
- **Where Obsidian should beat them:**
  - Modpack installs that reliably work, with clear errors.
  - Deduplicated incremental backups with single-file restore.
  - A human-friendly scheduler.
  - Crash-log hints, e.g. "mod X requires Y".
  - Preset sharing roles.
  - A one-binary install that idles light. Pterodactyl needs 10–15 setup steps
    and 1.5–2 GB RAM.
- **Post-v1 ideas:**
  - Auto-start on player join (a lightweight lobby listener).
  - Auto-stop when the server is empty.
  - A Discord bot.
  - S3 / cloud backup targets.
  - mclo.gs-style log sharing.
  - A provisioning/billing API for the hosting company.

## 13. Milestones

1. **Foundation:** workspace crates, `obsidian-db` (memdb + 30s flusher +
   migrations), config, logging, the WebSocket hub, design tokens, the app
   shell.
2. **Auth:** setup wizard, login page, sessions, TOTP, trusted IPs, invites,
   SMTP, password reset, CLI reset.
3. **Runtime:** `obsidian-java`, `obsidian-loaders`, `obsidian-runner`,
   instance CRUD, console, power controls, stats.
4. **Files:** file browser, chunked uploads, SFTP, server icon cropper.
5. **Content:** mods/plugins/modpacks from Modrinth, CurseForge, Hangar, Spiget.
6. **Backups + Scheduler.**
7. **Sharing & ops:** roles/permissions UI, UPnP, OpenAPI docs, packaging
   (Docker image, tray binary, service installers).

## 14. CI, Releases & Updates (GitHub Actions)

- **`.github/workflows/ci.yml`** (push / PR): `cargo clippy --workspace -- -D warnings`,
  `cargo test --workspace`, `pnpm tsc --noEmit`, and the frontend build, on
  Linux, Windows, and macOS.
- **`.github/workflows/release.yml`** (on `v*` tags) builds and publishes:
  - **Windows:** x86_64 `.msi` installer (`cargo-wix`).
  - **macOS:** universal or per-arch `.dmg` (`cargo-bundle` + `create-dmg`).
  - **Linux:** x86_64 and aarch64 `.tar.gz` binaries plus an `install.sh`
    installer script (downloads the latest release, installs the binary and an
    optional systemd unit).
  - **Docker:** multi-arch (amd64/arm64) image pushed to
    `ghcr.io/<owner>/obsidian-server-panel` tagged with the version and
    `latest`.
  - A **GitHub Release** for the tag with every artifact and checksums
    attached.
- **Automatic updates:** desktop/standalone builds (Windows, macOS, Linux)
  check GitHub Releases with the `self_update` crate and can update in place.
  Because servers run under detached runners (§5), updating the panel never
  stops running Minecraft servers. **Docker builds never self-update**; they
  are updated by pulling a new image (compile-time `docker` feature disables
  the updater).
