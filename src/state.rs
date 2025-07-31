use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

/// Configure the database pool
pub async fn init_dbpool(db_url: &str) -> Result<sqlx::Pool<sqlx::Sqlite>, sqlx::Error> {
    let dbpool = SqlitePoolOptions::new()
        .connect_with(SqliteConnectOptions::from_str(db_url)?.create_if_missing(true))
        .await
        .expect("can't connect to database");

    sqlx::migrate!("./migrations")
        .run(&dbpool)
        .await
        .expect("database migration failed");

    Ok(dbpool)
}
