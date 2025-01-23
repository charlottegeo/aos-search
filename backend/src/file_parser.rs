use std::fs::read_dir;
use std::path::Path;
use sqlx::SqlitePool;
use tokio::{fs::File, io::{BufReader, AsyncBufReadExt}};

pub async fn process_seasons(db: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("../Seasons");
    let mut seasons = read_dir(path)?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    seasons.sort_by_key(|entry| {
        entry.file_name().to_str().unwrap().strip_prefix('S').unwrap().parse::<i32>().unwrap()
    });

    for season in seasons {
        let season_file_name = season.file_name();
        let season_num = season_file_name.to_str().unwrap().strip_prefix('S').unwrap().parse::<i32>().unwrap();

        let result = sqlx::query("INSERT INTO seasons (number) VALUES (?)")
            .bind(season_num)
            .execute(db)
            .await
            .unwrap();

        let season_id = result.last_insert_rowid();

        let mut episodes = read_dir(season.path())?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        episodes.sort_by_key(|entry| {
            entry.file_name().to_str().unwrap().strip_prefix('E').unwrap().split('-').next().unwrap().trim().parse::<i32>().unwrap()
        });

        for episode in episodes {
            let file_name = episode.file_name();
            let episode_num = match file_name.to_str().unwrap().strip_prefix('E').unwrap().split('-').next().unwrap().trim().parse::<i32>() {
                Ok(num) => num,
                Err(_) => continue,
            };
            let title = file_name.to_str().unwrap().replace(".txt", "").split(" - ").nth(1).map(|s| s.to_string()).unwrap_or_default();

            let result = sqlx::query("INSERT INTO episodes (season_id, number, title) VALUES (?, ?, ?)")
                .bind(season_id)
                .bind(episode_num)
                .bind(title)
                .execute(db)
                .await
                .unwrap();

            let episode_id = result.last_insert_rowid();

            let file = File::open(episode.path()).await.unwrap();
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
                        .execute(db)
                        .await
                        .unwrap();

                    //Gets speaker id
                    let speaker_id: (i64,) = sqlx::query_as("SELECT id FROM speakers WHERE name = ?;")
                        .bind(speaker)
                        .fetch_one(db)
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
                    .execute(db)
                    .await
                    .unwrap();

                line_num += 1;
            }
        }
    }

    Ok(())
}