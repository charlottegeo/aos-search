use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

const DB_URL: &str = "sqlite://transcript.db";

pub async fn setup_database() -> Result<(), Box<dyn std::error::Error>> {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
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
    let db = SqlitePool::connect(DB_URL).await?;
    Ok(db)
}
