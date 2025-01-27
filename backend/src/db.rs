use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::env;
use std::{fs, path::Path};

pub async fn setup_database(database_url: &str, schema_path: &str) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let path = Path::new(database_url.strip_prefix("sqlite:").unwrap_or(database_url));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|err| {
            panic!("Failed to create parent directory: {}", err);
        });
    }
    let db = SqlitePool::connect(database_url).await?;
    let schema = std::fs::read_to_string(schema_path)?;
    sqlx::query(&schema).execute(&db).await?;

    Ok(db)
}


pub async fn get_pool() -> Result<SqlitePool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").map_err(|e| {
        sqlx::Error::Configuration(e.into())
    })?;
    let db = SqlitePool::connect(&database_url).await?;
    Ok(db)
}