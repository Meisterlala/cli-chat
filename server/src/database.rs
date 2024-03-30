use log::info;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};

use crate::ChatMessage;

static MESSAGE_RETRIVAL_AMOUNT: u32 = 100;

pub async fn establish_connection(database_url: &str) -> anyhow::Result<Pool<Sqlite>> {
    // Create database if needed
    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        info!("Database does not exist, creating it");
        Sqlite::create_database(database_url).await?;
    }

    SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|e| e.into())
}

pub async fn create_table(pool: &Pool<Sqlite>) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            group_name TEXT NOT NULL,
            username TEXT NOT NULL,
            message TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create table");

    // Count number of messages
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages")
        .fetch_one(pool)
        .await
        .expect("Failed to count messages");
    info!("Database contains {} messages", count);
}

pub async fn insert_message(pool: &Pool<Sqlite>, group_name: &str, message: &ChatMessage) {
    sqlx::query(
        r#"
        INSERT INTO messages (group_name, username, message)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(group_name)
    .bind(&message.username)
    .bind(&message.message)
    .execute(pool)
    .await
    .expect("Failed to insert message");
}

pub async fn get_messages(pool: &Pool<Sqlite>, group_name: &str) -> Vec<ChatMessage> {
    sqlx::query_as(
        r#"
        SELECT username, message
        FROM messages
        WHERE group_name = ?
        ORDER BY id ASC
        LIMIT ?
        "#,
    )
    .bind(group_name)
    .bind(MESSAGE_RETRIVAL_AMOUNT)
    .fetch_all(pool)
    .await
    .map(|messages: Vec<(String, String)>| {
        messages
            .into_iter()
            .map(|(username, message)| ChatMessage { username, message })
            .collect()
    })
    .expect("Failed to fetch messages")
}
