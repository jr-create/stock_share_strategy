-- 单列索引
CREATE INDEX IF NOT EXISTS idx_replays_trade_date ON replays(trade_date);
CREATE INDEX IF NOT EXISTS idx_replays_created_at ON replays(created_at);
CREATE INDEX IF NOT EXISTS idx_replays_user_id ON replays(user_id);

-- 复合索引：用户复盘列表（按创建时间倒序）
CREATE INDEX IF NOT EXISTS idx_replays_user_id_created_at ON replays(user_id, created_at DESC);

-- 复合索引：广场公开复盘列表（按创建时间倒序）
CREATE INDEX IF NOT EXISTS idx_replays_is_public_created_at ON replays(is_public, created_at DESC);
