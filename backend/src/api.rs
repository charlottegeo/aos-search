use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use sqlx::SqlitePool;
use crate::models::{Season, Episode, Line, Speaker};

#[get("/seasons")]
async fn get_seasons(db: web::Data<SqlitePool>) -> impl Responder {
    let seasons = sqlx::query_as::<_, Season>("SELECT * FROM seasons")
        .fetch_all(db.get_ref())
        .await
        .unwrap();

    HttpResponse::Ok().json(seasons)
}

#[get("/seasons/{season_id}/episodes")]
async fn get_episodes_from_season(db: web::Data<SqlitePool>, season_id: web::Path<i64>) -> impl Responder {
    let episodes = sqlx::query_as::<_, Episode>("SELECT * FROM episodes WHERE season_id = ?")
        .bind(season_id.into_inner())
        .fetch_all(db.get_ref())
        .await
        .unwrap();
    HttpResponse::Ok().json(episodes)
}

#[get("/transcripts/{season_num}/{episode_num}")]
async fn get_transcript(db: web::Data<SqlitePool>, path: web::Path<(i64, i32)>) -> impl Responder {
    let (season_num, episode_num) = path.into_inner();
    println!("Fetching transcript for season {} episode {}", season_num, episode_num);
    let query = r#"
    SELECT l.id, l.season_id, l.episode_id, l.speaker_id, l.line_number, l.content
    FROM lines l
    JOIN episodes e ON l.episode_id = e.id
    JOIN seasons s ON e.season_id = s.id
    WHERE s.number = ? AND e.number = ?
    "#;
    let transcript = sqlx::query_as::<_, Line>(query)
        .bind(season_num)
        .bind(episode_num)
        .fetch_all(db.get_ref())
        .await;

    match transcript {
        Ok(transcript) => HttpResponse::Ok().json(transcript),
        Err(err) => {
            eprintln!("Error fetching transcript: {}", err);
            HttpResponse::InternalServerError().body(format!("Error fetching transcript: {}", err))
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_seasons);
    cfg.service(get_episodes_from_season);
    cfg.service(get_transcript);
}