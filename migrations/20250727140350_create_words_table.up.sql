CREATE TABLE IF NOT EXISTS words (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  word_type TEXT NOT NULL CHECK (
    word_type IN ("noun", "verb", "adjective", "adverb")
  ),
  word TEXT NOT NULL UNIQUE,
  definition TEXT NOT NULL UNIQUE,
  pronunciation TEXT NOT NULL UNIQUE,
  created_at TEXT,
  updated_at TEXT
);
