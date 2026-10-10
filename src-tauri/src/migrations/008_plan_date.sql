ALTER TABLE trips ADD COLUMN month INTEGER;
ALTER TABLE goals ADD COLUMN month INTEGER;
ALTER TABLE goals ADD COLUMN year INTEGER;

UPDATE trips SET month = 1 WHERE month IS NULL AND when_text LIKE '%January%';
UPDATE trips SET month = 2 WHERE month IS NULL AND when_text LIKE '%February%';
UPDATE trips SET month = 3 WHERE month IS NULL AND when_text LIKE '%March%';
UPDATE trips SET month = 4 WHERE month IS NULL AND when_text LIKE '%April%';
UPDATE trips SET month = 5 WHERE month IS NULL AND (when_text LIKE '%May %' OR when_text LIKE 'May %' OR when_text LIKE '% May');
UPDATE trips SET month = 6 WHERE month IS NULL AND when_text LIKE '%June%';
UPDATE trips SET month = 7 WHERE month IS NULL AND when_text LIKE '%July%';
UPDATE trips SET month = 8 WHERE month IS NULL AND when_text LIKE '%August%';
UPDATE trips SET month = 9 WHERE month IS NULL AND when_text LIKE '%September%';
UPDATE trips SET month = 10 WHERE month IS NULL AND when_text LIKE '%October%';
UPDATE trips SET month = 11 WHERE month IS NULL AND when_text LIKE '%November%';
UPDATE trips SET month = 12 WHERE month IS NULL AND when_text LIKE '%December%';

UPDATE trips
SET
  month = CAST(strftime('%m', 'now') AS INTEGER),
  year = CAST(strftime('%Y', 'now') AS INTEGER),
  done_year = CAST(strftime('%Y', 'now') AS INTEGER),
  when_text = CASE CAST(strftime('%m', 'now') AS INTEGER)
    WHEN 1 THEN 'January ' || strftime('%Y', 'now')
    WHEN 2 THEN 'February ' || strftime('%Y', 'now')
    WHEN 3 THEN 'March ' || strftime('%Y', 'now')
    WHEN 4 THEN 'April ' || strftime('%Y', 'now')
    WHEN 5 THEN 'May ' || strftime('%Y', 'now')
    WHEN 6 THEN 'June ' || strftime('%Y', 'now')
    WHEN 7 THEN 'July ' || strftime('%Y', 'now')
    WHEN 8 THEN 'August ' || strftime('%Y', 'now')
    WHEN 9 THEN 'September ' || strftime('%Y', 'now')
    WHEN 10 THEN 'October ' || strftime('%Y', 'now')
    WHEN 11 THEN 'November ' || strftime('%Y', 'now')
    ELSE 'December ' || strftime('%Y', 'now')
  END
WHERE status = 'done'
  AND year IS NOT NULL
  AND (
    year > CAST(strftime('%Y', 'now') AS INTEGER)
    OR (
      year = CAST(strftime('%Y', 'now') AS INTEGER)
      AND month IS NOT NULL
      AND month > CAST(strftime('%m', 'now') AS INTEGER)
    )
  );
