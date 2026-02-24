//! Basic CRUD operations using obsidian_database.
//!
//! Run with: `cargo run --example basic_crud --features sqlite`

use obsidian_database::{execute_schema, query, query_as, sql, Database, DatabaseError, FromRow};

#[derive(Debug, FromRow)]
struct User {
    id: i64,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<(), DatabaseError> {
    let db = Database::builder()
        .connection_string(":memory:")
        .build()
        .await?;

    let pool = db.pool();

    // Create table
    execute_schema(
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE
        );
        "#,
    )
    .await?;

    // Insert rows
    query(&*sql("INSERT INTO users (name, email) VALUES (?, ?)"))
        .bind("Alice")
        .bind("alice@example.com")
        .execute(pool)
        .await
        .map_err(DatabaseError::from)?;

    query(&*sql("INSERT INTO users (name, email) VALUES (?, ?)"))
        .bind("Bob")
        .bind("bob@example.com")
        .execute(pool)
        .await
        .map_err(DatabaseError::from)?;

    // Fetch all users
    let users: Vec<User> = query_as(&*sql("SELECT id, name, email FROM users"))
        .fetch_all(pool)
        .await
        .map_err(DatabaseError::from)?;

    println!("All users:");
    for user in &users {
        println!("  [{id}] {name} <{email}>", id = user.id, name = user.name, email = user.email);
    }

    // Fetch single user
    let alice: User = query_as(&*sql("SELECT id, name, email FROM users WHERE name = ?"))
        .bind("Alice")
        .fetch_one(pool)
        .await
        .map_err(DatabaseError::from)?;

    println!("\nFetched: {} <{}>", alice.name, alice.email);

    // Update
    query(&*sql("UPDATE users SET email = ? WHERE name = ?"))
        .bind("alice@newdomain.com")
        .bind("Alice")
        .execute(pool)
        .await
        .map_err(DatabaseError::from)?;

    // Delete
    query(&*sql("DELETE FROM users WHERE name = ?"))
        .bind("Bob")
        .execute(pool)
        .await
        .map_err(DatabaseError::from)?;

    // Verify
    let remaining: Vec<User> = query_as(&*sql("SELECT id, name, email FROM users"))
        .fetch_all(pool)
        .await
        .map_err(DatabaseError::from)?;

    println!("\nAfter update and delete:");
    for user in &remaining {
        println!("  [{id}] {name} <{email}>", id = user.id, name = user.name, email = user.email);
    }

    Ok(())
}
