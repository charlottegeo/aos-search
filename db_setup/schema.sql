CREATE TABLE seasons (
    id SERIAL PRIMARY KEY,
    number INTEGER NOT NULL
);

CREATE TABLE episodes (
    id SERIAL PRIMARY KEY,
    season_id INTEGER NOT NULL REFERENCES seasons(id),
    number INTEGER NOT NULL,
    title VARCHAR(255) NOT NULL
);

CREATE TABLE speakers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE lines (
    id SERIAL PRIMARY KEY,
    season_id INTEGER NOT NULL REFERENCES seasons(id),
    episode_id INTEGER NOT NULL REFERENCES episodes(id),
    speaker_id INTEGER REFERENCES speakers(id),
    line_number INTEGER NOT NULL,
    content TEXT NOT NULL,
    CONSTRAINT unique_season_episode_line UNIQUE (season_id, episode_id, line_number)
)