CREATE TABLE IF NOT EXISTS seasons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    number INTEGER NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS episodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    season_id INTEGER NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    number INTEGER NOT NULL,
    title VARCHAR(255) NOT NULL,
    UNIQUE (season_id, number)
);

CREATE TABLE IF NOT EXISTS speakers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    season_id INTEGER NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    episode_id INTEGER NOT NULL REFERENCES episodes(id) ON DELETE CASCADE,
    speaker_id INTEGER REFERENCES speakers(id) ON DELETE SET NULL,
    line_number INTEGER NOT NULL,
    content TEXT NOT NULL COLLATE NOCASE,
    CONSTRAINT unique_season_episode_line UNIQUE (season_id, episode_id, line_number)
);

CREATE TABLE IF NOT EXISTS metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    line_id INTEGER NOT NULL REFERENCES lines(id),
    sentiment TEXT,
    tone TEXT,
    primary_emotion TEXT
);

CREATE INDEX IF NOT EXISTS idx_episodes_season_id ON episodes(season_id);
CREATE INDEX IF NOT EXISTS idx_lines_season_id ON lines(season_id);
CREATE INDEX IF NOT EXISTS idx_lines_episode_id ON lines(episode_id);
CREATE INDEX IF NOT EXISTS idx_lines_speaker_id ON lines(speaker_id);
CREATE INDEX IF NOT EXISTS idx_lines_content ON lines(content);
CREATE INDEX IF NOT EXISTS idx_lines_line_number ON lines(line_number);