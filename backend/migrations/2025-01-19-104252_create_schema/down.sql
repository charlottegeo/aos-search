DROP INDEX IF EXISTS idx_lines_speaker_id;
DROP INDEX IF EXISTS idx_lines_episode_id;
DROP INDEX IF EXISTS idx_lines_season_id;
DROP INDEX IF EXISTS idx_episodes_season_id;

DROP TABLE IF EXISTS lines;
DROP TABLE IF EXISTS speakers;
DROP TABLE IF EXISTS episodes;
DROP TABLE IF EXISTS seasons;