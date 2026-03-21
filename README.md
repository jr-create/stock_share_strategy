# Stock Replay - 股票复盘记录平台

一个基于 Rust 的股票复盘记录平台，帮助交易者记录每日交易计划、执行情况和复盘总结。

## ✨ 功能特性

- 📝 **每日复盘** - 记录交易计划、执行情况和复盘总结
- 📊 **股票行情** - 实时股票行情、板块数据、热门股票
- 🔍 **智能搜索** - 股票代码/名称前缀模糊搜索
- 👥 **社区分享** - 公开复盘、评论互动、点赞收藏
- 🏷️ **标签系统** - 灵活的标签分类
- 📈 **统计分析** - 个人交易数据统计

## 🛠️ 技术栈

| 类别 | 技术 |
|------|------|
| 后端框架 | Axum 0.8 |
| 数据库 | PostgreSQL 15 + SQLx 0.8 |
| 模板引擎 | Askama 0.13 |
| 前端样式 | Tailwind CSS |
| 异步运行时 | Tokio |
| 密码加密 | Argon2 |
| 数据采集 | AkShare (Python) |

## 📁 项目结构

```
stock_share_strategy/
├── apps/
│   └── web/                 # Web 应用
│       ├── src/
│       │   ├── routes/      # 路由处理
│       │   ├── templates/   # 页面模板 (Askama)
│       │   └── main.rs      # 入口文件
│       └── Cargo.toml
├── crates/
│   ├── domain/              # 领域模型
│   ├── db/                  # 数据库操作 (SQLx)
│   ├── cache/               # 缓存层
│   ├── common/              # 公共工具
│   └── analysis/            # 分析模块
├── scripts/
│   └── akshare_data.py      # 股票数据采集脚本
├── migrations/              # 数据库迁移
├── Dockerfile               # Docker 配置
├── docker-compose.yml       # Docker Compose 配置
└── zeabur.yaml              # Zeabur 部署配置
```

## 🚀 快速开始

### 环境要求

- Rust 1.75+
- PostgreSQL 15+
- Python 3.10+ (可选，用于数据采集)

### 本地开发

1. **克隆项目**
```bash
git clone https://github.com/你的用户名/stock_share_strategy.git
cd stock_share_strategy
```

2. **配置环境变量**
```bash
cp .env.example .env
# 编辑 .env 文件，配置数据库连接
```

3. **创建数据库**
```bash
# 使用 psql 或其他工具创建数据库
createdb stock_replay

# 运行迁移
psql -d stock_replay -f migrations/20240101000000_init_schema.up.sql
```

4. **启动服务**
```bash
cargo run --package stock-replay-web
```

5. **访问应用**

打开浏览器访问 http://127.0.0.1:3000

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

详见 [DEPLOY.md](./DEPLOY.md)

## 📊 数据采集

使用 AkShare 采集股票数据：

```bash
# 安装依赖
pip install -r scripts/requirements.txt

# 同步所有A股数据到数据库
python scripts/akshare_data.py --sync-db

# 获取实时行情
python scripts/akshare_data.py --realtime

# 获取板块数据
python scripts/akshare_data.py --sector

# 获取热门股票
python scripts/akshare_data.py --hot

# 采集所有基础数据
python scripts/akshare_data.py --all
```

建议每天执行1-3次数据同步（早/中/晚）。

## 🔌 API 接口

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/stocks/search?q=xxx` | 股票搜索（前缀匹配） |
| GET | `/api/stocks/{code}/quote` | 获取股票行情 |
| GET | `/api/sectors?q=xxx` | 获取板块列表 |
| GET | `/api/hot-stocks` | 获取热门股票 |
| GET | `/api/replays` | 复盘列表 |
| GET | `/api/tags/hot` | 热门标签 |

## 📄 页面说明

| 路径 | 说明 |
|------|------|
| `/` | 首页 |
| `/login` | 登录 |
| `/register` | 注册 |
| `/dashboard` | 用户控制台 |
| `/dashboard/replays/new` | 发布新复盘 |
| `/dashboard/replays` | 我的复盘 |
| `/dashboard/statistics` | 统计数据 |
| `/replays` | 复盘广场 |
| `/replays/{id}` | 复盘详情 |
| `/market` | 股票行情 |
| `/stocks/{code}` | 股票详情 |

## 🗄️ 数据库表结构

### 用户表 (users)
| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGSERIAL | 主键 |
| username | VARCHAR(32) | 用户名 |
| email | VARCHAR(128) | 邮箱 |
| password_hash | TEXT | 密码哈希 |
| created_at | TIMESTAMPTZ | 创建时间 |

### 复盘表 (replays)
| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGSERIAL | 主键 |
| user_id | BIGINT | 用户ID |
| title | VARCHAR(200) | 标题 |
| trade_date | DATE | 交易日期 |
| market_sentiment | VARCHAR(32) | 市场情绪 |
| market_view | TEXT | 大盘观点 |
| sector_focus | TEXT | 主线板块 |
| watchlist | TEXT | 关注个股 |
| plan | TEXT | 交易计划 |
| execution | TEXT | 实际执行 |
| review | TEXT | 复盘总结 |
| is_public | BOOLEAN | 是否公开 |

### 股票表 (stocks)
| 字段 | 类型 | 说明 |
|------|------|------|
| code | VARCHAR(16) | 股票代码（主键） |
| name | VARCHAR(64) | 股票名称 |
| market | VARCHAR(16) | 市场（沪/深/北） |
| sector | VARCHAR(64) | 所属板块 |
| updated_at | TIMESTAMPTZ | 更新时间 |

## 🔒 安全特性

- Argon2 密码加密
- Cookie-based Session 管理
- SQL 注入防护 (SQLx 参数化查询)
- XSS 防护 (Askama 自动转义)

## 📈 性能优化

- 异步 I/O (Tokio)
- 数据库连接池
- 文件缓存系统
- 静态资源 CDN

## 📝 开发计划

- [ ] CSRF 保护
- [ ] 收藏功能
- [ ] 用户个人主页
- [ ] 实时行情推送
- [ ] K线图表展示
- [ ] 策略回测功能

## 📜 License

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！
