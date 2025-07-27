CREATE TABLE IF NOT EXISTS words (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  word TEXT NOT NULL,
  definition TEXT NOT NULL,
  pronunciation TEXT NOT NULL,
  created_at TEXT,
  updated_at TEXT
);

CREATE TRIGGER IF NOT EXISTS insert_createdat_for_word AFTER INSERT ON words BEGIN
UPDATE words
SET
  created_at=DATETIME('NOW', 'subsec')
WHERE
  ROWID=new.ROWID;

END;

CREATE TRIGGER IF NOT EXISTS update_updatedat_for_word AFTER
UPDATE ON words BEGIN
UPDATE words
SET
  updated_at=DATETIME('NOW', 'subsec');

END;

CREATE INDEX IF NOT EXISTS words_idx ON words (word);
