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
    node node_modules/typescript/bin/tsc
    node node_modules/vite/bin/vite.js build

install:
    pnpm i

# Start the Vite dev server with HMR
dev-frontend:
    pnpm exec vite dev

# ─── Backend ───────────────────────────────────────────────

# Build backend for a single database feature (debug)
build-backend feature="sqlite":
    cargo build --no-default-features --features {{ feature }}

# Build backend for a single database feature (release)
build-backend-release feature="sqlite":
    cargo build --release --no-default-features --features {{ feature }}

# Run clippy for a single database feature
clippy feature="sqlite":
    cargo clippy --no-default-features --features {{ feature }}

# Run tests for a single database feature
test feature="sqlite":
    cargo test --no-default-features --features {{ feature }}

# ─── Full Build ────────────────────────────────────────────

# Build frontend + backend for a single database feature, package into zip
build feature="sqlite": build-frontend (_build-and-package feature) install

# Build and package release binaries for ALL database backends
build-all: build-frontend (_build-and-package "sqlite") (_build-and-package "mysql") (_build-and-package "postgres") install

# Internal: build release binary for a feature and package it into a zip
[private]
_build-and-package feature:
    cargo build --release --no-default-features --features {{ feature }}
    just _package {{ feature }}

# Internal: copy binary + wwwroot into a zip archive
[private]
[windows]
_package feature:
    if (!(Test-Path "{{ dist_dir }}")) { New-Item -ItemType Directory -Path "{{ dist_dir }}" | Out-Null }
    $staging = "{{ dist_dir }}/_staging"; \
    if (Test-Path $staging) { Remove-Item $staging -Recurse -Force }; \
    New-Item -ItemType Directory -Path $staging | Out-Null; \
    Copy-Item "target/release/{{ bin_name }}.exe" $staging; \
    $zip = "{{ dist_dir }}/obsidian-server-{{ os() }}-{{ arch() }}-{{ feature }}.zip"; \
    if (Test-Path $zip) { Remove-Item $zip -Force }; \
    Compress-Archive -Path "$staging/*" -DestinationPath $zip; \
    Remove-Item $staging -Recurse -Force; \
    Write-Host "Packaged: $zip"

[private]
[unix]
_package feature:
    mkdir -p "{{ dist_dir }}/_staging"
    cp "target/release/{{ bin_name }}" "{{ dist_dir }}/_staging/"
    cd "{{ dist_dir }}/_staging" && zip -r "../obsidian-server-{{ os() }}-{{ arch() }}-{{ feature }}.zip" .
    rm -rf "{{ dist_dir }}/_staging"
    @echo "Packaged: {{ dist_dir }}/obsidian-server-{{ os() }}-{{ arch() }}-{{ feature }}.zip"

# Build the NSIS installer from an existing release binary (requires makensis)
[windows]
installer version="0.0.0": (build "sqlite")
    makensis /DVERSION={{ version }} installer/installer.nsi

# ─── Development ───────────────────────────────────────────

# Run the backend in debug mode (sqlite)
run feature="sqlite": install
    cargo run --no-default-features --features {{ feature }}

# Run the backend in release mode (sqlite)
run-release feature="sqlite": install
    cargo run --release --no-default-features --features {{ feature }}

# Watch for backend changes and rebuild (sqlite)
watch feature="sqlite": install
    cargo watch -x "run --no-default-features --features {{ feature }}"

# Start the local MySQL dev database (Docker)
dev-mysql:
    docker compose -f .docker/mysql.compose.yml up -d --wait

# Stop the local MySQL dev database
dev-mysql-down:
    docker compose -f .docker/mysql.compose.yml down

# Run the backend against the local MySQL dev database
run-mysql $DATABASE_CONNECTION_STRING="mysql://root:root@localhost:3306/obsidian-server-panel": dev-mysql install
    cargo run --no-default-features --features mysql

# Start the ephemeral MySQL test database (Docker, host port 3307)
dev-mysql-test:
    docker compose -f .docker/mysql-test.compose.yml up -d --wait

# Stop and remove the ephemeral MySQL test database
dev-mysql-test-down:
    docker compose -f .docker/mysql-test.compose.yml down -v

# Start the ephemeral PostgreSQL test database (Docker, host port 5433)
dev-postgres:
    docker compose -f .docker/postgres.compose.yml up -d --wait

# Stop and remove the ephemeral PostgreSQL test database
dev-postgres-down:
    docker compose -f .docker/postgres.compose.yml down -v

# Run the backend against the ephemeral PostgreSQL test database
run-postgres $DATABASE_CONNECTION_STRING="postgres://postgres:postgres@localhost:5433/obsidian-server-panel": dev-postgres install
    cargo run --no-default-features --features postgres

# Run the integration test suite against the ephemeral MySQL test database
test-mysql: dev-mysql-test install
    docker compose -f .docker/mysql-test.compose.yml exec -T db mysql -uroot -proot -e "DROP DATABASE IF EXISTS obsidian_test; CREATE DATABASE obsidian_test;"
    env DATABASE_CONNECTION_STRING=mysql://root:root@localhost:3307/obsidian_test OBSIDIAN_TEST_DB_ENGINE=mysql OBSIDIAN_TEST_DB_URL=mysql://root:root@localhost:3307/obsidian_test OBSIDIAN_TEST_DB_ADMIN_URL=mysql://root:root@localhost:3307 cargo test --no-default-features --features mysql

# Run the integration test suite against the ephemeral PostgreSQL test database
test-postgres: dev-postgres install
    docker compose -f .docker/postgres.compose.yml exec -T db psql -U postgres -c "DROP DATABASE IF EXISTS obsidian_test;" -c "CREATE DATABASE obsidian_test;"
    env DATABASE_CONNECTION_STRING=postgres://postgres:postgres@localhost:5433/obsidian_test OBSIDIAN_TEST_DB_ENGINE=postgres OBSIDIAN_TEST_DB_URL=postgres://postgres:postgres@localhost:5433/obsidian_test OBSIDIAN_TEST_DB_ADMIN_URL=postgres://postgres:postgres@localhost:5433/postgres cargo test --no-default-features --features postgres

# Run the integration test suite against every database engine (sqlite, mysql, postgres)
test-databases: test-mysql test-postgres

# ─── Quality ───────────────────────────────────────────────

# Run clippy across all database backends
clippy-all:
    cargo clippy --no-default-features --features sqlite
    cargo clippy --no-default-features --features mysql
    cargo clippy --no-default-features --features postgres

# Run tests across all database backends
test-all:
    cargo test --no-default-features --features sqlite
    cargo test --no-default-features --features mysql
    cargo test --no-default-features --features postgres

# Check TypeScript types
check-frontend:
    pnpm exec tsc --noEmit

# Run all checks (frontend types + clippy for all backends)
check-all: check-frontend clippy-all

# ─── Docker ───────────────────────────────────────────────

# Build and start the Docker Compose stack (MySQL + Obsidian)
compose:
    docker compose -f composer.yml up --build

# Build and start in detached mode
compose-detach:
    docker compose -f composer.yml up --build -d

# Stop the Docker Compose stack
compose-down:
    docker compose -f composer.yml down

# Stop and remove all volumes (destroys data)
compose-purge:
    docker compose -f composer.yml down -v

# View logs from all containers
compose-logs:
    docker compose -f composer.yml logs -f

# View logs from the obsidian container only
compose-logs-obsidian:
    docker compose -f composer.yml logs -f obsidian

# Open a shell in the running obsidian container
compose-shell:
    docker compose -f composer.yml exec obsidian /bin/sh

# Open the MySQL CLI in the running MySQL container
compose-mysql:
    docker compose -f composer.yml exec mysql mysql -u obsidian -pobsidian_pass obsidian

# Build the Docker image for a specific database backend
docker-build feature="mysql":
    docker build --build-arg DB_FEATURE={{ feature }} -t obsidian-server-panel:{{ feature }} .

# ─── Cleanup ───────────────────────────────────────────────

# Remove all build artifacts
clean:
    cargo clean