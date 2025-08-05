CREATE TABLE IF NOT EXISTS words (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  word TEXT NOT NULL UNIQUE,
  definition TEXT NOT NULL,
  pronunciation TEXT NOT NULL,
  created_at TEXT,
  updated_at TEXT
);
