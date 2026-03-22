-- 删除索引
DROP INDEX IF EXISTS idx_replays_is_pinned;

-- 删除字段
ALTER TABLE replays DROP COLUMN IF EXISTS is_pinned;
