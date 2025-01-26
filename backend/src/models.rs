use sqlx::{FromRow, Decode};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromRow, Debug, Deserialize, Serialize)]
pub struct Season {
    pub id: i64,
    pub number: i32,
}

#[derive(Clone, FromRow, Debug, Deserialize, Serialize)]
pub struct Episode {
    pub id: i64,
    pub season_id: i64,
    pub number: i32,
    pub title: String,
}

#[derive(Clone, FromRow, Debug, Deserialize, Serialize)]
pub struct Speaker {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, FromRow, Debug, Deserialize, Serialize, Decode)]
pub struct Line {
    pub id: i64,
    pub season_id: i64,
    pub episode_id: i64,
    pub speaker_id: Option<i64>,
    pub line_number: i32,
    pub content: String,
}

//This is just to handle the random line endpoint
#[derive(Deserialize)]
pub struct RandomLineQuery {
    pub season: Option<i64>,
    pub episode: Option<i64>,
    pub speaker: Option<i64>,
}