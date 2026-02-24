//! Schema management with execute_schema and execute_schemas.
//!
//! Run with: `cargo run --example schema_management --features sqlite`

use obsidian_database::{execute_schema, execute_schemas, query, query_as, sql, Database, DatabaseError, FromRow};

#[derive(Debug, FromRow)]
struct TableInfo {
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), DatabaseError> {
    let db = Database::builder()
        .connection_string(":memory:")
        .build()
        .await?;

    let pool = db.pool();

    // Create multiple tables in a single call
    execute_schemas(
        pool,
        &[
            r#"
            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                category_id INTEGER REFERENCES categories(id),
                price INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_items_category ON items(category_id);
            "#,
        ],
    )
    .await?;

    println!("Schema created successfully.");

    // Verify tables exist
    let tables: Vec<TableInfo> = query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .map_err(DatabaseError::from)?;

    println!("\nTables in database:");
    for table in &tables {
        println!("  - {}", table.name);
    }

    // Add a migration-style schema change
    execute_schema(
        pool,
        "ALTER TABLE items ADD COLUMN sku TEXT;",
    )
    .await?;

    println!("\nMigration applied: added 'sku' column to items.");

    // Insert some data
    query(&*sql("INSERT INTO categories (name, description) VALUES (?, ?)"))
        .bind("Electronics")
        .bind("Gadgets and devices")
        .execute(pool)
        .await
        .map_err(DatabaseError::from)?;

    query(&*sql("INSERT INTO items (name, category_id, price, sku) VALUES (?, ?, ?, ?)"))
        .bind("Widget Pro")
        .bind(1i64)
        .bind(2999i64)
        .bind("WGT-PRO-001")
        .execute(pool)
        .await
        .map_err(DatabaseError::from)?;

    #[derive(Debug, FromRow)]
    struct ItemRow {
        name: String,
        price: i64,
        sku: Option<String>,
    }

    let items: Vec<ItemRow> = query_as(&*sql("SELECT name, price, sku FROM items"))
        .fetch_all(pool)
        .await
        .map_err(DatabaseError::from)?;

    println!("\nItems:");
    for item in &items {
        println!(
            "  {} - ${:.2} (SKU: {})",
            item.name,
            item.price as f64 / 100.0,
            item.sku.as_deref().unwrap_or("N/A")
        );
    }

    Ok(())
}
