-- 添加 is_admin 字段到 users 表
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT FALSE;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_users_is_admin ON users(is_admin);
