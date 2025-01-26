mod db;
mod models;
mod api;

use actix_web::{web, App, HttpServer};
use db::setup_database;
use api::init_routes;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    setup_database().await?;
    let db_pool = db::get_pool().await?;
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