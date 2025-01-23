mod db;
mod models;
mod file_parser;

use db::setup_database;
use file_parser::process_seasons;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_database().await?;
    let db = db::get_pool().await?;
    process_seasons(&db).await?;
    Ok(())
}