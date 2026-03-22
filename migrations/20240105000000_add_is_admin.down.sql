-- 删除索引
DROP INDEX IF EXISTS idx_users_is_admin;

-- 删除字段
ALTER TABLE users DROP COLUMN IF EXISTS is_admin;
