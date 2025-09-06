CREATE TABLE IF NOT EXISTS words (
    id INTEGER PRIMARY KEY NOT NULL,
    word_type TEXT NOT NULL CHECK (
        word_type IN (
            "noun",
            "verb",
            "adjective",
            "adverb",
            "pronoun",
            "preposition",
            "conjunction",
            "interjection",
            "article"
        )
    ),
    word TEXT NOT NULL UNIQUE,
    definition TEXT NOT NULL UNIQUE,
    pronunciation TEXT NOT NULL UNIQUE,
    created_at TEXT,
    updated_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_words ON words (word);

CREATE TRIGGER IF NOT EXISTS trg_insert_createdat_for_word
AFTER
INSERT
    ON words
BEGIN
UPDATE
    words
SET
    created_at = DATETIME('NOW', 'subsec')
WHERE
    ROWID = new.ROWID;

END;

CREATE TRIGGER IF NOT EXISTS trg_update_updatedat_for_word
AFTER
UPDATE
    ON words
BEGIN
UPDATE
    words
SET
    updated_at = DATETIME('NOW', 'subsec');

END;
