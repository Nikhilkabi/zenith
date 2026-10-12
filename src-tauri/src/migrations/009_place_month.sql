ALTER TABLE places ADD COLUMN month INTEGER;

UPDATE places SET month = 1 WHERE month IS NULL AND when_text LIKE '%January%';
UPDATE places SET month = 2 WHERE month IS NULL AND when_text LIKE '%February%';
UPDATE places SET month = 3 WHERE month IS NULL AND when_text LIKE '%March%';
UPDATE places SET month = 4 WHERE month IS NULL AND when_text LIKE '%April%';
UPDATE places SET month = 5 WHERE month IS NULL AND (when_text LIKE '%May %' OR when_text LIKE 'May %' OR when_text LIKE '% May');
UPDATE places SET month = 6 WHERE month IS NULL AND when_text LIKE '%June%';
UPDATE places SET month = 7 WHERE month IS NULL AND when_text LIKE '%July%';
UPDATE places SET month = 8 WHERE month IS NULL AND when_text LIKE '%August%';
UPDATE places SET month = 9 WHERE month IS NULL AND when_text LIKE '%September%';
UPDATE places SET month = 10 WHERE month IS NULL AND when_text LIKE '%October%';
UPDATE places SET month = 11 WHERE month IS NULL AND when_text LIKE '%November%';
UPDATE places SET month = 12 WHERE month IS NULL AND when_text LIKE '%December%';

UPDATE places SET status = 'been'
WHERE status != 'been'
  AND month IS NOT NULL
  AND year IS NOT NULL
  AND (
    year < CAST(strftime('%Y', 'now') AS INTEGER)
    OR (
      year = CAST(strftime('%Y', 'now') AS INTEGER)
      AND month < CAST(strftime('%m', 'now') AS INTEGER)
    )
  );
