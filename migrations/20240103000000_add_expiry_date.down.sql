-- Remove expiry_date and is_prediction columns from replays table
DROP INDEX IF EXISTS idx_replays_is_prediction;
DROP INDEX IF EXISTS idx_replays_expiry_date;
ALTER TABLE replays DROP COLUMN IF EXISTS is_prediction;
ALTER TABLE replays DROP COLUMN IF EXISTS expiry_date;
