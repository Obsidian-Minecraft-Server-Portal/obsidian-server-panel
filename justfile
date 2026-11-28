[windows]
set shell := ["powershell.exe", "-NoProfile", "-NoLogo", "-Command"]
[linux]
set shell := ["bash", "-c"]
[macos]
set shell := ["zsh", "-c"]

set dotenv-load := true

data_dir := env("OBSIDIAN_DATA_DIR", justfile_directory() / "target" / "dev-env")
image := "obsidian-server-panel:dev"

[private]
default:
    @just --list --unsorted

# Install frontend dependencies
[group('setup')]
install:
    pnpm install --frozen-lockfile

# Run the panel in debug mode; Actix proxies the Vite dev server
[group('dev')]
dev: install
    cargo run --bin obsidian -- serve --headless

[group('dev')]
dev-watch: install
    cargo watch -x "run --bin obsidian -- serve --headless" --why

# Run the panel with the tray icon
[group('dev')]
dev-tray: install
    cargo run --bin obsidian -- serve

# Run only the Vite dev server
[group('dev')]
frontend: install
    pnpm vite

# Reset a user's password against the dev data dir (panel must be stopped)
[group('dev')]
reset-password username:
    cargo run --bin obsidian -- reset-password {{ username }}

# Delete the dev data dir (database, instances, backups)
[group('dev')]
[confirm("Delete the dev data dir, including every instance and backup in it?")]
clean-data:
    rm -rf "{{ data_dir }}"

# Fast compile check of every crate
[group('check')]
check:
    cargo check --workspace --all-targets

# Clippy on every feature set CI builds, with warnings as errors
[group('check')]
clippy:
    cargo clippy --workspace --all-targets -- -D warnings
    cargo clippy -p obsidian --all-targets --no-default-features --features docker -- -D warnings
    cargo clippy -p obsidian --all-targets --no-default-features --features updater,service -- -D warnings

# Type-check the frontend
[group('check')]
tsc: install
    pnpm tsc --noEmit

# Fail if any source file is longer than 750 lines
[group('check')]
[unix]
@lines:
    #!/usr/bin/env bash
    set -euo pipefail
    long=$(git ls-files '*.rs' '*.ts' '*.tsx' '*.css' '*.sql' '*.yml' '*.sh' | xargs wc -l | awk '$2 != "total" && $1 > 750')
    if [ -n "$long" ]; then echo "Files over 750 lines:"; echo "$long"; exit 1; fi
    echo "All source files are within 750 lines."

[group("check")]
[windows]
@lines:
    $long = git ls-files '*.rs' '*.ts' '*.tsx' '*.css' '*.sql' '*.yml' '*.sh' | ForEach-Object { $n = (Get-Content -LiteralPath $_ | Measure-Object).Count; if ($n -gt 750) { '{0} {1}' -f $n, $_ } }; if ($long) { Write-Host 'Files over 750 lines:'; $long; exit 1 }; Write-Host 'All source files are within 750 lines.'

# Run every Rust test, or only those matching a filter
[group('check')]
test *filter:
    cargo test --workspace {{ filter }}

# Everything CI runs: clippy, tests, tsc, frontend build, line limit
[group('check')]
ci: clippy test tsc build-frontend lines

# Build the frontend into target/wwwroot
[group('build')]
build-frontend: install
    pnpm tsc --noEmit
    pnpm vite build

# Release build of the desktop binary (tray, service, updater)
[group('build')]
build: build-frontend
    cargo build --release --bin obsidian

# Release build with the Docker feature set (no tray, no self-update)
[group('build')]
build-headless: build-frontend
    cargo build --release --bin obsidian --no-default-features --features docker

# Run the release binary against the dev data dir
[group('build')]
run-release: build
    OBSIDIAN_DATA_DIR="{{ data_dir }}" ./target/release/obsidian serve --headless

# Build the Docker image
[group('docker')]
docker-build:
    docker build -t {{ image }} .

# Run the Docker image on the panel and SFTP ports with a named volume
[group('docker')]
docker-run: docker-build
    docker run --rm -it -p 1421:1421 -p 2022:2022 -v obsidian-dev-data:/data {{ image }}

# Stamp a version into Cargo.toml and Cargo.lock, commit it and tag it (push the tag yourself to release)
[group('release')]
[unix]
tag version:
    ./packaging/stamp-version.sh v{{ trim_start_match(version, "v") }}
    cargo check --workspace --quiet
    git add Cargo.toml Cargo.lock
    git commit -m "Release v{{ trim_start_match(version, "v") }}"
    git tag -a v{{ trim_start_match(version, "v") }} -m "v{{ trim_start_match(version, "v") }}"
    @echo "Tagged v{{ trim_start_match(version, "v") }}. Push with: git push --follow-tags"

[group('release')]
publish version="":
    uv -p ./Cargo.toml -t toml -g commit-push-tag {{version}}