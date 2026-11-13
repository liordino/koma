-- 0003 — chapter local storage directory (relative to the server storage root).
-- Path-safety contract (decision: path-safe page serving): the DB is the only key to bytes;
-- the handler canonicalizes and prefix-checks before any read.

ALTER TABLE chapter ADD COLUMN storage_dir TEXT;
