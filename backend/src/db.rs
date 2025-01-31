use lazy_static::lazy_static;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Sqlite, SqlitePool};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::Mutex;

lazy_static! {
    static ref DB_CACHE: Mutex<HashMap<String, SqlitePool>> = Mutex::new(HashMap::new());
}

pub async fn setup_database(
    user_id: &str,
    schema_path: &str,
) -> Result<(SqlitePool, PathBuf), Box<dyn std::error::Error>> {
    let db_path = Path::new("./temp_dbs").join(format!("{}.sqlite", user_id));
    fs::create_dir_all("./temp_dbs").await?;

    let database_url = format!("sqlite://{}", db_path.to_string_lossy());
    {
        let cache = DB_CACHE.lock().await;
        if let Some(pool) = cache.get(&database_url) {
            return Ok((pool.clone(), db_path));
        }
    }

    let db_pool = if !Sqlite::database_exists(&database_url)
        .await
        .unwrap_or(false)
    {
        Sqlite::create_database(&database_url).await?;
        println!("Database created for user: {}", user_id);

        let pool = SqlitePool::connect(&database_url).await?;
        let schema = tokio::fs::read_to_string(schema_path).await?;
        sqlx::query(&schema).execute(&pool).await?;

        pool
    } else {
        SqlitePool::connect(&database_url).await?
    };
    DB_CACHE
        .lock()
        .await
        .insert(database_url.clone(), db_pool.clone());

    Ok((db_pool, db_path))
}
