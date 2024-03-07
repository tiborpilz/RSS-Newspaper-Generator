use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

#[cfg(feature = "ssr")]
pub async fn connect_db() -> SqlitePool {
    let db_url = "db.sqlite3";
    let conn_opts = SqliteConnectOptions::new()
        .filename(db_url)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(conn_opts).await.unwrap();

    // Create the table if doesnt exist
    let _ = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS feeds (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await;
    return pool;
}
