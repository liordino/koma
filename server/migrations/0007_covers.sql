-- 0007 - covers for the library grid (M4 wave 23) + chapter page counts for detail lists.
-- Cover bytes live on disk under {storage}/covers/, fetched once from the source (allowlisted
-- outbound, API plane) and cached; the row stores the upstream URL only.

ALTER TABLE manga ADD COLUMN cover_url TEXT;
ALTER TABLE chapter ADD COLUMN pages INTEGER;
