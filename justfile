# Obsidian Server Panel - Build Recipes
# Usage: just <recipe> or just --list

set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

bin_name := "obsidian_server_panel"
dist_dir := "target/dist"

# Default recipe: list all available recipes
default:
    @just --list

# ─── Frontend ──────────────────────────────────────────────

# Build the frontend (TypeScript check + Vite build → target/wwwroot)
build-frontend:
    pnpm exec tsc
    pnpm exec vite build

# Start the Vite dev server with HMR
dev-frontend:
    pnpm exec vite dev

# ─── Backend ───────────────────────────────────────────────

# Build backend for a single database feature (debug)
build-backend feature="sqlite":
    cargo build --features {{feature}}

# Build backend for a single database feature (release)
build-backend-release feature="sqlite":
    cargo build --release --features {{feature}}

# Run clippy for a single database feature
clippy feature="sqlite":
    cargo clippy --features {{feature}}

# Run tests for a single database feature
test feature="sqlite":
    cargo test --features {{feature}}

# ─── Full Build ────────────────────────────────────────────

# Build frontend + backend for a single database feature, package into zip
build feature="sqlite": build-frontend (_build-and-package feature)

# Build and package release binaries for ALL database backends
build-all: build-frontend (_build-and-package "sqlite") (_build-and-package "mysql") (_build-and-package "postgres")

# Internal: build release binary for a feature and package it into a zip
[private]
_build-and-package feature:
    cargo build --release --features {{feature}}
    just _package {{feature}}

# Internal: copy binary + wwwroot into a zip archive
[private]
[windows]
_package feature:
    if (!(Test-Path "{{dist_dir}}")) { New-Item -ItemType Directory -Path "{{dist_dir}}" | Out-Null }
    $staging = "{{dist_dir}}/_staging"; \
    if (Test-Path $staging) { Remove-Item $staging -Recurse -Force }; \
    New-Item -ItemType Directory -Path $staging | Out-Null; \
    Copy-Item "target/release/{{bin_name}}.exe" $staging; \
    if (Test-Path "target/wwwroot") { Copy-Item "target/wwwroot" $staging -Recurse }; \
    $zip = "{{dist_dir}}/obsidian-server-{{os()}}-{{arch()}}-{{feature}}.zip"; \
    if (Test-Path $zip) { Remove-Item $zip -Force }; \
    Compress-Archive -Path "$staging/*" -DestinationPath $zip; \
    Remove-Item $staging -Recurse -Force; \
    Write-Host "Packaged: $zip"

[private]
[unix]
_package feature:
    mkdir -p "{{dist_dir}}/_staging"
    cp "target/release/{{bin_name}}" "{{dist_dir}}/_staging/"
    if [ -d "target/wwwroot" ]; then cp -r "target/wwwroot" "{{dist_dir}}/_staging/"; fi
    cd "{{dist_dir}}/_staging" && zip -r "../obsidian-server-{{os()}}-{{arch()}}-{{feature}}.zip" .
    rm -rf "{{dist_dir}}/_staging"
    @echo "Packaged: {{dist_dir}}/obsidian-server-{{os()}}-{{arch()}}-{{feature}}.zip"

# ─── Development ───────────────────────────────────────────

# Run the backend in debug mode (sqlite)
run feature="sqlite":
    cargo run --features {{feature}}

# Run the backend in release mode (sqlite)
run-release feature="sqlite":
    cargo run --release --features {{feature}}

# Watch for backend changes and rebuild (sqlite)
watch feature="sqlite":
    cargo watch -x "run --features {{feature}}"

# ─── Quality ───────────────────────────────────────────────

# Run clippy across all database backends
clippy-all:
    cargo clippy --features sqlite
    cargo clippy --features mysql
    cargo clippy --features postgres

# Run tests across all database backends
test-all:
    cargo test --features sqlite
    cargo test --features mysql
    cargo test --features postgres

# Check TypeScript types
check-frontend:
    pnpm exec tsc --noEmit

# Run all checks (frontend types + clippy for all backends)
check-all: check-frontend clippy-all

# ─── Cleanup ───────────────────────────────────────────────

# Remove all build artifacts
clean:
    cargo clean
