-- 添加 is_admin 字段到 users 表
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT FALSE;

-- 创建一个默认管理员用户（可选，需要手动设置密码）
-- UPDATE users SET is_admin = TRUE WHERE email = 'admin@example.com';
