-- Add expiry_date and is_prediction columns to replays table
ALTER TABLE replays ADD COLUMN expiry_date DATE;
ALTER TABLE replays ADD COLUMN is_prediction BOOLEAN DEFAULT FALSE;

-- Add index for expiry_date for better query performance
CREATE INDEX idx_replays_expiry_date ON replays(expiry_date);
CREATE INDEX idx_replays_is_prediction ON replays(is_prediction);
