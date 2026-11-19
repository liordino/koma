-- 0005 — read tracking + trash window (M2 wave 17). Retention is recorded, never auto-purged.

ALTER TABLE chapter ADD COLUMN read_marked_at INTEGER;
ALTER TABLE chapter ADD COLUMN trash_moved_at INTEGER;
ALTER TABLE chapter ADD COLUMN trash_dir TEXT;
