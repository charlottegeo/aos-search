use sqlx::FromRow;

#[derive(Clone, FromRow, Debug)]
struct Season {
    pub id: i64,
    pub number: i32,
}

#[derive(Clone, FromRow, Debug)]
struct Episode {
    pub id: i64,
    pub season_id: i64,
    pub number: i32,
    pub title: String,
}

#[derive(Clone, FromRow, Debug)]
struct Speaker {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, FromRow, Debug)]
struct Line {
    pub id: i64,
    pub season_id: i64,
    pub episode_id: i64,
    pub speaker_id: Option<i64>,
    pub line_number: i32,
    pub content: String,
}
