use crate::db::setup_database;
use crate::file_parser;
use crate::models::{
    Episode, Line, RandomLineQuery, SearchPhrasesQuery, Season, Speaker, UserQuery,
};
use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures_util::stream::StreamExt as _;
use sanitize_filename::sanitize;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use uuid::Uuid;
use zip::ZipArchive;

pub type DatabaseRegistry = Arc<Mutex<HashMap<String, SqlitePool>>>;

#[get("/init-db")]
async fn init_db(
    db_registry: web::Data<DatabaseRegistry>,
    schema_path: web::Data<String>,
) -> impl Responder {
    let user_id = Uuid::new_v4().to_string();
    let (db_pool, _db_path) = match setup_database(&user_id, schema_path.get_ref()).await {
        Ok((pool, path)) => (pool, path),
        Err(err) => {
            eprintln!("Failed to set up database: {}", err);
            return HttpResponse::InternalServerError().body("Failed to initialize database");
        }
    };

    db_registry
        .lock()
        .await
        .insert(user_id.clone(), db_pool.clone());

    HttpResponse::Ok().json(serde_json::json!({ "user_id": user_id }))
}

#[get("/cleanup/{user_id}")]
async fn cleanup_db(
    db_registry: web::Data<DatabaseRegistry>,
    user_id: web::Path<String>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    let removed = {
        let mut registry = db_registry.lock().await;
        registry.remove(&user_id)
    };

    if let Some(_pool) = removed {
        let db_path = format!("./temp_dbs/{}.sqlite", user_id);
        if let Err(err) = tokio::fs::remove_file(&db_path).await {
            eprintln!("Failed to remove database file {}: {}", db_path, err);
        }
        HttpResponse::Ok().body("Database cleaned up successfully")
    } else {
        HttpResponse::NotFound().body("User database not found")
    }
}

#[get("/search/phrases")]
async fn search_phrases(
    db_registry: web::Data<DatabaseRegistry>,
    query: web::Query<SearchPhrasesQuery>,
    user_query: web::Query<UserQuery>,
) -> impl Responder {
    let user_id = &user_query.user_id;

    if user_id.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "User ID is required" }));
    }

    let db_pool = match get_db_pool(db_registry, &user_query.user_id).await {
        Some(pool) => pool,
        None => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({ "error": "Database not found for user" }))
        }
    };

    let phrase = query.phrase.clone().unwrap_or_default();
    let season = query.season;
    let episode = query.episode;
    let speaker = query.speaker;
    let context_lines = query.context.unwrap_or(0);

    let mut sql_query = String::from(
        r#"
        SELECT 
            l.id, 
            l.season_id, 
            l.episode_id, 
            l.speaker_id, 
            s.name AS speaker_name, 
            l.line_number, 
            l.content
        FROM lines l
        LEFT JOIN speakers s ON l.speaker_id = s.id
        WHERE l.content LIKE ?
        "#,
    );
    let phrase_query = format!("%{}%", phrase);
    let mut params: Vec<Box<dyn std::fmt::Display>> = vec![Box::new(phrase_query)];

    if let Some(season_id) = season {
        sql_query.push_str(" AND l.season_id = ?");
        params.push(Box::new(season_id));
    }

    if let Some(episode_id) = episode {
        sql_query.push_str(" AND l.episode_id = ?");
        params.push(Box::new(episode_id));
    }

    if let Some(speaker_id) = speaker {
        sql_query.push_str(" AND l.speaker_id = ?");
        params.push(Box::new(speaker_id));
    }

    let mut query_builder = sqlx::query_as::<_, Line>(&sql_query);

    for param in params {
        query_builder = query_builder.bind(param.to_string());
    }

    let results: Vec<Line> = match query_builder.fetch_all(&db_pool).await {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error executing search: {}", err);
            return HttpResponse::InternalServerError().body("Error executing search");
        }
    };

    if context_lines > 0 {
        let mut results_with_context = vec![];

        for line in results {
            let context_query = r#"
                SELECT 
                    l.id, 
                    l.season_id, 
                    l.episode_id, 
                    l.speaker_id, 
                    s.name AS speaker_name, 
                    l.line_number, 
                    l.content
                FROM lines l
                LEFT JOIN speakers s ON l.speaker_id = s.id
                WHERE l.episode_id = ? AND l.line_number BETWEEN ? AND ?
                ORDER BY l.line_number
            "#;

            let context: Vec<Line> = sqlx::query_as(context_query)
                .bind(line.episode_id)
                .bind(line.line_number - context_lines)
                .bind(line.line_number + context_lines)
                .fetch_all(&db_pool)
                .await
                .unwrap_or_default();

            results_with_context.push((line, context));
        }

        return HttpResponse::Ok().json(results_with_context);
    }

    HttpResponse::Ok().json(results)
}

#[get("/random-line")]
async fn get_random_line(
    db_registry: web::Data<DatabaseRegistry>,
    query: web::Query<RandomLineQuery>,
    user_query: web::Query<UserQuery>,
) -> impl Responder {
    let db_pool = match get_db_pool(db_registry, &user_query.user_id).await {
        Some(pool) => pool,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "User ID is required" }))
        }
    };

    let mut sql_query = String::from(
        r#"
        SELECT 
            l.id, 
            l.season_id,
            l.episode_id,
            l.speaker_id,
            s.name AS speaker_name,
            l.line_number,
            l.content
        FROM lines l
        LEFT JOIN speakers s ON l.speaker_id = s.id
        "#,
    );

    let mut conditions = Vec::new();
    let mut params = Vec::new();

    if let Some(season_id) = query.season {
        conditions.push("l.season_id = ?");
        params.push(season_id);
    }
    if let Some(episode_id) = query.episode {
        conditions.push("l.episode_id = ?");
        params.push(episode_id);
    }
    if let Some(speaker_id) = query.speaker {
        conditions.push("l.speaker_id = ?");
        params.push(speaker_id);
    }

    if !conditions.is_empty() {
        sql_query.push_str(" WHERE ");
        sql_query.push_str(&conditions.join(" AND "));
    }

    sql_query.push_str(" ORDER BY RANDOM() LIMIT 1;");

    let mut query_builder = sqlx::query_as::<_, Line>(&sql_query);
    for param in params {
        query_builder = query_builder.bind(param);
    }

    match query_builder.fetch_one(&db_pool).await {
        Ok(line) => HttpResponse::Ok().json(line),
        Err(err) => {
            eprintln!("Error fetching random line: {:?}", err);
            HttpResponse::NotFound().body("No matching line found")
        }
    }
}

#[get("/transcripts/{season_num}/{episode_num}")]
async fn get_transcript(
    db_registry: web::Data<DatabaseRegistry>,
    path: web::Path<(i64, i32)>,
    user_query: web::Query<UserQuery>,
) -> impl Responder {
    let user_id = &user_query.user_id;

    if user_id.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "User ID is required" }));
    }

    let db_pool = match get_db_pool(db_registry, &user_query.user_id).await {
        Some(pool) => pool,
        None => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({ "error": "Database not found for user" }))
        }
    };

    let (season_num, episode_num) = path.into_inner();

    let season_exists: i64 = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM seasons WHERE number = ?)",
        season_num
    )
    .fetch_one(&db_pool)
    .await
    .unwrap_or(0);

    if season_exists == 0 {
        return HttpResponse::NotFound().body(format!("Season {} not found", season_num));
    }

    let episode_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM episodes WHERE number = ?)",
        episode_num
    )
    .fetch_one(&db_pool)
    .await
    .unwrap_or(0);

    if episode_exists == 0 {
        return HttpResponse::NotFound().body(format!("Episode {} not found", episode_num));
    }

    let query = r#"
    SELECT 
        l.id, 
        l.season_id, 
        l.episode_id, 
        l.speaker_id, 
        s.name AS speaker_name, 
        l.line_number, 
        l.content
    FROM lines l
    LEFT JOIN speakers s ON l.speaker_id = s.id
    JOIN episodes e ON l.episode_id = e.id
    JOIN seasons sn ON e.season_id = sn.id
    WHERE sn.number = ? AND e.number = ?
    "#;

    let transcript = sqlx::query_as::<_, Line>(query)
        .bind(season_num)
        .bind(episode_num)
        .fetch_all(&db_pool)
        .await;

    match transcript {
        Ok(transcript) => HttpResponse::Ok().json(transcript),
        Err(err) => {
            eprintln!("Error fetching transcript: {}", err);
            HttpResponse::InternalServerError().body(format!("Error fetching transcript: {}", err))
        }
    }
}

#[get("/seasons")]
async fn get_seasons(
    db_registry: web::Data<DatabaseRegistry>,
    user_query: web::Query<UserQuery>,
) -> impl Responder {
    let user_id = &user_query.user_id;

    if user_id.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "User ID is required" }));
    }

    let db_pool = match get_db_pool(db_registry, &user_query.user_id).await {
        Some(pool) => pool,
        None => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({ "error": "Database not found for user" }))
        }
    };

    let seasons = match sqlx::query_as::<_, Season>("SELECT * FROM seasons")
        .fetch_all(&db_pool)
        .await
    {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error fetching seasons: {}", err);
            return HttpResponse::InternalServerError().body("Error fetching seasons");
        }
    };

    HttpResponse::Ok().json(seasons)
}

#[get("/speakers")]
async fn get_speakers(
    db_registry: web::Data<DatabaseRegistry>,
    user_query: web::Query<UserQuery>,
) -> impl Responder {
    let user_id = &user_query.user_id;

    if user_id.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "User ID is required" }));
    }

    let db_pool = match get_db_pool(db_registry, &user_query.user_id).await {
        Some(pool) => pool,
        None => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({ "error": "Database not found for user" }))
        }
    };

    let speakers = match sqlx::query_as::<_, Speaker>("SELECT * FROM speakers")
        .fetch_all(&db_pool)
        .await
    {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error fetching speakers: {}", err);
            return HttpResponse::InternalServerError().body("Error fetching speakers");
        }
    };

    HttpResponse::Ok().json(speakers)
}

#[get("/seasons/{season_id}/episodes")]
async fn get_episodes(
    db_registry: web::Data<DatabaseRegistry>,
    season_id: web::Path<i64>,
    user_query: web::Query<UserQuery>,
) -> impl Responder {
    let user_id = &user_query.user_id;

    if user_id.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "User ID is required" }));
    }

    let db_pool = match get_db_pool(db_registry, &user_query.user_id).await {
        Some(pool) => pool,
        None => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({ "error": "Database not found for user" }))
        }
    };

    let episodes = match sqlx::query_as::<_, Episode>(
        "SELECT * FROM episodes WHERE season_id = ? ORDER BY number ASC",
    )
    .bind(season_id.into_inner())
    .fetch_all(&db_pool)
    .await
    {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error fetching episodes: {}", err);
            return HttpResponse::InternalServerError().body("Error fetching episodes");
        }
    };

    HttpResponse::Ok().json(episodes)
}

#[post("/upload")]
async fn upload_zip(
    mut payload: Multipart,
    db_registry: web::Data<DatabaseRegistry>,
    query: web::Query<UserQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = &query.user_id;
    let temp_dir = "./temp_uploads";

    if user_id.is_empty() {
        return Ok(
            HttpResponse::BadRequest().json(serde_json::json!({ "error": "User ID is required" }))
        );
    }

    let db_pool = {
        let registry = db_registry.lock().await;
        match registry.get(user_id) {
            Some(pool) => pool.clone(),
            None => {
                return Ok(HttpResponse::NotFound()
                    .json(serde_json::json!({ "error": "Database not found for user" })))
            }
        }
    };

    if !Path::new(temp_dir).exists() {
        fs::create_dir_all(temp_dir).await.map_err(|err| {
            eprintln!("Failed to create temp directory: {}", err);
            actix_web::error::ErrorInternalServerError("Failed to create temp directory")
        })?;
    }

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|err| {
            eprintln!("Error reading multipart field: {}", err);
            actix_web::error::ErrorInternalServerError("Failed to process multipart data")
        })?;

        let filename = field
            .content_disposition()
            .and_then(|cd| cd.get_filename())
            .map(sanitize)
            .unwrap_or_else(|| "upload.zip".to_string());
        let filepath = format!("{}/{}", temp_dir, filename);

        let mut f = fs::File::create(&filepath).await.map_err(|err| {
            eprintln!("Failed to create file: {}", err);
            actix_web::error::ErrorInternalServerError("Failed to create file")
        })?;

        while let Some(chunk) = field.next().await {
            if let Ok(data) = chunk {
                f.write_all(&data).await.map_err(|err| {
                    eprintln!("Error writing to file {}: {}", filepath, err);
                    actix_web::error::ErrorInternalServerError("Failed to write to file")
                })?;
            }
        }

        let extract_path = format!("{}/extracted", temp_dir);
        fs::create_dir_all(&extract_path).await.map_err(|err| {
            eprintln!("Failed to create extract directory: {}", err);
            actix_web::error::ErrorInternalServerError("Failed to create extract directory")
        })?;

        let file = fs::File::open(&filepath).await.map_err(|err| {
            eprintln!("Failed to open uploaded file: {}", err);
            actix_web::error::ErrorInternalServerError("Failed to process uploaded file")
        })?;

        let mut archive = ZipArchive::new(file.into_std().await).map_err(|err| {
            eprintln!("Failed to open ZIP archive: {}", err);
            actix_web::error::ErrorInternalServerError("Failed to process ZIP file")
        })?;

        archive.extract(&extract_path).map_err(|err| {
            eprintln!("Failed to extract ZIP archive: {}", err);
            actix_web::error::ErrorInternalServerError("Failed to extract ZIP file")
        })?;

        file_parser::process_seasons(&db_pool, &extract_path)
            .await
            .map_err(|err| {
                eprintln!("Failed to process seasons: {}", err);
                actix_web::error::ErrorInternalServerError("Failed to process ZIP content")
            })?;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Upload successful" })))
}

async fn get_db_pool(
    db_registry: web::Data<DatabaseRegistry>,
    user_id: &str,
) -> Option<SqlitePool> {
    if user_id.is_empty() {
        return None;
    }
    let registry = db_registry.lock().await;
    registry.get(user_id).cloned()
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(init_db)
        .service(cleanup_db)
        .service(upload_zip)
        .service(get_transcript)
        .service(get_random_line)
        .service(get_speakers)
        .service(get_seasons)
        .service(get_episodes)
        .service(search_phrases);
}
