CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT,
    updated_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users (username);

CREATE TRIGGER IF NOT EXISTS trg_insert_createdat_for_user
AFTER
INSERT
    ON users
BEGIN
UPDATE
    users
SET
    created_at = DATETIME('NOW', 'subsec')
WHERE
    ROWID = new.ROWID;

END;

CREATE TRIGGER IF NOT EXISTS trg_update_updatedat_for_user
AFTER
UPDATE
    ON users
BEGIN
UPDATE
    users
SET
    updated_at = DATETIME('NOW', 'subsec')
WHERE
    ROWID = new.ROWID;

END;

-- Note: No default users created. Use the /auth/register endpoint to create users.
-- The first user can be created via API and manually set as admin in the database if needed.
