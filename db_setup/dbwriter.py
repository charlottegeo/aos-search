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

#Note: I know the parsing is weird, but because of the way the folders and files were structured, I thought it was easier to do this rather than renaming all the files

#Loops through all seasons in the directory
for season in sorted(os.listdir(dir), key=lambda x: int(x.split("S")[1])):

    #Extracts season number
    season_number = int(season.split("S")[1])

    #Insert season into database and get id
    cursor.execute("INSERT INTO seasons (number) VALUES (%s) RETURNING id;", (season_number,))
    season_id = cursor.fetchone()[0]
    conn.commit()

    #Loops through all episodes in the season
    for episode in sorted(os.listdir(os.path.join(dir, season)), key=lambda x: int(x.split("E")[1].split("-")[0])):

        #Extracts episode number and title from filename
        episode_number = int(episode.split("E")[1].split("-")[0])
        episode_title = episode.replace(".txt", "").split(" - ")[1]

        #Insert episode into database and get id
        cursor.execute("INSERT INTO episodes (season_id, number, title) VALUES (%s, %s, %s) RETURNING id;", (season_id, episode_number, episode_title))
        episode_id = cursor.fetchone()[0]
        conn.commit()

        #Loops through all lines in the episode
        with open(os.path.join(dir, season, episode), "r") as f:
                line_number = 1
                for line in f.readlines():

                    #Extracts speaker and adds them to the database
                    if ":" in line:
                        speaker, line_content = line.split(":", 1)
                        speaker = speaker.strip()
                        cursor.execute("INSERT INTO speakers (name) VALUES (%s) ON CONFLICT DO NOTHING;", (speaker,))
                        cursor.execute("SELECT id FROM speakers WHERE name = %s;", (speaker,))
                        speaker_id = cursor.fetchone()[0]
                    else:
                        #If there is no speaker, set speaker_id to None, still need to figure out what to do with those lines
                        speaker_id = None

                    #Inserts line into database
                    line_content = line_content.strip()
                    print(f"Inserting line {line_number} for episode {episode_number}, speaker {speaker_id}, content: {line_content}")
                    cursor.execute("INSERT INTO lines (season_id, episode_id, speaker_id, line_number, content) VALUES (%s, %s, %s, %s, %s);", (season_id, episode_id, speaker_id, line_number, line_content))
                    line_number += 1
                    
                conn.commit()

cursor.close()
conn.close()