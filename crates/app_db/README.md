# obsidian_database

A generic, feature-gated database wrapper around [sqlx](https://crates.io/crates/sqlx) providing a unified interface for SQLite, MySQL, and PostgreSQL. Originally built for the Obsidian Minecraft Server Panel, but designed as a standalone crate that works in any Rust project needing a simple database abstraction.

Enable exactly **one** of the `sqlite`, `mysql`, or `postgres` features. The crate uses conditional compilation to expose a single set of types (`Pool`, `Row`, `QueryResult`) that resolve to the appropriate sqlx backend.

## Features

- **Unified interface** -- Write database code once and swap backends by changing a feature flag
- **Builder-pattern connection** -- Configure pool size, log level, and backend-specific options via `DatabaseBuilder`
- **Cross-database SQL** -- The `sql()` helper converts `?` placeholders to `$1, $2, ...` for PostgreSQL while passing through unchanged for SQLite and MySQL
- **Transaction wrapper** -- `Transaction` struct with `Deref`/`DerefMut` to the inner sqlx transaction, auto-rollback on drop
- **Schema helpers** -- `execute_schema()` and `execute_schemas()` for running DDL statements
- **sqlx re-exports** -- Consumers can use `query`, `query_as`, `query_scalar`, `FromRow`, `Executor`, and `Row` (as `RowTrait`) without adding sqlx as a direct dependency
- **Compile-time guards** -- Fails to compile if zero or multiple backend features are enabled simultaneously

## Requirements

- **Rust Edition 2024** -- Rust 1.85.0 or later
- **Tokio Runtime** -- All async operations require a [tokio](https://tokio.rs/) runtime

## Installation

Add to your `Cargo.toml` with exactly one backend feature:

```toml
[dependencies]
obsidian_database = { path = "../app_db", features = ["sqlite"] }
```

Or from a Git repository:

```toml
[dependencies]
obsidian_database = { git = "https://github.com/drew-chase/obsidian-server-panel.git", path = "crates/app_db", features = ["sqlite"] }
```

### Available Features

| Feature    | Description                      | Backend          |
|------------|----------------------------------|------------------|
| `sqlite`   | Use SQLite via sqlx              | `sqlx::Sqlite`   |
| `mysql`    | Use MySQL via sqlx               | `sqlx::MySql`    |
| `postgres` | Use PostgreSQL via sqlx          | `sqlx::Postgres` |

Enabling more than one feature at a time is a compile error.

## Quick Start

```rust
use obsidian_database::{Database, execute_schema, query, query_as, sql, FromRow};

#[derive(Debug, FromRow)]
struct User {
    id: i64,
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Connect
    let db = Database::builder()
        .connection_string("app.db")
        .max_connections(5)
        .build()
        .await?;

    let pool = db.pool();

    // 2. Create schema
    execute_schema(pool, r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL
        );
    "#).await?;

    // 3. Insert (sql() converts ? to $N for PostgreSQL)
    query(&*sql("INSERT INTO users (name) VALUES (?)"))
        .bind("Alice")
        .execute(pool)
        .await?;

    // 4. Query
    let users: Vec<User> = query_as(&*sql("SELECT id, name FROM users"))
        .fetch_all(pool)
        .await?;

    for user in &users {
        println!("{}: {}", user.id, user.name);
    }

    Ok(())
}
```

## Usage

### Creating a Database Connection

Use `Database::builder()` for a fluent configuration API:

```rust
use obsidian_database::Database;

let db = Database::builder()
    .connection_string("app.db")   // SQLite: file path; MySQL/PostgreSQL: URI
    .max_connections(10)           // Optional pool size limit
    .log_level(log::LevelFilter::Debug) // SQL statement log level (default: Trace)
    .build()
    .await?;

let pool = db.pool(); // &Pool -- borrow for queries
```

#### SQLite-Specific Options

When the `sqlite` feature is enabled, additional builder methods are available:

```rust
let db = Database::builder()
    .connection_string("app.db")
    .create_if_missing(true)  // Create DB file if absent (default: true)
    .wal_mode(true)           // Enable WAL journal mode (default: true)
    .foreign_keys(true)       // Enable foreign key constraints (default: true)
    .build()
    .await?;
```

#### MySQL / PostgreSQL

These backends require a connection string:

```rust
// MySQL
let db = Database::builder()
    .connection_string("mysql://user:pass@localhost/dbname")
    .build()
    .await?;

// PostgreSQL
let db = Database::builder()
    .connection_string("postgres://user:pass@localhost/dbname")
    .build()
    .await?;
```

### Cross-Database SQL with `sql()`

Write queries using `?` placeholders. The `sql()` function handles backend differences:

- **SQLite / MySQL** -- Returns the query unchanged (zero-allocation borrow)
- **PostgreSQL** -- Replaces `?` with `$1`, `$2`, etc.

```rust
use obsidian_database::{sql, query};

// Works on all backends:
query(&*sql("INSERT INTO users (name, email) VALUES (?, ?)"))
    .bind("Alice")
    .bind("alice@example.com")
    .execute(pool)
    .await?;
```

### Typed Queries with `FromRow`

Use `query_as` with `#[derive(FromRow)]` for typed results:

```rust
use obsidian_database::{query_as, sql, FromRow};

#[derive(Debug, FromRow)]
struct User {
    id: i64,
    name: String,
    email: String,
}

let users: Vec<User> = query_as(&*sql("SELECT id, name, email FROM users WHERE name = ?"))
    .bind("Alice")
    .fetch_all(pool)
    .await?;
```

### Scalar Queries

Use `query_scalar` for single-value results:

```rust
use obsidian_database::{query_scalar, sql};

let count: i64 = query_scalar("SELECT COUNT(*) FROM users")
    .fetch_one(pool)
    .await?;
```

### Transactions

The `Transaction` wrapper provides commit/rollback with auto-rollback on drop:

```rust
use obsidian_database::{Transaction, query, sql};

let mut tx = Transaction::begin(pool).await?;

query(&*sql("UPDATE accounts SET balance = balance - ? WHERE id = ?"))
    .bind(200i64)
    .bind(1i64)
    .execute(&mut **tx)  // double deref: Transaction -> sqlx::Transaction -> Connection
    .await?;

query(&*sql("UPDATE accounts SET balance = balance + ? WHERE id = ?"))
    .bind(200i64)
    .bind(2i64)
    .execute(&mut **tx)
    .await?;

tx.commit().await?; // If not called, the transaction rolls back on drop
```

### Schema Management

Run DDL statements with dedicated helpers:

```rust
use obsidian_database::{execute_schema, execute_schemas};

// Single statement
execute_schema(pool, "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);")
    .await?;

// Multiple statements (fails fast on first error)
execute_schemas(pool, &[
    "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
    "CREATE TABLE IF NOT EXISTS posts (id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users(id));",
]).await?;
```

### Owning the Pool

Use `into_pool()` when you need to move the pool into a global or framework state:

```rust
use std::sync::OnceLock;
use obsidian_database::{Database, Pool};

static DB_POOL: OnceLock<Pool> = OnceLock::new();

async fn init() {
    let db = Database::builder()
        .connection_string("app.db")
        .build()
        .await
        .expect("database connection failed");

    DB_POOL.set(db.into_pool()).ok();
}
```

## API Reference

### `Database`

| Method | Returns | Description |
|--------|---------|-------------|
| `builder()` | `DatabaseBuilder` | Start building a new connection |
| `pool()` | `&Pool` | Borrow the connection pool |
| `into_pool()` | `Pool` | Consume and return the owned pool |

### `DatabaseBuilder`

| Method | Description |
|--------|-------------|
| `connection_string(s)` | Set connection string (required for MySQL/PostgreSQL, optional for SQLite) |
| `max_connections(n)` | Set maximum pool connections |
| `log_level(level)` | Set SQL log level (default: `Trace`) |
| `create_if_missing(bool)` | SQLite only: create DB file if missing (default: `true`) |
| `wal_mode(bool)` | SQLite only: enable WAL journal mode (default: `true`) |
| `foreign_keys(bool)` | SQLite only: enable foreign key constraints (default: `true`) |
| `build()` | Connect and return `Result<Database, DatabaseError>` |

### `Transaction`

| Method | Description |
|--------|-------------|
| `begin(pool)` | Start a new transaction |
| `commit()` | Commit the transaction |
| `rollback()` | Explicitly roll back (also happens automatically on drop) |

Implements `Deref<Target = sqlx::Transaction>` and `DerefMut`, so use `&mut **tx` to get the executor.

### Free Functions

| Function | Description |
|----------|-------------|
| `sql(query)` | Convert `?` placeholders for the active backend |
| `execute_schema(pool, sql)` | Execute a single DDL statement |
| `execute_schemas(pool, &[...])` | Execute multiple DDL statements in sequence |

### Re-exported from sqlx

| Item | Original | Description |
|------|----------|-------------|
| `query` | `sqlx::query` | Build untyped queries |
| `query_as` | `sqlx::query_as` | Build typed queries with `FromRow` |
| `query_scalar` | `sqlx::query_scalar` | Build single-value queries |
| `FromRow` | `sqlx::FromRow` | Derive macro for mapping rows to structs |
| `RowTrait` | `sqlx::Row` | Trait for accessing row columns by name |
| `Executor` | `sqlx::Executor` | Trait implemented by pools and transactions |
| `SqlxError` | `sqlx::Error` | The underlying sqlx error type |

### Type Aliases

| Type | SQLite | MySQL | PostgreSQL |
|------|--------|-------|------------|
| `Pool` | `SqlitePool` | `MySqlPool` | `PgPool` |
| `Row` | `SqliteRow` | `MySqlRow` | `PgRow` |
| `QueryResult` | `SqliteQueryResult` | `MySqlQueryResult` | `PgQueryResult` |

### `DatabaseError`

| Variant | Description |
|---------|-------------|
| `Sqlx(sqlx::Error)` | Passthrough from the underlying sqlx library |
| `ConnectionStringRequired { backend, example }` | MySQL/PostgreSQL require a connection string |
| `ConnectionStringParse(String)` | Failed to parse the connection string |
| `TransactionCommitFailed(sqlx::Error)` | Transaction commit failed |
| `SchemaExecutionFailed(sqlx::Error)` | A DDL statement failed |

## Examples

The `examples/` directory contains runnable examples:

### `basic_crud.rs` -- CRUD Operations

Creates a table, inserts rows, queries with `FromRow`, updates, and deletes.

```bash
cargo run --example basic_crud -p obsidian_database --features sqlite
```

### `transactions.rs` -- Transaction Handling

Demonstrates commit, rollback-on-drop, and multi-operation atomic transfers.

```bash
cargo run --example transactions -p obsidian_database --features sqlite
```

### `schema_management.rs` -- Schema and Migrations

Shows single and batch schema creation, indexes, and basic migration patterns.

```bash
cargo run --example schema_management -p obsidian_database --features sqlite
```

## Testing

Run all tests (unit + integration + doc-tests):

```bash
cargo test -p obsidian_database --features sqlite
```

The integration test suite covers connection management, CRUD operations, transactions, schema execution, error handling, and row trait usage. Tests use in-memory SQLite databases with `max_connections(1)` (required because each connection in a pool gets its own separate in-memory database).

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `sqlx` | 0.8 | Database driver and connection pool |
| `log` | 0.4 | Logging facade for SQL statement logging |
| `thiserror` | 2.0 | Derive macro for `DatabaseError` |
| `anyhow` | 1.0 | Flexible error handling |

### Dev Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1 | Async runtime for tests (`rt-multi-thread`, `macros`) |
