use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Deserialize)]
pub struct UserQuery {
    pub user_id: String,
}

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

#[derive(Clone, FromRow, Debug, Deserialize, Serialize)]
pub struct Line {
    pub id: i64,
    pub season_id: i64,
    pub episode_id: i64,
    pub speaker_id: Option<i64>,
    pub speaker_name: Option<String>,
    pub line_number: i32,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize, Type, Clone, Copy)]
#[sqlx(type_name = "TEXT")]
pub enum Sentiment {
    Positive,
    Neutral,
    Negative,
}

#[derive(Clone, FromRow, Debug, Deserialize, Serialize)]
pub struct Metadata {
    pub id: i64,
    pub line_id: i64,
    pub sentiment: Sentiment,
    pub tone: String,
    pub primary_emotion: String,
}

#[derive(Deserialize)]
pub struct SearchPhrasesQuery {
    pub phrase: Option<String>,
    pub season: Option<i64>,
    pub episode: Option<i64>,
    pub speaker: Option<i64>,
    pub context: Option<i32>,
}

#[derive(Deserialize)]
pub struct RandomLineQuery {
    pub season: Option<i64>,
    pub episode: Option<i64>,
    pub speaker: Option<i64>,
}
