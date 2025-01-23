use std::fs::read_dir;
use std::path::Path;
use sqlx::{FromRow, migrate::MigrateDatabase, Sqlite, SqlitePool};
use tokio::{fs::File, io::{BufReader, AsyncBufReadExt}};

const DB_URL: &str = "sqlite://transcript.db";

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Creates the database if it doesn't exist
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    // Connects to database (or creates database if it doesn't exist), and sets up the schema
    let db = SqlitePool::connect(DB_URL).await.unwrap();
    let sql = include_str!("../schema.sql");
    sqlx::query(&sql).execute(&db).await.unwrap();

    //Reads the Seasons directory
    let path = Path::new("../Seasons");
    let mut seasons = read_dir(path)?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    //Sorts directory by season number
    seasons.sort_by_key(|entry| {
        entry.file_name().to_str().unwrap().strip_prefix('S').unwrap().parse::<i32>().unwrap()
    });


    //Iterates through each folder in the Seasons directory
    for season_dir in seasons {

        //Gets season file name and number
        let season_file_name = season_dir.file_name();
        let season_num = season_file_name.to_str().unwrap().strip_prefix('S').unwrap().parse::<i32>().unwrap();


        //Adds season to season table in database
        let result = sqlx::query("INSERT INTO seasons (number) VALUES (?)")
            .bind(season_num)
            .execute(&db)
            .await
            .unwrap();
        
        let season_id = result.last_insert_rowid();


        //Gets each episode file in the folder and sorts by episode number
        let mut episodes = read_dir(season_dir.path())?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        episodes.sort_by_key(|entry| {
            entry.file_name().to_str().unwrap().strip_prefix('E').unwrap().split('-').next().unwrap().trim().parse::<i32>().unwrap()
        });


        //Iterates through each episode file
        for episode_file in episodes {

            //Gets episode number and title
            let file_name = episode_file.file_name();
            let episode_num = match file_name.to_str().unwrap().strip_prefix('E').unwrap().split('-').next().unwrap().trim().parse::<i32>() {
                Ok(num) => num,
                Err(e) => {
                    println!("Error parsing episode number: {:?}", e);
                    continue;
                }
            };
            let title = file_name.to_str().unwrap().replace(".txt", "").split(" - ").nth(1).map(|s| s.to_string()).unwrap_or_default();

            //Adds episode to episode table in database
            let result = sqlx::query("INSERT INTO episodes (season_id, number, title) VALUES (?, ?, ?)")
                .bind(season_id)
                .bind(episode_num)
                .bind(title)
                .execute(&db)
                .await
                .unwrap();
            let episode_id = result.last_insert_rowid();

            //Opens episode file and parses each line
            let file = File::open(episode_file.path()).await.unwrap();
            let reader = BufReader::new(file);
            let mut lines = reader.lines();
            let mut line_num = 1;
            while let Some(line_result) = lines.next_line().await.unwrap() {
                let line = line_result;

                //If there is a speaker, separate speaker and content, otherwise just content (speaker is none)
                let (speaker_id, content) = if let Some((speaker, content)) = line.split_once(':') {
                    let speaker = speaker.trim();

                    //Adds speaker to speaker table
                    sqlx::query("INSERT INTO speakers (name) VALUES (?) ON CONFLICT(name) DO NOTHING;")
                        .bind(speaker)
                        .execute(&db)
                        .await
                        .unwrap();

                    //Gets speaker id
                    let speaker_id: (i64,) = sqlx::query_as("SELECT id FROM speakers WHERE name = ?;")
                        .bind(speaker)
                        .fetch_one(&db)
                        .await
                        .unwrap();
                    
                    (Some(speaker_id.0), content.trim().to_string())
                } else {
                    (None, line.trim().to_string())
                };
                println!("Season: {}, Episode: {}, Line: {}", season_num, episode_num, content);

                //Adds line to line table
                sqlx::query("INSERT INTO lines (season_id, episode_id, speaker_id, line_number, content) VALUES (?, ?, ?, ?, ?);")
                    .bind(season_id)
                    .bind(episode_id)
                    .bind(speaker_id)
                    .bind(line_num)
                    .bind(content)
                    .execute(&db)
                    .await
                    .unwrap();

                line_num += 1;
            }
        }
    }
    Ok(())
}