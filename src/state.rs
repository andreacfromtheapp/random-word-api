// App State
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

use crate::error::SqlxError;

/// Configure the database pool
pub async fn init_dbpool(db_url: &str) -> Result<sqlx::Pool<sqlx::Sqlite>, SqlxError> {
    let dbpool = SqlitePoolOptions::new()
        .connect_with(SqliteConnectOptions::from_str(db_url)?.create_if_missing(true))
        .await?;

    sqlx::migrate!("./migrations").run(&dbpool).await?;

    Ok(dbpool)
}
