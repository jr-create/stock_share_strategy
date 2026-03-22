-- Add expiry_date and is_prediction columns to replays table
ALTER TABLE replays ADD COLUMN IF NOT EXISTS expiry_date DATE;
ALTER TABLE replays ADD COLUMN IF NOT EXISTS is_prediction BOOLEAN DEFAULT FALSE;

-- Add index for expiry_date for better query performance
CREATE INDEX IF NOT EXISTS idx_replays_expiry_date ON replays(expiry_date);
CREATE INDEX IF NOT EXISTS idx_replays_is_prediction ON replays(is_prediction);
