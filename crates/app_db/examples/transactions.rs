//! Transaction usage with obsidian_database.
//!
//! Run with: `cargo run --example transactions --features sqlite`

use obsidian_database::{execute_schema, query, query_scalar, sql, Database, DatabaseError, Transaction};

#[tokio::main]
async fn main() -> Result<(), DatabaseError> {
    let db = Database::builder()
        .connection_string(":memory:")
        .build()
        .await?;

    let pool = db.pool();

    execute_schema(
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            balance INTEGER NOT NULL DEFAULT 0
        );
        "#,
    )
    .await?;

    // Seed accounts
    query(&*sql("INSERT INTO accounts (name, balance) VALUES (?, ?)"))
        .bind("Alice")
        .bind(1000i64)
        .execute(pool)
        .await
        .map_err(DatabaseError::from)?;

    query(&*sql("INSERT INTO accounts (name, balance) VALUES (?, ?)"))
        .bind("Bob")
        .bind(500i64)
        .execute(pool)
        .await
        .map_err(DatabaseError::from)?;

    // Transfer within a transaction
    let amount = 200i64;
    println!("Transferring {amount} from Alice to Bob...");

    let mut tx = Transaction::begin(pool).await?;

    query(&*sql("UPDATE accounts SET balance = balance - ? WHERE name = ?"))
        .bind(amount)
        .bind("Alice")
        .execute(&mut **tx)
        .await
        .map_err(DatabaseError::from)?;

    query(&*sql("UPDATE accounts SET balance = balance + ? WHERE name = ?"))
        .bind(amount)
        .bind("Bob")
        .execute(&mut **tx)
        .await
        .map_err(DatabaseError::from)?;

    tx.commit().await?;

    // Check balances
    let alice_balance: i64 = query_scalar(&*sql("SELECT balance FROM accounts WHERE name = ?"))
        .bind("Alice")
        .fetch_one(pool)
        .await
        .map_err(DatabaseError::from)?;

    let bob_balance: i64 = query_scalar(&*sql("SELECT balance FROM accounts WHERE name = ?"))
        .bind("Bob")
        .fetch_one(pool)
        .await
        .map_err(DatabaseError::from)?;

    println!("Alice: {alice_balance}");
    println!("Bob:   {bob_balance}");
    assert_eq!(alice_balance, 800);
    assert_eq!(bob_balance, 700);

    // Demonstrate rollback (transaction dropped without commit)
    println!("\nStarting a transaction that will be rolled back...");
    {
        let mut tx = Transaction::begin(pool).await?;

        query(&*sql("UPDATE accounts SET balance = 0 WHERE name = ?"))
            .bind("Alice")
            .execute(&mut **tx)
            .await
            .map_err(DatabaseError::from)?;

        // Drop tx without committing -- auto-rollback
        drop(tx);
    }

    let alice_balance: i64 = query_scalar(&*sql("SELECT balance FROM accounts WHERE name = ?"))
        .bind("Alice")
        .fetch_one(pool)
        .await
        .map_err(DatabaseError::from)?;

    println!("Alice after rollback: {alice_balance}");
    assert_eq!(alice_balance, 800);
    println!("Rollback worked correctly.");

    Ok(())
}
