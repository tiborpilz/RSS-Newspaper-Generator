use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

#[cfg(feature = "ssr")]
pub async fn connect_db() -> SqlitePool {
    let db_url = "db.sqlite3";
    let conn_opts = SqliteConnectOptions::new()
        .filename(db_url)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(conn_opts).await.unwrap();

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    return pool;
}
