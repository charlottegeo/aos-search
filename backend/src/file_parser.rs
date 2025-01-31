use sqlx::SqlitePool;
use std::fs::read_dir;
use std::path::Path;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

pub async fn process_seasons(
    db: &SqlitePool,
    base_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(base_path);
    let first_dir = read_dir(path)?
        .filter_map(Result::ok)
        .find(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false));

    let season_base_path = first_dir
        .map(|dir| dir.path())
        .unwrap_or_else(|| path.to_path_buf());

    let mut seasons = read_dir(&season_base_path)?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map_or(false, |name| name.starts_with('S'))
        })
        .collect::<Vec<_>>();

    println!(
        "Found seasons: {:?}",
        seasons.iter().map(|s| s.file_name()).collect::<Vec<_>>()
    );

    seasons.sort_by_key(|entry| {
        entry
            .file_name()
            .to_str()
            .and_then(|name| name.strip_prefix('S'))
            .and_then(|num| num.parse::<i32>().ok())
            .unwrap_or(0)
    });

    let mut transaction = db.begin().await?;

    for season in seasons {
        let season_num = season
            .file_name()
            .to_str()
            .and_then(|name| name.strip_prefix('S'))
            .and_then(|num| num.parse::<i32>().ok())
            .ok_or("Invalid season file name")?;

        println!("Processing season: {}", season_num);

        let season_id: i64 = sqlx::query_scalar("INSERT INTO seasons (number) VALUES (?) ON CONFLICT(number) DO UPDATE SET number = excluded.number RETURNING id")
            .bind(season_num)
            .fetch_one(&mut *transaction)
            .await?;

        let episodes = read_dir(season.path())?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map_or(false, |name| name.starts_with('E'))
            })
            .collect::<Vec<_>>();

        println!(
            "Found episodes for season {}: {:?}",
            season_num,
            episodes.iter().map(|e| e.file_name()).collect::<Vec<_>>()
        );

        for episode in episodes {
            let file_name = episode.file_name();
            let episode_num = file_name
                .to_str()
                .and_then(|name| name.strip_prefix('E'))
                .and_then(|num| num.split('-').next())
                .and_then(|num| num.trim().parse::<i32>().ok())
                .ok_or_else(|| format!("Invalid episode file name: {:?}", file_name))?;

            let title = file_name
                .to_str()
                .and_then(|name| name.strip_suffix(".txt"))
                .and_then(|name| name.split(" - ").nth(1))
                .map_or_else(|| "".to_string(), |title| title.to_string());

            println!(
                "Processing episode {} - {} with title: {}",
                episode_num,
                file_name.to_string_lossy(),
                title
            );

            let episode_id: i64 = sqlx::query_scalar("INSERT INTO episodes (season_id, number, title) VALUES (?, ?, ?) ON CONFLICT(season_id, number) DO UPDATE SET title = excluded.title RETURNING id")
                .bind(season_id)
                .bind(episode_num)
                .bind(title)
                .fetch_one(&mut *transaction)
                .await?;

            let file = File::open(episode.path()).await?;
            let mut reader = BufReader::new(file).lines();
            let mut line_num = 1;

            while let Some(line_result) = reader.next_line().await? {
                let line = line_result;

                let (speaker_id, content) = if let Some((speaker, content)) = line.split_once(':') {
                    let speaker = speaker.trim();

                    let speaker_id: i64 = sqlx::query_scalar(
                        "INSERT INTO speakers (name) VALUES (?) ON CONFLICT(name) DO UPDATE SET name = excluded.name RETURNING id"
                    )
                    .bind(speaker)
                    .fetch_one(&mut *transaction)
                    .await?;

                    (Some(speaker_id), content.trim().to_string())
                } else {
                    (None, line.trim().to_string())
                };

                sqlx::query("INSERT INTO lines (season_id, episode_id, speaker_id, line_number, content) VALUES (?, ?, ?, ?, ?)")
                    .bind(season_id)
                    .bind(episode_id)
                    .bind(speaker_id)
                    .bind(line_num)
                    .bind(content)
                    .execute(&mut *transaction)
                    .await?;

                line_num += 1;
            }
        }
    }
    transaction.commit().await?;

    println!("Finished processing all seasons and episodes.");
    Ok(())
}
