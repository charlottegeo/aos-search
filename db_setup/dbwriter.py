import dotenv
import os
import psycopg2

dotenv.load_dotenv()

USERNAME = os.getenv("USERNAME")
PASSWORD = os.getenv("PASSWORD")
DATABASE_NAME = os.getenv("DATABASE_NAME")
DATABASE_URL = os.getenv("DATABASE_URL")

dir = "Seasons"

conn = psycopg2.connect(dbname=DATABASE_NAME, user=USERNAME, password=PASSWORD, host=DATABASE_URL)
cursor = conn.cursor()

season_dict = {}

for season in sorted(os.listdir(dir), key=lambda x: int(x.split("S")[1])):
    season_number = int(season.split("S")[1])
    print(f"Processing season: {season_number}")

    cursor.execute("INSERT INTO seasons (number) VALUES (%s) RETURNING id;", (season_number,))
    season_id = cursor.fetchone()[0]
    print(f"Inserted season {season_number} with id {season_id}")
    conn.commit()

    season_dict[season_number] = season_id

    for episode in sorted(os.listdir(os.path.join(dir, season)), key=lambda x: int(x.split("E")[1].split("-")[0])):
        episode_number = int(episode.split("E")[1].split("-")[0])
        episode_title = episode.replace(".txt", "").split(" - ")[1]
        print(f"Processing episode: {episode_number} - {episode_title}")

        cursor.execute("INSERT INTO episodes (season_id, number, title) VALUES (%s, %s, %s) RETURNING id;", (season_id, episode_number, episode_title))
        episode_id = cursor.fetchone()[0]
        print(f"Inserted episode {episode_number} with id {episode_id}")
        conn.commit()

        with open(os.path.join(dir, season, episode), "r") as f:
                line_number = 1
                for line in f.readlines():
                    if ":" in line:
                        speaker, line_content = line.split(":", 1)
                        speaker = speaker.strip()
                        cursor.execute("INSERT INTO speakers (name) VALUES (%s) ON CONFLICT DO NOTHING;", (speaker,))
                        cursor.execute("SELECT id FROM speakers WHERE name = %s;", (speaker,))
                        speaker_id = cursor.fetchone()[0]
                    else:
                        speaker_id = None

                    line_content = line_content.strip()
                    print(f"Inserting line {line_number} for episode {episode_number}, speaker {speaker_id}, content: {line_content}")

                    cursor.execute("INSERT INTO lines (season_id, episode_id, speaker_id, line_number, content) VALUES (%s, %s, %s, %s, %s);", (season_id, episode_id, speaker_id, line_number, line_content))
                    line_number += 1
                conn.commit()
                print(f"Added episode {episode_number} for season {season_number}")

cursor.close()
conn.close()