-- Rollback Migration
-- Drops all tables in reverse order (respecting foreign key constraints)

DROP TABLE IF EXISTS settings;
DROP TABLE IF EXISTS note_topics;
DROP TABLE IF EXISTS topics;
DROP TABLE IF EXISTS note_versions;
DROP TABLE IF EXISTS attachments;
DROP TABLE IF EXISTS note_links;
DROP TABLE IF EXISTS note_tags;
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS notes;
DROP TABLE IF EXISTS notebooks;
DROP TABLE IF EXISTS workspaces;
