-- 0006 - purge safety (M2 wave 18). Last metadata backup per category gates cleanup forever.

ALTER TABLE category ADD COLUMN last_backup_at INTEGER;
