# Testing Documentation

## Backend Tests

### Server Update Module Tests

All tests are located in the `src-actix/server/updates/` module.

#### Running Tests

```bash
# Run all update module tests
cargo test --lib server::updates

# Run specific test file
cargo test --lib server::updates::update_data
cargo test --lib server::updates::update_checker
cargo test --lib server::updates::update_service

# Run with output
cargo test --lib server::updates -- --nocapture
```

#### Test Coverage

**update_data.rs** (7 tests):
- ✅ `test_update_info_new_with_update` - Tests creating UpdateInfo with different versions
- ✅ `test_update_info_new_same_version` - Tests that same version results in no update
- ✅ `test_update_info_no_update` - Tests no_update constructor
- ✅ `test_update_info_without_changelog` - Tests UpdateInfo without changelog URL
- ✅ `test_update_info_serialization` - Tests JSON serialization/deserialization
- ✅ `test_update_info_version_comparison` - Tests version string comparison logic

**update_checker.rs** (8 tests):
- ✅ `test_vanilla_changelog_url_format` - Validates Vanilla changelog URL formatting
- ✅ `test_fabric_download_url_format` - Validates Fabric download URL construction
- ✅ `test_forge_download_url_format` - Validates Forge download URL construction
- ✅ `test_forge_version_key_format` - Tests Forge version key generation
- ✅ `test_neoforge_download_url_format` - Validates NeoForge download URL construction
- ✅ `test_quilt_download_url_format` - Validates Quilt download URL construction
- ✅ `test_version_comparison` - Tests version string equality
- ✅ `test_api_endpoint_urls` - Validates all API endpoint URLs are correct
- ✅ `test_changelog_urls` - Validates changelog URLs for all loaders

**update_service.rs** (4 tests):
- ✅ `test_server_idle_check` - Validates test server creation in Idle state
- ✅ `test_backup_filename_generation` - Tests backup JAR naming convention
- ✅ `test_server_type_database_update_logic` - Tests server type differentiation
- ✅ `test_temp_jar_naming` - Tests temporary JAR file naming

#### Test Results

```
running 19 tests
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured
```

### HTTP API Integration Tests

`tests/api.rs` (+ `tests/common/`) boots the real server binary (sqlite build)
on port 18101 and exercises every endpoint group over real HTTP: auth and
permissions, users, servers CRUD, lifecycle (via a fake `java` script), the
filesystem scope (upload/download/archive/extract with SSE trackers), mods,
backups, updates, settings, platforms and SSE/WebSocket surfaces. The suite is
re-run safe and network-tolerant (upstream-dependent routes degrade to
"route must answer" assertions when offline).

```bash
cargo test --no-default-features --features sqlite
```

The suite only compiles under the `sqlite` feature (the app builds exactly one
database driver per feature flag); cross-engine coverage is provided by the
database engine suite below.

#### Real Minecraft Lifecycle Test (opt-in)

The fast lifecycle tests use a driver stub so the suite stays quick and
offline-tolerant — they prove the panel's control plumbing, not Minecraft
itself. `just test-real` runs the real thing: the panel installs the actual
gamma JDK through its own java manager, downloads the real vanilla server jar
from Mojang, boots the real JVM, waits for `Done (`, then stops it gracefully
(first run downloads ~150 MB; later runs reuse the cached JDK, ~40s).

```bash
just test-real
```

### Database Engine Tests

The backend compiles for three database engines via cargo features (`sqlite`,
`mysql`, `postgres`; exactly one at a time). The integration suite in
`tests/database_engines.rs` boots the real server binary against the selected
engine and validates the full CRUD surface (auth, servers, filesystem,
backups, users, settings) plus an app-restart persistence scenario.

### Running

```bash
# MySQL (ephemeral Docker DB on port 3307; scratch DB recreated each run)
just test-mysql

# PostgreSQL (ephemeral Docker DB on port 5433; scratch DB recreated each run)
just test-postgres

# All three engines sequentially (sqlite + mysql + postgres)
just test-databases

# Bring the test databases up/down manually
just dev-mysql-test / just dev-mysql-test-down
just dev-postgres / just dev-postgres-down
```

### Configuration

The suite is controlled by environment variables (set automatically by the
justfile recipes):

| Variable | Purpose |
| --- | --- |
| `OBSIDIAN_TEST_DB_ENGINE` | `mysql`, `postgres` or `sqlite`; tests skip early when unset |
| `OBSIDIAN_TEST_DB_URL` | Connection string for the scratch database |
| `OBSIDIAN_TEST_DB_ADMIN_URL` | Admin connection (DB lifecycle is managed by the justfile) |

Compose files: `.docker/mysql-test.compose.yml` and
`.docker/postgres.compose.yml`. Engine-specific schema lives in
`resources/sql/{sqlite,mysql,postgres}/`.

## UI Pathway Tests (Puppeteer)

`tools/ui-tests/` drives the real app (backend + Vite) with Puppeteer and
screenshots every pathway at a mobile viewport of **360x760** (override with
`UI_VIEWPORT=1920x1080`). Every page load is audited for horizontal overflow.

```bash
# Starts/reuses the backend on port 18104 (Vite on 3174) and runs every spec
./node_modules/.bin/tsx tools/ui-tests/run-all.ts

# Single spec
./node_modules/.bin/tsx tools/ui-tests/auth.spec.ts
```

Specs: auth, dashboard (+ all 4 accessibility themes), server-create (all
loaders), server-lifecycle (fake java start/stop/restart/kill), server-tabs,
server-files (upload/Monaco edit/download/rename/archive/drag-move), server-options,
server-backups, discover, account, errors. Results aggregate to
`target/ui-tests/report.json` with screenshots under `target/ui-tests/`.

The suite runs the backend against an isolated dev-env
(`target/ui-tests/dev-env-root/target/dev-env`), so manual testing data and UI
test fixtures never interfere; the fake java runtimes are seeded automatically.

## Frontend Tests

Frontend tests use the standard testing tools configured in the project.

### Running Frontend Tests

```bash
# Run all frontend tests
pnpm test

# Run with coverage
pnpm test:coverage
```

### Test Files

- `src/ts/api/server-update-api.test.ts` - API client tests (to be implemented)
- `src/hooks/useServerUpdates.test.ts` - Hook tests (to be implemented)

## Manual Testing Checklist

### Backend API Endpoints

- [ ] `GET /api/server/:id/updates/check` - Check for updates
- [ ] `POST /api/server/:id/updates/apply` - Apply update
- [ ] `POST /api/server/:id/updates/rollback` - Rollback update
- [ ] `GET /api/server/:id/updates/backups` - List backup JARs

### Frontend UI

- [ ] "Check for Updates" button works
- [ ] Update badge shows correct version information
- [ ] "Apply Update" button is disabled when server is running
- [ ] "Apply Update" creates backup before updating
- [ ] "Rollback" restores previous version
- [ ] Toast notifications appear for all operations
- [ ] Changelog link opens correctly (Vanilla only)

### Update Flow for Each Loader Type

- [ ] Vanilla: Check → Apply → Verify new version
- [ ] Fabric: Check → Apply → Verify new version
- [ ] Forge: Check → Apply → Verify new version
- [ ] NeoForge: Check → Apply → Verify new version
- [ ] Quilt: Check → Apply → Verify new version

### Error Scenarios

- [ ] Attempting to update running server shows error
- [ ] Network failure during update check handled gracefully
- [ ] Download failure during update handled gracefully
- [ ] Rollback when no backup exists handled gracefully

## Test Data

### Example Update Responses

**Vanilla Update Available:**
```json
{
  "update_available": true,
  "current_version": "1.20.1",
  "latest_version": "1.20.2",
  "download_url": "https://piston-data.mojang.com/...",
  "changelog_url": "https://www.minecraft.net/en-us/article/minecraft-java-edition-1-20-2"
}
```

**No Update Available:**
```json
{
  "update_available": false,
  "message": "Server is up to date"
}
```

### Test Server Configurations

```rust
// Vanilla 1.20.1
ServerData {
    server_type: Some(ServerType::Vanilla),
    minecraft_version: Some("1.20.1"),
    loader_version: None,
    ...
}

// Fabric 1.20.1 with loader 0.15.0
ServerData {
    server_type: Some(ServerType::Fabric),
    minecraft_version: Some("1.20.1"),
    loader_version: Some("0.15.0"),
    ...
}
```

## CI/CD Integration

Tests should be run in CI/CD pipeline:

```yaml
# Example GitHub Actions workflow
- name: Run backend tests
  run: cargo test --lib

- name: Run frontend tests
  run: pnpm test
```
