use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use sqlx::{Sqlite, SqlitePool};
use crate::models::{Season, Episode, Line, Speaker, RandomLineQuery};

//#[get("/search/phrases")]

//#[get("/search/phrases/seasons")]

//#[get("/search/phrases/speakers")]

#[get("/random-line")]
async fn get_random_line(db: web::Data<SqlitePool>, query: web::Query<RandomLineQuery>) -> impl Responder {
    let mut sql_query = String::from("SELECT * FROM lines");
    let mut conditions = Vec::new();
    
    if let Some(season_id) = query.season {
        conditions.push("season_id = ?");
    }

    if let Some(episode_id) = query.episode {
        conditions.push("episode_id = ?");
    }

    if let Some(speaker_id) = query.speaker {
        conditions.push("speaker_id = ?");
    }

    if !conditions.is_empty() {
        sql_query.push_str(" WHERE ");
        sql_query.push_str(&conditions.join(" AND "));
    }

    sql_query.push_str(" ORDER BY RANDOM() LIMIT 1");

    let mut line = sqlx::query_as::<_, Line>(&sql_query);

    if let Some(season_id) = query.season {
        line = line.bind(season_id);
    }

    if let Some(episode_id) = query.episode {
        line = line.bind(episode_id);
    }

    if let Some(speaker_id) = query.speaker {
        line = line.bind(speaker_id);
    }

    match line.fetch_one(db.get_ref()).await {
        Ok(line) => HttpResponse::Ok().json(line),
        Err(_) => HttpResponse::NotFound().finish(),
    }

}



#[get("/transcripts/{season_num}/{episode_num}")]
async fn get_transcript(db: web::Data<SqlitePool>, path: web::Path<(i64, i32)>) -> impl Responder {
    let (season_num, episode_num) = path.into_inner();
    
    let season_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM seasons WHERE number = ?)",
        season_num
    )
    .fetch_one(db.get_ref())
    .await
    .unwrap();

    if season_exists == 0 {
        return HttpResponse::NotFound().body(format!("Season {} not found", season_num))
    }

    let episode_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM episodes WHERE number = ?)",
        episode_num
    )
    .fetch_one(db.get_ref())
    .await
    .unwrap();

    if episode_exists == 0 {
        return HttpResponse::NotFound().body(format!("Episode {} not found", episode_num))
    }

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

//#[get("/episodes/{episode_id}")]

//#[get("/seasons")] //The season table only has season numbers, wtf could go here


//#[get("/seasons/{season_id}")] is this the same as getting the episodes? What other info could there be?

#[get("/seasons/{season_id}/episodes")]
async fn get_episodes_from_season(db: web::Data<SqlitePool>, season_id: web::Path<i64>) -> impl Responder {
    let episodes = sqlx::query_as::<_, Episode>("SELECT * FROM episodes WHERE season_id = ?")
        .bind(season_id.into_inner())
        .fetch_all(db.get_ref())
        .await
        .unwrap();
    HttpResponse::Ok().json(episodes)
}



pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_episodes_from_season)
       .service(get_transcript)
       .service(get_random_line);
}