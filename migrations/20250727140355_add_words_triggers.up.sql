CREATE TRIGGER IF NOT EXISTS TG_insert_createdat_for_word AFTER INSERT ON words BEGIN
UPDATE words
SET
  created_at=DATETIME('NOW', 'subsec')
WHERE
  ROWID=new.ROWID;

END;

CREATE TRIGGER IF NOT EXISTS TG_update_updatedat_for_word AFTER
UPDATE ON words BEGIN
UPDATE words
SET
  updated_at=DATETIME('NOW', 'subsec');

END;
