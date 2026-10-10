UPDATE places
SET status = 'dream'
WHERE deleted_at IS NULL
  AND status = 'been'
  AND id IN (
    SELECT s.place_id
    FROM trip_stops s
    JOIN trips t ON t.id = s.trip_id AND t.deleted_at IS NULL AND t.status != 'done'
  )
  AND id NOT IN (
    SELECT s.place_id
    FROM trip_stops s
    JOIN trips t ON t.id = s.trip_id AND t.deleted_at IS NULL AND t.status = 'done'
  );
