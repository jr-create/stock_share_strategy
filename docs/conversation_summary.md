# 股票复盘平台开发对话记录

## 项目概述

**项目名称**: Stock Replay (股票复盘记录平台)  
**技术栈**: Rust + Axum 0.8 + PostgreSQL + SQLx 0.8 + Askama 0.13 + Tailwind CSS  
**项目路径**: `e:\All_Project\RustProject\stock_share_strategy`

---

## 一、核心功能实现

### 1.1 用户认证系统

**文件**: `apps/web/src/routes/auth.rs`

- 用户注册/登录
- Argon2 密码哈希
- Cookie-based Session (格式: `user_id:username:is_admin`)
- 登录后跳转到主页或来源页面

### 1.2 复盘管理

**文件**: `apps/web/src/routes/dashboard.rs`, `crates/db/src/replays.rs`

- CRUD 操作
- 标题去重（公开复盘不允许重复标题）
- 分页查询
- 搜索功能
- 标签管理

### 1.3 评论系统

**文件**: `crates/db/src/comments.rs`

- 评论 CRUD
- 评论计数

### 1.4 点赞系统

**文件**: `crates/db/src/likes.rs`

- 点赞/取消点赞
- 点赞计数
- 用户点赞状态查询

---

## 二、安全功能

### 2.1 API 限流中间件

**文件**: `apps/web/src/middleware/rate_limit.rs`

- 滑动窗口算法实现
- 默认限制：100次/分钟/IP
- 返回 429 状态码和限流响应头
- 响应头：`X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`

### 2.2 CSRF 保护

**文件**: `apps/web/src/middleware/csrf.rs`

- 32字节安全随机令牌
- 所有 POST/PUT/DELETE 表单验证
- 验证失败返回 403

### 2.3 安全响应头

**文件**: `apps/web/src/middleware/security_headers.rs`

| 响应头 | 值 |
|--------|-----|
| X-Content-Type-Options | nosniff |
| X-Frame-Options | DENY |
| X-XSS-Protection | 1; mode=block |
| Content-Security-Policy | 允许 Tailwind CDN、jQuery、Select2 |
| Referrer-Policy | strict-origin-when-cross-origin |

### 2.4 SQL 注入防护

- 所有数据库查询使用 SQLx 参数化查询
- 无原始 SQL 拼接

---

## 三、功能优化

### 3.1 分页优化

**数据库索引**:
```sql
CREATE INDEX idx_replays_trade_date ON replays(trade_date);
CREATE INDEX idx_replays_created_at ON replays(created_at);
CREATE INDEX idx_replays_user_id ON replays(user_id);
CREATE INDEX idx_replays_user_id_created_at ON replays(user_id, created_at DESC);
CREATE INDEX idx_replays_is_public_created_at ON replays(is_public, created_at DESC);
```

**分页元数据**:
- `total` - 总记录数
- `total_pages` - 总页数
- `current_page` - 当前页
- `per_page` - 每页大小

### 3.2 搜索功能

**搜索字段选择**:
- 全部字段
- 标题
- 大盘观点
- 主线板块
- 关注个股
- 交易计划
- 实际执行
- 复盘总结

### 3.3 排序功能

**排序字段**:
- 发布时间（默认）
- 交易日期
- 点赞数
- 评论数
- 有效期

---

## 四、导入导出功能

### 4.1 导出功能

**文件**: `apps/web/src/routes/export.rs`

| 路由 | 方法 | 说明 |
|------|------|------|
| `/my/export/json` | GET | 导出所有复盘为 JSON |
| `/my/export/csv` | GET | 导出所有复盘为 CSV |
| `/my/export/{id}/json` | GET | 导出单个复盘为 JSON |
| `/my/export/{id}/csv` | GET | 导出单个复盘为 CSV |

**导出字段**:
- title, trade_date, market_sentiment, market_view
- sector_focus, watchlist, plan, execution, review
- is_public, is_prediction, expiry_date, tags
- created_at, updated_at

### 4.2 导入功能

**文件**: `apps/web/src/routes/import.rs`

| 路由 | 方法 | 说明 |
|------|------|------|
| `/my/import` | POST | 导入 JSON/CSV 文件 |

**特性**:
- 支持 JSON 和 CSV 格式
- 文件大小限制 10MB
- 批量导入，事务处理
- 详细错误报告

### 4.3 发布页面导入

**文件**: `apps/web/templates/new_replay.html`

- 从文件导入 JSON
- 从文本框粘贴 JSON
- 自动填充表单字段

---

## 五、管理员功能

### 5.1 管理员登录

**路由**: `/admin/login`

**默认账号**:
- 用户名: `admin`
- 密码: `admin@123`

### 5.2 管理员权限

- 删除任意复盘记录
- 批量导出复盘
- 批量删除复盘

### 5.3 管理员 API

| 路由 | 方法 | 说明 |
|------|------|------|
| `/admin/export/{id}/json` | GET | 导出任意单个复盘 JSON |
| `/admin/export/{id}/csv` | GET | 导出任意单个复盘 CSV |
| `/admin/export/batch/json` | POST | 批量导出 JSON |
| `/admin/export/batch/csv` | POST | 批量导出 CSV |
| `/admin/replays/batch-delete` | POST | 批量删除 |

---

## 六、点赞功能

### 6.1 点赞 API

| 路由 | 方法 | 说明 |
|------|------|------|
| `/replays/{id}/like` | POST | 点赞/取消点赞 |

**响应**:
```json
{
  "liked": true,
  "like_count": 5
}
```

### 6.2 点赞状态

- 已点赞显示红色
- 实时更新点赞数
- 需要登录才能点赞

---

## 七、页面功能

### 7.1 主页优化

**文件**: `apps/web/templates/home.html`

- 渐变色 Hero 区域
- 统计卡片展示
- 功能介绍卡片
- 快速链接入口
- CTA 引导区域

### 7.2 复盘广场

**文件**: `apps/web/templates/replays.html`

- 搜索框 + 字段选择
- 排序按钮
- 点赞按钮
- 分页导航
- 管理员批量操作

### 7.3 复盘详情

**文件**: `apps/web/templates/replay_detail.html`

- 完整内容展示
- 点赞按钮
- 评论列表
- 编辑/删除按钮（权限控制）
- 导出按钮

### 7.4 我的复盘

**文件**: `apps/web/templates/my_replays.html`

- 复盘列表
- 导入/导出按钮
- 分页导航

---

## 八、数据库结构

### 8.1 用户表 (users)

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    bio TEXT,
    avatar_url VARCHAR(500),
    created_at TIMESTAMP DEFAULT NOW(),
    is_admin BOOLEAN DEFAULT FALSE
);
```

### 8.2 复盘表 (replays)

```sql
CREATE TABLE replays (
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    title VARCHAR(255) NOT NULL,
    trade_date DATE NOT NULL,
    market_sentiment TEXT,
    market_view TEXT,
    sector_focus TEXT,
    watchlist TEXT,
    plan TEXT,
    execution TEXT,
    review TEXT,
    is_public BOOLEAN DEFAULT FALSE,
    is_prediction BOOLEAN DEFAULT FALSE,
    expiry_date DATE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

### 8.3 点赞表 (likes)

```sql
CREATE TABLE likes (
    id SERIAL PRIMARY KEY,
    replay_id BIGINT NOT NULL REFERENCES replays(id),
    user_id BIGINT NOT NULL REFERENCES users(id),
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(replay_id, user_id)
);
```

### 8.4 评论表 (comments)

```sql
CREATE TABLE comments (
    id SERIAL PRIMARY KEY,
    replay_id BIGINT NOT NULL REFERENCES replays(id),
    user_id BIGINT NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
```

### 8.5 标签表 (tags)

```sql
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE
);

CREATE TABLE replay_tags (
    replay_id BIGINT NOT NULL REFERENCES replays(id),
    tag_id BIGINT NOT NULL REFERENCES tags(id),
    PRIMARY KEY (replay_id, tag_id)
);
```

---

## 九、Session 格式

```
user_id:username:is_admin
```

示例:
- 普通用户: `123:张三:0`
- 管理员: `1:admin:1`

---

## 十、API 路由汇总

### 认证
- `GET /login` - 登录页面
- `POST /login` - 登录处理
- `GET /register` - 注册页面
- `POST /register` - 注册处理
- `GET /logout` - 登出

### 复盘
- `GET /replays` - 复盘广场
- `GET /replays/{id}` - 复盘详情
- `POST /replays/{id}/like` - 点赞
- `POST /replays/{id}/comments` - 发表评论
- `GET /replays/{id}/delete` - 删除复盘

### 控制台
- `GET /dashboard` - 控制台首页
- `GET /dashboard/my-replays` - 我的复盘
- `GET /dashboard/replays/new` - 发布复盘页面
- `POST /dashboard/replays/new` - 发布复盘
- `GET /dashboard/replays/{id}/edit` - 编辑页面
- `POST /dashboard/replays/{id}/edit` - 更新复盘
- `GET /dashboard/statistics` - 统计页面

### 导入导出
- `GET /my/export/json` - 导出全部 JSON
- `GET /my/export/csv` - 导出全部 CSV
- `GET /my/export/{id}/json` - 导出单个 JSON
- `GET /my/export/{id}/csv` - 导出单个 CSV
- `POST /my/import` - 导入文件

### 管理员
- `GET /admin/login` - 管理员登录页面
- `POST /admin/login` - 管理员登录
- `GET /admin/export/{id}/json` - 导出任意复盘 JSON
- `GET /admin/export/{id}/csv` - 导出任意复盘 CSV
- `POST /admin/export/batch/json` - 批量导出 JSON
- `POST /admin/export/batch/csv` - 批量导出 CSV
- `POST /admin/replays/batch-delete` - 批量删除

---

## 十一、文件结构

```
stock_share_strategy/
├── apps/
│   └── web/
│       ├── src/
│       │   ├── routes/
│       │   │   ├── auth.rs
│       │   │   ├── dashboard.rs
│       │   │   ├── export.rs
│       │   │   ├── home.rs
│       │   │   ├── import.rs
│       │   │   ├── market.rs
│       │   │   ├── mod.rs
│       │   │   ├── replay.rs
│       │   │   └── stocks.rs
│       │   ├── middleware/
│       │   │   ├── csrf.rs
│       │   │   ├── mod.rs
│       │   │   ├── rate_limit.rs
│       │   │   └── security_headers.rs
│       │   ├── config.rs
│       │   ├── error.rs
│       │   ├── main.rs
│       │   └── state.rs
│       ├── templates/
│       │   ├── base.html
│       │   ├── home.html
│       │   ├── login.html
│       │   ├── register.html
│       │   ├── admin_login.html
│       │   ├── replays.html
│       │   ├── replay_detail.html
│       │   ├── my_replays.html
│       │   ├── new_replay.html
│       │   ├── edit_replay.html
│       │   ├── statistics.html
│       │   ├── import_result.html
│       │   ├── 404.html
│       │   └── 500.html
│       └── migrations/
│           └── 001_add_replays_indexes.sql
├── crates/
│   ├── db/
│   │   └── src/
│   │       ├── comments.rs
│   │       ├── lib.rs
│   │       ├── likes.rs
│   │       ├── replays.rs
│   │       └── users.rs
│   ├── domain/
│   │   └── src/
│   │       ├── comment.rs
│   │       ├── like.rs
│   │       ├── lib.rs
│   │       ├── replay.rs
│   │       └── user.rs
│   └── common/
│       └── src/
│           └── password.rs
└── Cargo.toml
```

---

## 十二、部署注意事项

1. **数据库迁移**: 运行 `apps/web/migrations/` 下的 SQL 文件
2. **环境变量**: 配置数据库连接、端口等
3. **HTTPS**: 生产环境建议启用 HTTPS
4. **CSP 策略**: 根据实际 CDN 调整 Content-Security-Policy
5. **管理员设置**: 首次登录 `/admin/login` 会自动创建管理员账户

---

## 十三、待优化项

1. 单元测试和集成测试
2. 性能基准测试
3. 日志分析工具
4. 监控告警机制
5. 数据库备份恢复方案
6. 生产环境部署脚本
