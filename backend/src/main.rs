mod db;
mod models;
mod api;
mod file_parser;

use actix_web::{web, App, HttpServer};
use db::setup_database;
use api::init_routes;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./transcripts.db".to_string());
    let schema_path = env::var("SCHEMA_PATH").unwrap_or_else(|_| "./schema.sql".to_string());

    let db_pool = setup_database(&database_url, &schema_path).await?;

    file_parser::process_seasons(&db_pool).await?;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .configure(init_routes)
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await?;

    Ok(())
}