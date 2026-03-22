# 部署指南

## 推荐方案：Zeabur 免费部署

Zeabur 是最适合本项目的免费部署平台：

- ✅ 完全免费额度（每月$5免费额度）
- ✅ 中国境内可访问
- ✅ 支持 Rust + PostgreSQL 一键部署
- ✅ 自动 HTTPS 域名

***

## 部署步骤

### 第一步：准备工作

1. 将代码推送到 GitHub

```bash
git init
git add .
git commit -m "Initial commit"
git branch -M main
git remote add origin https://github.com/你的用户名/stock_share_strategy.git
git push -u origin main
```

1. 注册 Zeabur 账号

- 访问 <https://zeabur.com>
- 使用 GitHub 账号登录

***

### 第二步：在 Zeabur 创建项目

1. 点击 **New Project** 创建新项目
2. 选择 **Deploy from GitHub**
3. 授权 GitHub 并选择 `stock_share_strategy` 仓库
4. Zeabur 会自动检测 Dockerfile 并开始构建

***

### 第三步：添加 PostgreSQL 数据库

1. 在项目中点击 **Add Service**
2. 选择 **Prebuilt Services** → **PostgreSQL**
3. 选择版本 `15`
4. 点击 **Deploy**

***

### 第四步：配置环境变量

在 Web 服务中添加环境变量：

| 变量名            | 值                             | 说明   |
| -------------- | ----------------------------- | ---- |
| `DATABASE_URL` | `${POSTGRES_DATABASE_URL}`    | 自动注入 |
| `RUST_LOG`     | `info,stock_replay_web=debug` | 日志级别 |
| `SERVER_ADDR`  | `0.0.0.0:3000`                | 服务地址 |

***

### 第五步：运行数据库迁移

部署成功后，需要运行数据库迁移：

1. 进入 Zeabur 控制台
2. 点击 Web 服务的 **Terminal**
3. 执行迁移命令（需要先安装 sqlx-cli）：
   1. 方案一：清空迁移记录（推荐）
```sql
-- 删除迁移记录表
DROP TABLE IF EXISTS _sqlx_migrations;

-- 然后重新部署，sqlx 会重新创建迁移记录并执行所有迁移
```

***

### 第六步：绑定域名

1. 在 Web 服务设置中点击 **Domains**
2. 添加自定义域名或使用 Zeabur 提供的免费域名
3. 自动配置 HTTPS

***

## 替代方案

### 方案二：Railway（国外访问）

1. 访问 <https://railway.app>
2. 使用 GitHub 登录
3. 新建项目 → Deploy from GitHub
4. 添加 PostgreSQL 服务
5. 配置环境变量

**缺点**：中国访问较慢，需要$5/月

### 方案三：自有服务器部署

如果有自己的服务器（如阿里云、腾讯云）：

```bash
# 1. 安装 Docker
curl -fsSL https://get.docker.com | sh

# 2. 克隆代码
git clone https://github.com/你的用户名/stock_share_strategy.git
cd stock_share_strategy

# 3. 创建 docker-compose.yml
# 4. 启动服务
docker-compose up -d
```

***

## docker-compose.yml（自有服务器）

```yaml
version: '3.8'

services:
  web:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://stock_replay:stock_replay_dev@postgres:5432/stock_replay
      - RUST_LOG=info,stock_replay_web=debug
      - SERVER_ADDR=0.0.0.0:3000
    depends_on:
      - postgres
    restart: always

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_USER=stock_replay
      - POSTGRES_PASSWORD=stock_replay_dev
      - POSTGRES_DB=stock_replay
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d
    restart: always

volumes:
  postgres_data:
```

***

## 常见问题

### Q: 数据库迁移如何执行？

A: Zeabur 的 PostgreSQL 会自动执行 `docker-entrypoint-initdb.d` 目录下的 SQL 文件。将迁移SQL复制到该目录即可。

### Q: 如何查看日志？

A: 在 Zeabur 控制台 → Web 服务 → Logs 查看

### Q: 如何更新部署？

A: 推送新代码到 GitHub，Zeabur 会自动重新部署

***

## 费用说明

| 平台      | 免费额度 | 超出费用  |
| ------- | ---- | ----- |
| Zeabur  | $5/月 | 按量计费  |
| Railway | $5试用 | $5/月起 |
| 自有服务器   | 无    | 服务器费用 |

**推荐**：Zeabur 免费额度足够个人项目使用。
