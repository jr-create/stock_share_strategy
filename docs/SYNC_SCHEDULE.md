# 定时同步股票数据方案

## 方案概述

由于 Zeabur 目前不直接支持 Cron Job，我们提供以下几种方案来实现每天自动同步股票数据：

---

## 方案一：外部定时服务调用 API（推荐）

### 实现方式

项目已内置 API 端点，可以通过外部定时服务触发同步：

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/sync/stocks` | POST | 触发股票数据同步 |
| `/api/sync/status` | GET | 查看同步状态 |

### 使用免费定时服务

#### 1. cron-job.org（推荐）

免费、稳定、支持中国访问

1. 访问 https://cron-job.org
2. 注册账号
3. 创建新任务：
   - **URL**: `https://你的域名/api/sync/stocks`
   - **请求方法**: POST
   - **执行时间**: 每天 09:00（北京时间）
   - **时区**: Asia/Shanghai

#### 2. UptimeRobot

免费监控服务，可设置定时请求

1. 访问 https://uptimerobot.com
2. 创建新监控：
   - **Monitor Type**: HTTP(s)
   - **URL**: `https://你的域名/api/sync/stocks`
   - **检查间隔**: 24小时

#### 3. GitHub Actions（免费）

在仓库中创建 `.github/workflows/sync-stocks.yml`：

```yaml
name: Sync Stock Data

on:
  schedule:
    - cron: '0 1 * * *'  # 每天 UTC 1:00 (北京时间 9:00)
  workflow_dispatch:  # 手动触发

jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - name: Trigger sync
        run: |
          curl -X POST https://你的域名/api/sync/stocks
```

---

## 方案二：内置定时器

项目已内置 Rust 定时任务模块 `scheduler.rs`，启动后会自动每天执行同步。

### 配置

在 `main.rs` 中已自动启动：

```rust
state.start_scheduler().await;
```

### 限制

- 需要服务持续运行
- 服务重启后计时器重置

---

## 方案三：手动触发

### 通过 API

```bash
curl -X POST https://你的域名/api/sync/stocks
```

### 通过命令行

```bash
python scripts/akshare_data.py --sync-db
```

---

## 推荐配置

| 场景 | 推荐方案 |
|------|---------|
| 生产环境 | cron-job.org + GitHub Actions 双保险 |
| 开发测试 | 手动触发 |
| 自有服务器 | 系统 crontab |

---

## cron-job.org 详细配置

1. **注册账号**
   - 访问 https://cron-job.org
   - 使用邮箱注册

2. **创建任务**
   - 点击 "Create cronjob"
   - 填写配置：

   | 字段 | 值 |
   |------|-----|
   | Title | 股票数据同步 |
   | URL | `https://你的域名/api/sync/stocks` |
   | Request method | POST |
   | Schedule | Daily at 09:00 |
   | Timezone | Asia/Shanghai |

3. **保存并启用**

---

## 验证同步

### 检查同步状态

```bash
curl https://你的域名/api/sync/status
```

### 查看数据库

```sql
SELECT COUNT(*) FROM stocks;
SELECT MAX(updated_at) FROM stocks;
```

---

## 注意事项

1. **首次部署**：需要手动执行一次同步，填充初始数据
2. **监控**：建议设置失败通知
3. **频率**：每天一次足够，无需频繁同步
4. **资源**：同步过程会消耗一定内存，建议在低峰期执行
