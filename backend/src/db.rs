use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::env;
pub async fn setup_database() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")?;

    if !Sqlite::database_exists(&database_url).await.unwrap_or(false) {
        println!("Creating database {}", database_url);
        match Sqlite::create_database(&database_url).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    let db = get_pool().await?;
    let sql = include_str!("../schema.sql");
    sqlx::query(&sql).execute(&db).await.unwrap();
    Ok(())
} 

pub async fn get_pool() -> Result<SqlitePool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").map_err(|e| {
        sqlx::Error::Configuration(e.into())
    })?;
    let db = SqlitePool::connect(&database_url).await?;
    Ok(db)
}
