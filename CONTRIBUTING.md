# Contributing to Obsidian Server Panel

Thanks for your interest in improving Obsidian Server Panel. This document covers everything you need to build, run, test, and contribute to the project.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Repository Layout](#repository-layout)
- [Building](#building)
- [Running in Development](#running-in-development)
- [Database Backends](#database-backends)
- [Environment Variables](#environment-variables)
- [Testing](#testing)
- [Code Style](#code-style)
- [Commit & Pull Request Guidelines](#commit--pull-request-guidelines)
- [Packaging & Releases](#packaging--releases)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| [Rust toolchain](https://www.rust-lang.org/tools/install) | 1.92.0+ | Edition 2024 |
| [Node.js](https://nodejs.org/) | 20+ | Required by Vite 7 |
| [pnpm](https://pnpm.io/) | 10+ | `npm install -g pnpm` |
| [just](https://github.com/casey/just) | any | Command runner used by all build recipes |

Optional, depending on what you're working on:

- **Docker** — for the MySQL dev database and building container images.
- **makensis** ([NSIS](https://nsis.sourceforge.io/)) — only needed to build the Windows installer locally.
- **GNU `screen` 4.06+** — Linux only; required at runtime for server console sessions.

## Repository Layout

```
src-actix/          Rust backend (Actix Web)
  authentication/   Users, sessions, permissions, email verification & 2FA
  server/           Server lifecycle, files, backups, mods, modpack installs
  platforms/        Third-party API proxies (Modrinth, CurseForge, ATLauncher, Technic, getbukkit)
  host_info/        System resource monitoring (SSE)
  java/             Java runtime management
src/                React frontend (TypeScript)
  components/heroui/  HeroUI v2-compat layer (see Code Style)
  pages/ providers/   Pages and React context providers
crates/             Internal workspace crates (app_db, backups, scheduler,
                    modrinth, curseforge, *_loader, minecraft_server, upnp, cache)
resources/sql/      Per-dialect schema files (sqlite / mysql / postgres)
installer/          NSIS installer script
.github/workflows/  Release and GHCR publish pipelines
```

Backend modules follow a consistent pattern: types in `*_data.rs`, database access in `*_db.rs`, HTTP handlers in `*_endpoint.rs`, registered via a `configure()` function in `src-actix/lib.rs`.

## Building

This project uses [`just`](https://github.com/casey/just) as its command runner. Run `just --list` to see every available recipe.

```bash
git clone https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel.git
cd obsidian-server-panel
pnpm install
```

Build the frontend and backend together, packaged into a distributable zip in `target/dist/`:

```bash
just build              # SQLite (default)
just build mysql        # MySQL
just build postgres     # PostgreSQL
just build-all          # All three backends
```

Or build pieces individually:

```bash
just build-frontend            # TypeScript check + Vite build → target/wwwroot
just build-backend             # Backend debug build (SQLite)
just build-backend mysql       # Backend debug build (MySQL)
just build-backend-release     # Backend release build
```

The frontend is compiled into `target/wwwroot` and embedded directly into the backend binary at compile time (`src-actix/build.rs`), so a release build produces a single self-contained executable.

## Running in Development

```bash
just run                # SQLite (default)
just run mysql          # MySQL
just run-release        # Release build
just watch              # Rebuild backend on file changes (requires cargo-watch)
```

Two things behave differently in debug builds:

- The backend automatically starts a **Vite dev server** with hot module replacement and proxies to it, so you do not need to run the frontend separately. (`just dev-frontend` runs Vite standalone if you want it.)
- The working directory is switched to **`./target/dev-env`**, so all databases, server instances, and metadata stay inside `target/` and out of your working tree.

The panel listens on **port 80** by default. Override it with `--port`:

```bash
cargo run -- --port 8080
```

| Flag | Default | Description |
|------|---------|-------------|
| `--port`, `-p` | `80` | Web panel port |
| `--forward-webpanel` | `false` | Enable UPnP port forwarding for the panel |

## Database Backends

The backend supports SQLite, MySQL, and PostgreSQL through the internal `obsidian_database` crate (`crates/app_db`). **Exactly one** backend feature may be enabled — the crate raises a `compile_error!` otherwise.

`sqlite` is the default feature, so selecting a different backend requires `--no-default-features`:

```bash
cargo build --no-default-features --features mysql
```

The `just` recipes handle this for you; pass the backend as the recipe argument instead.

Schema files live in `resources/sql/<dialect>/` and are executed at startup. When you add or change a table, update **all three** dialect files and keep types consistent — notably, integer keys are `BIGINT` on MySQL (sqlx refuses to decode `i64` from `UNSIGNED` columns), and avoid `RETURNING`, which MySQL 8 does not support.

To develop against a local MySQL instance:

```bash
just dev-mysql          # Start MySQL 8 in Docker (waits for healthcheck)
just run-mysql          # Run the panel against it
just dev-mysql-down     # Tear it down
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_CONNECTION_STRING` | `app.db` (SQLite) | Connection string for the selected backend |
| `RUST_LOG` | `debug` in debug builds, `info` in release | Log level filter |
| `SMTP_HOST` | *(unset)* | **Master switch.** Unset or empty disables email verification and 2FA entirely |
| `SMTP_PORT` | `587` | SMTP port |
| `SMTP_USERNAME` | *(unset)* | Credentials are only applied if both username and password are set |
| `SMTP_PASSWORD` | *(unset)* | |
| `SMTP_FROM` | falls back to `SMTP_USERNAME` | Sender mailbox; required when `SMTP_HOST` is set |
| `SMTP_TLS` | inferred from port (`465` → `implicit`, else `starttls`) | One of `none`, `starttls`, `implicit` |

When `SMTP_HOST` is unset, signup, login, and password flows behave exactly as they did before the feature existed, and the verification UI is hidden. When it is set, email verification and 2FA are enforced across signup, login, and forgot-password.

## Testing

```bash
just test               # Tests for one backend (SQLite default)
just test mysql
just test-all           # All three backends
just check-frontend     # TypeScript type check
just check-all          # Type check + clippy across all backends
just clippy             # Clippy for one backend
```

Some tests hit live third-party APIs and are marked `#[ignore]` so they don't run in normal CI. Run them explicitly when changing platform integrations:

```bash
cargo test --features sqlite -- --ignored
```

All tests and scripts must be written in **Rust or TypeScript**. Do not add Python, Bash, or PowerShell script files — inline commands in `justfile` recipes and CI `run:` steps are the accepted exception.

See [TESTING.md](TESTING.md) for module-specific test documentation.

## Code Style

**Rust**

- Formatting is governed by `rustfmt.toml` — run `cargo fmt` before committing.
- Use `anyhow::Result` for application logic and `thiserror` for custom error types.
- Everything async runs on Tokio. Never block the async runtime: wrap CPU-bound work (bcrypt, large sync filesystem scans) in `tokio::task::spawn_blocking`.
- Prefer bounded concurrency (`buffer_unordered`) over sequential loops for network I/O.

**Frontend**

- Follow [style-guide.md](style-guide.md) for UX consistency.
- The UI targets **HeroUI v3**, but application code imports from the compatibility layer at `src/components/heroui/` (aliased as `@heroui-compat`), which presents the flatter v2-style API on top of v3's compound components. Import from `@heroui-compat`, not `@heroui/react`, unless you are working inside the compat layer itself.
- **No rounded corners anywhere.** Radius is globally zeroed in the theme; pass `radius="none"` where a component still accepts it.
- Typography: `font-minecraft-header` for headings, `font-minecraft-body` for UI text.
- Icons: pixel art style (`pixelarticons:*`, `pixel:*`).
- Never hardcode colors — use theme variables, and verify changes against all four accessibility themes (default dark, deuteranopia, tritanopia, monochrome).

**Comments**

Keep them minimal. Write a comment only to explain a constraint the code cannot express on its own — not to narrate what the next line does or to justify a change.

## Commit & Pull Request Guidelines

- Use [Conventional Commits](https://www.conventionalcommits.org/): `feat(server): …`, `fix(auth): …`, `perf(modpack): …`, `refactor(frontend): …`, `docs: …`, `ci: …`, `chore: …`.
- Keep commits focused; one logical change per commit.
- Do not add attribution trailers or generated-by notices to commit messages.
- Before opening a PR, make sure these pass:
  ```bash
  just check-all
  just test
  ```
- Update `openapi-v3.json` when you add or change an HTTP endpoint.
- If your change affects the database schema, update all three dialect files under `resources/sql/`.

## Packaging & Releases

Releases are produced by `.github/workflows/release.yml`, triggered by pushing a `v*` tag. It builds a matrix of Windows, macOS, and Linux binaries for the SQLite and MySQL backends, builds the Windows installer, and attaches everything to a GitHub Release.

To produce artifacts locally:

```bash
just build sqlite               # → target/dist/obsidian-server-<os>-<arch>-sqlite.zip
just installer 1.2.3            # → NSIS installer (Windows, requires makensis)
just docker-build mysql         # → obsidian-server-panel:mysql container image
```

The container image is published to GHCR by `.github/workflows/publish-ghcr.yml`. It is Alpine-based and includes `screen` so Linux console sessions work inside the container.

## Troubleshooting

**`pnpm exec` fails with `ERR_PNPM_IGNORED_BUILDS` or re-runs `pnpm install`**

Newer pnpm versions run a dependency-status check that can fail on some machines. Invoke the tools directly instead:

```bash
node node_modules/typescript/lib/tsc.js --noEmit
node node_modules/vite/bin/vite.js build
```

If the lockfile check blocks installation, use `pnpm install --no-frozen-lockfile`.

**Phantom `E0599: no variant ... found` errors after switching branches**

Stale incremental build artifacts. Clear the affected crate and rebuild:

```bash
cargo clean -p <crate-name>
```

**`compile_error!: Cannot enable both 'sqlite' and 'mysql' features`**

You enabled a second backend on top of the default. Add `--no-default-features`, or use the `just` recipes which do it for you.

**Server console is empty on Linux**

Console sessions require GNU `screen` 4.06 or newer (for `-Logfile` support). Without it the panel falls back to direct process pipes and logs a warning. Sessions are named `obsidian-<server-id>` and can be attached manually with `screen -r`.
