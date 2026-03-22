# Stock Replay - 股票复盘记录平台

一个基于 Rust 的股票复盘记录平台，帮助交易者记录每日交易计划、执行情况和复盘总结。

## ✨ 功能特性

### 核心功能

- 📝 **每日复盘** - 记录交易计划、执行情况和复盘总结
- 📊 **股票行情** - 实时股票行情、板块数据、热门股票
- 🔍 **智能搜索** - 股票代码/名称前缀模糊搜索，支持多字段搜索
- 👥 **社区分享** - 公开复盘、评论互动、点赞收藏
- 🏷️ **标签系统** - 灵活的标签分类
- 📈 **统计分析** - 个人交易数据统计

### 安全特性

- 🔐 **验证码保护** - 登录/注册验证码防止机器人
- 🛡️ **API 限流** - 滑动窗口算法，100次/分钟/IP
- 🔒 **CSRF 保护** - 表单提交安全验证
- 📋 **安全响应头** - CSP、X-Frame-Options 等

### 管理功能

- 📌 **置顶功能** - 管理员可置顶重要复盘
- 📤 **批量导出** - 支持 JSON/CSV 格式导出
- 🗑️ **批量删除** - 管理员批量管理复盘
- 👨‍💼 **管理员登录** - 专门的管理员入口

### 用户体验

- 🌓 **主题切换** - 支持亮色/暗色主题
- 📱 **响应式设计** - 适配桌面和移动设备
- ⚡ **高性能** - 异步 I/O，连接池优化

## 🛠️ 技术栈

| 类别    | 技术                       |
| ----- | ------------------------ |
| 后端框架  | Axum 0.8                 |
| 数据库   | PostgreSQL 15 + SQLx 0.8 |
| 模板引擎  | Askama 0.13              |
| 前端样式  | Tailwind CSS (CDN)       |
| 异步运行时 | Tokio                    |
| 密码加密  | Argon2                   |
| 数据采集  | 东方财富 API / 新浪财经 API      |

## 📁 项目结构

```
stock_share_strategy/
├── apps/
│   └── web/                 # Web 应用
│       ├── src/
│       │   ├── routes/      # 路由处理
│       │   │   ├── auth.rs      # 认证路由
│       │   │   ├── admin.rs     # 管理员路由
│       │   │   ├── replay.rs    # 复盘路由
│       │   │   ├── export.rs    # 导出路由
│       │   │   └── ...
│       │   ├── middleware/  # 中间件
│       │   │   ├── rate_limit.rs    # API 限流
│       │   │   ├── csrf.rs          # CSRF 保护
│       │   │   └── security_headers.rs
│       │   ├── captcha.rs   # 验证码模块
│       │   ├── scheduler.rs # 股票数据同步
│       │   └── main.rs      # 入口文件
│       └── templates/       # 页面模板 (Askama)
├── crates/
│   ├── domain/              # 领域模型
│   ├── db/                  # 数据库操作 (SQLx)
│   ├── cache/               # 缓存层
│   ├── common/              # 公共工具
│   └── analysis/            # 分析模块
├── migrations/              # 数据库迁移 (自动执行)
├── Dockerfile               # Docker 配置
├── docker-compose.yml       # Docker Compose 配置
└── zeabur.yaml              # Zeabur 部署配置
```

## 🚀 快速开始

### 环境要求

- Rust 1.75+
- PostgreSQL 15+

### 本地开发

1. **克隆项目**

```bash
git clone https://github.com/你的用户名/stock_share_strategy.git
cd stock_share_strategy
```

1. **配置环境变量**

```bash
# 设置数据库连接
export DATABASE_URL="postgres://postgres:password@localhost/stock_replay"
```

1. **创建数据库**

```bash
createdb stock_replay
```

1. **启动服务**

```bash
cargo run --package stock-replay-web
```

1. **访问应用**

打开浏览器访问 <http://127.0.0.1:3000>

### Docker 部署

```bash
# 构建并启动
docker-compose up -d

# 查看日志
docker-compose logs -f web

# 停止服务
docker-compose down
```

### Zeabur 云部署

1. Fork 本项目到你的 GitHub
2. 在 Zeabur 中导入项目
3. 添加 PostgreSQL 服务
4. 配置环境变量 `DATABASE_URL`
5. 部署完成

**数据库迁移**: 应用启动时自动执行迁移，无需手动操作。

## 🔌 API 接口

### 公开 API

| 方法   | 路径                         | 说明         |
| ---- | -------------------------- | ---------- |
| GET  | `/api/stocks/search?q=xxx` | 股票搜索（前缀匹配） |
| GET  | `/api/sectors?q=xxx`       | 获取板块列表     |
| GET  | `/api/hot-stocks`          | 获取热门股票     |
| GET  | `/api/tags/hot`            | 热门标签       |
| POST | `/api/sync/stocks`         | 触发股票数据同步   |

### 认证 API

| 方法   | 路径          | 说明    |
| ---- | ----------- | ----- |
| POST | `/login`    | 用户登录  |
| POST | `/register` | 用户注册  |
| GET  | `/logout`   | 退出登录  |
| GET  | `/captcha`  | 刷新验证码 |

### 复盘 API

| 方法   | 路径                       | 说明      |
| ---- | ------------------------ | ------- |
| GET  | `/replays`               | 复盘广场列表  |
| GET  | `/replays/{id}`          | 复盘详情    |
| POST | `/replays/{id}/like`     | 点赞/取消点赞 |
| POST | `/dashboard/replays/new` | 发布新复盘   |
| GET  | `/dashboard/my-replays`  | 我的复盘列表  |

### 管理员 API

| 方法   | 路径                            | 说明        |
| ---- | ----------------------------- | --------- |
| GET  | `/admin/login`                | 管理员登录页    |
| POST | `/admin/login`                | 管理员登录     |
| POST | `/admin/replays/{id}/pin`     | 置顶/取消置顶   |
| POST | `/admin/replays/batch-delete` | 批量删除      |
| GET  | `/admin/export/json`          | 导出全部 JSON |
| GET  | `/admin/export/csv`           | 导出全部 CSV  |

### 导出 API

| 方法  | 路径                     | 说明          |
| --- | ---------------------- | ----------- |
| GET | `/my/export/json`      | 导出我的复盘 JSON |
| GET | `/my/export/csv`       | 导出我的复盘 CSV  |
| GET | `/my/export/{id}/json` | 导出单个复盘 JSON |
| GET | `/my/export/{id}/csv`  | 导出单个复盘 CSV  |

## 📄 页面说明

| 路径                       | 说明       |
| ------------------------ | -------- |
| `/`                      | 首页       |
| `/login`                 | 登录（带验证码） |
| `/register`              | 注册（带验证码） |
| `/admin/login`           | 管理员登录    |
| `/dashboard`             | 用户控制台    |
| `/dashboard/replays/new` | 发布新复盘    |
| `/dashboard/my-replays`  | 我的复盘     |
| `/dashboard/statistics`  | 统计数据     |
| `/replays`               | 复盘广场     |
| `/replays/{id}`          | 复盘详情     |
| `/market`                | 股票行情     |

## 🗄️ 数据库表结构

### 用户表 (users)

| 字段             | 类型           | 说明    |
| -------------- | ------------ | ----- |
| id             | BIGSERIAL    | 主键    |
| username       | VARCHAR(32)  | 用户名   |
| email          | VARCHAR(128) | 邮箱    |
| password\_hash | TEXT         | 密码哈希  |
| is\_admin      | BOOLEAN      | 是否管理员 |
| created\_at    | TIMESTAMPTZ  | 创建时间  |

### 复盘表 (replays)

| 字段                | 类型           | 说明   |
| ----------------- | ------------ | ---- |
| id                | BIGSERIAL    | 主键   |
| user\_id          | BIGINT       | 用户ID |
| title             | VARCHAR(200) | 标题   |
| trade\_date       | DATE         | 交易日期 |
| market\_sentiment | VARCHAR(32)  | 市场情绪 |
| market\_view      | TEXT         | 大盘观点 |
| sector\_focus     | TEXT         | 主线板块 |
| watchlist         | TEXT         | 关注个股 |
| plan              | TEXT         | 交易计划 |
| execution         | TEXT         | 实际执行 |
| review            | TEXT         | 复盘总结 |
| is\_public        | BOOLEAN      | 是否公开 |
| is\_prediction    | BOOLEAN      | 是否预测 |
| is\_pinned        | BOOLEAN      | 是否置顶 |
| expiry\_date      | DATE         | 有效期  |
| created\_at       | TIMESTAMPTZ  | 创建时间 |
| updated\_at       | TIMESTAMPTZ  | 更新时间 |

### 股票表 (stocks)

| 字段     | 类型          | 说明        |
| ------ | ----------- | --------- |
| code   | VARCHAR(16) | 股票代码（主键）  |
| name   | VARCHAR(64) | 股票名称      |
| market | VARCHAR(16) | 市场（沪/深/北） |
| sector | VARCHAR(64) | 所属板块      |

### 点赞表 (likes)

| 字段          | 类型          | 说明   |
| ----------- | ----------- | ---- |
| replay\_id  | BIGINT      | 复盘ID |
| user\_id    | BIGINT      | 用户ID |
| created\_at | TIMESTAMPTZ | 创建时间 |

### 评论表 (comments)

| 字段          | 类型          | 说明   |
| ----------- | ----------- | ---- |
| id          | BIGSERIAL   | 主键   |
| replay\_id  | BIGINT      | 复盘ID |
| user\_id    | BIGINT      | 用户ID |
| content     | TEXT        | 评论内容 |
| created\_at | TIMESTAMPTZ | 创建时间 |

## 🔒 安全特性

| 特性             | 说明                                   |
| -------------- | ------------------------------------ |
| Argon2 密码加密    | 高强度密码哈希                              |
| Cookie Session | 格式: `user_id:username:is_admin`      |
| API 限流         | 100次/分钟/IP                           |
| CSRF 保护        | 表单令牌验证                               |
| 安全响应头          | CSP、X-Frame-Options、X-XSS-Protection |
| SQL 注入防护       | SQLx 参数化查询                           |
| XSS 防护         | Askama 自动转义                          |

## 📈 性能优化

- 异步 I/O (Tokio)
- 数据库连接池
- 数据库索引优化
- 静态资源 CDN (Tailwind、jQuery、Select2)

## 👤 默认账户

### 管理员账户

- **用户名**: admin
- **密码**: xxx
- **登录地址**: `/admin/login`

> 首次登录会自动创建管理员账户

## 📝 开发计划

- [x] CSRF 保护
- [x] 点赞功能
- [x] 主题切换
- [x] 验证码保护
- [x] 管理员置顶
- [x] 批量导出/删除
- [ ] 收藏功能
- [ ] 用户个人主页
- [ ] 实时行情推送
- [ ] K线图表展示
- [ ] 策略回测功能

## 📜 License

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！
