ALTER TABLE trips ADD COLUMN done_year INTEGER;
ALTER TABLE goals ADD COLUMN done_year INTEGER;

UPDATE trips
SET done_year = COALESCE(year, CAST(strftime('%Y', 'now') AS INTEGER))
WHERE status = 'done' AND done_year IS NULL;

UPDATE goals
SET done_year = CAST(strftime('%Y', 'now') AS INTEGER)
WHERE status = 'done' AND done_year IS NULL;
