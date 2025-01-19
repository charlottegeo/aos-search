import sqlite3
import os

dir = "../Seasons"

conn = sqlite3.connect('transcripts.db')
cursor = conn.cursor()

with open("schema.sql", "r") as f:
    sql = f.read()
    cursor.executescript(sql)
    conn.commit()

for season in sorted(os.listdir(dir), key=lambda x: int(x.split("S")[1])):
    season_num = int(season.split("S")[1])
    cursor.execute("INSERT INTO seasons (number) VALUES (?);", (season_num,))
    season_id = cursor.lastrowid
    conn.commit()

    for episode in sorted(os.listdir(os.path.join(dir, season)), key=lambda x: int(x.split("E")[1].split("-")[0])):
        episode_num = int(episode.split("E")[1].split("-")[0])
        title = episode.replace(".txt", "").split(" - ")[1]

        cursor.execute("INSERT INTO episodes (season_id, number, title) VALUES (?, ?, ?);", (season_id, episode_num, title))
        episode_id = cursor.lastrowid
        conn.commit()

        with open(os.path.join(dir, season, episode), "r") as f:
            line_num = 1
            for line in f.readlines():
                if ":" in line:
                    speaker, line = line.split(":", 1)
                    speaker = speaker.strip()
                    cursor.execute("INSERT INTO speakers (name) VALUES (?) ON CONFLICT(name) DO NOTHING;", (speaker,))
                    cursor.execute("SELECT id FROM speakers WHERE name = ?;", (speaker,))
                    speaker_id = cursor.fetchone()[0]
                else:
                    speaker_id = None
                line = line.strip()
                print(f"Season: {season_num}, Episode: {episode_num}, Line: {line}")
                cursor.execute("INSERT INTO lines (season_id, episode_id, speaker_id, line_number, content) VALUES (?, ?, ?, ?, ?);", (season_id, episode_id, speaker_id, line_num, line))
                line_num += 1
            conn.commit()

cursor.close()
conn.close()