-- 添加 is_pinned 字段到 replays 表
ALTER TABLE replays ADD COLUMN IF NOT EXISTS is_pinned BOOLEAN NOT NULL DEFAULT FALSE;

-- 添加 is_prediction 字段（如果不存在）
ALTER TABLE replays ADD COLUMN IF NOT EXISTS is_prediction BOOLEAN NOT NULL DEFAULT FALSE;

-- 创建索引以优化置顶查询
CREATE INDEX IF NOT EXISTS idx_replays_is_pinned ON replays(is_pinned);
