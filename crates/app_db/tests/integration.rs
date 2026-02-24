use obsidian_database::{
    execute_schema, execute_schemas, query, query_as, query_scalar, sql, Database, DatabaseError,
    FromRow, Transaction,
};

async fn test_db() -> Database {
    // max_connections(1) is required for :memory: SQLite because each
    // connection in a pool gets its own separate in-memory database.
    Database::builder()
        .connection_string(":memory:")
        .max_connections(1)
        .build()
        .await
        .expect("Failed to create in-memory database")
}

// ── Connection & builder tests ───────────────────────────────────

#[tokio::test]
async fn test_connect_in_memory() {
    let db = test_db().await;
    let pool = db.pool();

    // Verify the pool is functional
    let row: (i64,) = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await
        .expect("SELECT 1 should succeed");
    assert_eq!(row.0, 1);
}

#[tokio::test]
async fn test_into_pool() {
    let db = test_db().await;
    let pool = db.into_pool();

    let row: (i64,) = sqlx::query_as("SELECT 42")
        .fetch_one(&pool)
        .await
        .expect("SELECT should succeed on owned pool");
    assert_eq!(row.0, 42);
}

#[tokio::test]
async fn test_builder_max_connections() {
    let db = Database::builder()
        .connection_string(":memory:")
        .max_connections(2)
        .build()
        .await
        .expect("Should connect with max_connections=2");

    let row: (i64,) = sqlx::query_as("SELECT 1")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(row.0, 1);
}

#[tokio::test]
async fn test_builder_log_level() {
    let db = Database::builder()
        .connection_string(":memory:")
        .log_level(log::LevelFilter::Off)
        .build()
        .await
        .expect("Should connect with custom log level");

    let row: (i64,) = sqlx::query_as("SELECT 1")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(row.0, 1);
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_builder_sqlite_options() {
    let db = Database::builder()
        .connection_string(":memory:")
        .wal_mode(false)
        .foreign_keys(true)
        .create_if_missing(true)
        .build()
        .await
        .expect("Should connect with custom SQLite options");

    let row: (i64,) = sqlx::query_as("SELECT 1")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(row.0, 1);
}

// ── sql() placeholder tests ──────────────────────────────────────

#[test]
fn test_sql_passthrough_no_placeholders() {
    let result = sql("SELECT * FROM users");
    assert_eq!(&*result, "SELECT * FROM users");
}

#[test]
fn test_sql_with_placeholders() {
    let result = sql("INSERT INTO t (a, b, c) VALUES (?, ?, ?)");
    #[cfg(any(feature = "sqlite", feature = "mysql"))]
    assert_eq!(&*result, "INSERT INTO t (a, b, c) VALUES (?, ?, ?)");
    #[cfg(feature = "postgres")]
    assert_eq!(&*result, "INSERT INTO t (a, b, c) VALUES ($1, $2, $3)");
}

// ── Schema execution tests ───────────────────────────────────────

#[tokio::test]
async fn test_execute_schema_create_table() {
    let db = test_db().await;
    let pool = db.pool();

    execute_schema(
        pool,
        "CREATE TABLE test_table (id INTEGER PRIMARY KEY, value TEXT NOT NULL);",
    )
    .await
    .expect("Schema creation should succeed");

    // Verify table exists by inserting
    query("INSERT INTO test_table (id, value) VALUES (1, 'hello')")
        .execute(pool)
        .await
        .expect("Insert should succeed");
}

#[tokio::test]
async fn test_execute_schemas_multiple() {
    let db = test_db().await;
    let pool = db.pool();

    execute_schemas(
        pool,
        &[
            "CREATE TABLE parent (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
            "CREATE TABLE child (id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES parent(id));",
        ],
    )
    .await
    .expect("Multiple schema statements should succeed");

    // Verify both tables
    query("INSERT INTO parent (id, name) VALUES (1, 'p1')")
        .execute(pool)
        .await
        .unwrap();
    query("INSERT INTO child (id, parent_id) VALUES (1, 1)")
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_execute_schema_invalid_sql() {
    let db = test_db().await;
    let pool = db.pool();

    let result = execute_schema(pool, "THIS IS NOT SQL").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        DatabaseError::SchemaExecutionFailed(_) => {} // expected
        other => panic!("Expected SchemaExecutionFailed, got: {other:?}"),
    }
}

// ── CRUD operations tests ────────────────────────────────────────

#[derive(Debug, FromRow, PartialEq)]
struct TestRow {
    id: i64,
    name: String,
    score: i64,
}

async fn setup_test_table(pool: &obsidian_database::Pool) {
    execute_schema(
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS test_rows (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            score INTEGER NOT NULL DEFAULT 0
        );
        "#,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_insert_and_fetch_one() {
    let db = test_db().await;
    let pool = db.pool();
    setup_test_table(pool).await;

    query(&*sql("INSERT INTO test_rows (name, score) VALUES (?, ?)"))
        .bind("Alice")
        .bind(100i64)
        .execute(pool)
        .await
        .unwrap();

    let row: TestRow = query_as(&*sql("SELECT id, name, score FROM test_rows WHERE name = ?"))
        .bind("Alice")
        .fetch_one(pool)
        .await
        .unwrap();

    assert_eq!(row.name, "Alice");
    assert_eq!(row.score, 100);
    assert_eq!(row.id, 1);
}

#[tokio::test]
async fn test_insert_and_fetch_all() {
    let db = test_db().await;
    let pool = db.pool();
    setup_test_table(pool).await;

    for (name, score) in [("Alice", 100), ("Bob", 200), ("Carol", 150)] {
        query(&*sql("INSERT INTO test_rows (name, score) VALUES (?, ?)"))
            .bind(name)
            .bind(score as i64)
            .execute(pool)
            .await
            .unwrap();
    }

    let rows: Vec<TestRow> =
        query_as(&*sql("SELECT id, name, score FROM test_rows ORDER BY name"))
            .fetch_all(pool)
            .await
            .unwrap();

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].name, "Alice");
    assert_eq!(rows[1].name, "Bob");
    assert_eq!(rows[2].name, "Carol");
}

#[tokio::test]
async fn test_fetch_optional() {
    let db = test_db().await;
    let pool = db.pool();
    setup_test_table(pool).await;

    let missing: Option<TestRow> =
        query_as(&*sql("SELECT id, name, score FROM test_rows WHERE name = ?"))
            .bind("Nobody")
            .fetch_optional(pool)
            .await
            .unwrap();

    assert!(missing.is_none());
}

#[tokio::test]
async fn test_query_scalar() {
    let db = test_db().await;
    let pool = db.pool();
    setup_test_table(pool).await;

    for i in 0..5 {
        query(&*sql("INSERT INTO test_rows (name, score) VALUES (?, ?)"))
            .bind(format!("user_{i}"))
            .bind(i * 10i64)
            .execute(pool)
            .await
            .unwrap();
    }

    let count: i64 = query_scalar(&*sql("SELECT COUNT(*) FROM test_rows"))
        .fetch_one(pool)
        .await
        .unwrap();

    assert_eq!(count, 5);

    let total: i64 = query_scalar(&*sql("SELECT SUM(score) FROM test_rows"))
        .fetch_one(pool)
        .await
        .unwrap();

    assert_eq!(total, 0 + 10 + 20 + 30 + 40);
}

#[tokio::test]
async fn test_update_rows() {
    let db = test_db().await;
    let pool = db.pool();
    setup_test_table(pool).await;

    query(&*sql("INSERT INTO test_rows (name, score) VALUES (?, ?)"))
        .bind("Alice")
        .bind(100i64)
        .execute(pool)
        .await
        .unwrap();

    let result = query(&*sql("UPDATE test_rows SET score = ? WHERE name = ?"))
        .bind(999i64)
        .bind("Alice")
        .execute(pool)
        .await
        .unwrap();

    assert_eq!(result.rows_affected(), 1);

    let score: i64 = query_scalar(&*sql("SELECT score FROM test_rows WHERE name = ?"))
        .bind("Alice")
        .fetch_one(pool)
        .await
        .unwrap();

    assert_eq!(score, 999);
}

#[tokio::test]
async fn test_delete_rows() {
    let db = test_db().await;
    let pool = db.pool();
    setup_test_table(pool).await;

    for name in ["Alice", "Bob", "Carol"] {
        query(&*sql("INSERT INTO test_rows (name, score) VALUES (?, ?)"))
            .bind(name)
            .bind(0i64)
            .execute(pool)
            .await
            .unwrap();
    }

    let result = query(&*sql("DELETE FROM test_rows WHERE name = ?"))
        .bind("Bob")
        .execute(pool)
        .await
        .unwrap();

    assert_eq!(result.rows_affected(), 1);

    let count: i64 = query_scalar("SELECT COUNT(*) FROM test_rows")
        .fetch_one(pool)
        .await
        .unwrap();

    assert_eq!(count, 2);
}

// ── Transaction tests ────────────────────────────────────────────

#[tokio::test]
async fn test_transaction_commit() {
    let db = test_db().await;
    let pool = db.pool();
    setup_test_table(pool).await;

    let mut tx = Transaction::begin(pool).await.unwrap();

    query(&*sql("INSERT INTO test_rows (name, score) VALUES (?, ?)"))
        .bind("TxUser")
        .bind(42i64)
        .execute(&mut **tx)
        .await
        .unwrap();

    tx.commit().await.unwrap();

    // Data should be visible after commit
    let count: i64 = query_scalar(&*sql("SELECT COUNT(*) FROM test_rows WHERE name = ?"))
        .bind("TxUser")
        .fetch_one(pool)
        .await
        .unwrap();

    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_transaction_rollback_on_drop() {
    let db = test_db().await;
    let pool = db.pool();
    setup_test_table(pool).await;

    {
        let mut tx = Transaction::begin(pool).await.unwrap();

        query(&*sql("INSERT INTO test_rows (name, score) VALUES (?, ?)"))
            .bind("GhostUser")
            .bind(0i64)
            .execute(&mut **tx)
            .await
            .unwrap();

        // Drop without commit -- should auto-rollback
        drop(tx);
    }

    let count: i64 = query_scalar(&*sql("SELECT COUNT(*) FROM test_rows WHERE name = ?"))
        .bind("GhostUser")
        .fetch_one(pool)
        .await
        .unwrap();

    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_transaction_explicit_rollback() {
    let db = test_db().await;
    let pool = db.pool();
    setup_test_table(pool).await;

    let mut tx = Transaction::begin(pool).await.unwrap();

    query(&*sql("INSERT INTO test_rows (name, score) VALUES (?, ?)"))
        .bind("RolledBack")
        .bind(0i64)
        .execute(&mut **tx)
        .await
        .unwrap();

    tx.rollback().await.unwrap();

    let count: i64 = query_scalar(&*sql("SELECT COUNT(*) FROM test_rows WHERE name = ?"))
        .bind("RolledBack")
        .fetch_one(pool)
        .await
        .unwrap();

    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_transaction_multiple_operations() {
    let db = test_db().await;
    let pool = db.pool();
    setup_test_table(pool).await;

    // Seed data outside transaction
    query(&*sql("INSERT INTO test_rows (name, score) VALUES (?, ?)"))
        .bind("Account_A")
        .bind(1000i64)
        .execute(pool)
        .await
        .unwrap();

    query(&*sql("INSERT INTO test_rows (name, score) VALUES (?, ?)"))
        .bind("Account_B")
        .bind(500i64)
        .execute(pool)
        .await
        .unwrap();

    // Transfer 200 in a transaction
    let mut tx = Transaction::begin(pool).await.unwrap();

    query(&*sql("UPDATE test_rows SET score = score - ? WHERE name = ?"))
        .bind(200i64)
        .bind("Account_A")
        .execute(&mut **tx)
        .await
        .unwrap();

    query(&*sql("UPDATE test_rows SET score = score + ? WHERE name = ?"))
        .bind(200i64)
        .bind("Account_B")
        .execute(&mut **tx)
        .await
        .unwrap();

    tx.commit().await.unwrap();

    let a: i64 = query_scalar(&*sql("SELECT score FROM test_rows WHERE name = ?"))
        .bind("Account_A")
        .fetch_one(pool)
        .await
        .unwrap();

    let b: i64 = query_scalar(&*sql("SELECT score FROM test_rows WHERE name = ?"))
        .bind("Account_B")
        .fetch_one(pool)
        .await
        .unwrap();

    assert_eq!(a, 800);
    assert_eq!(b, 700);
}

// ── Error handling tests ─────────────────────────────────────────

#[tokio::test]
async fn test_error_from_sqlx() {
    let db = test_db().await;
    let pool = db.pool();

    // Query a non-existent table
    let result = query("SELECT * FROM nonexistent_table")
        .execute(pool)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_unique_constraint() {
    let db = test_db().await;
    let pool = db.pool();

    execute_schema(
        pool,
        "CREATE TABLE unique_test (id INTEGER PRIMARY KEY, val TEXT UNIQUE);",
    )
    .await
    .unwrap();

    query("INSERT INTO unique_test (id, val) VALUES (1, 'a')")
        .execute(pool)
        .await
        .unwrap();

    // Duplicate should fail
    let result = query("INSERT INTO unique_test (id, val) VALUES (2, 'a')")
        .execute(pool)
        .await;

    assert!(result.is_err());
}

// ── Row trait usage tests ────────────────────────────────────────

#[tokio::test]
async fn test_row_trait_try_get() {
    use obsidian_database::RowTrait;

    let db = test_db().await;
    let pool = db.pool();

    execute_schema(pool, "CREATE TABLE kv (key TEXT PRIMARY KEY, value TEXT);")
        .await
        .unwrap();

    query("INSERT INTO kv (key, value) VALUES ('greeting', 'hello world')")
        .execute(pool)
        .await
        .unwrap();

    let row = query("SELECT key, value FROM kv WHERE key = 'greeting'")
        .fetch_one(pool)
        .await
        .unwrap();

    let key: &str = row.try_get("key").unwrap();
    let value: &str = row.try_get("value").unwrap();

    assert_eq!(key, "greeting");
    assert_eq!(value, "hello world");
}
