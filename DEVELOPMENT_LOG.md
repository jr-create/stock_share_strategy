# 股票复盘记录平台 - 开发记录

## 项目概述

基于 Rust 技术栈开发的股票复盘记录平台，使用 Axum 0.8 + PostgreSQL + SQLx 0.8 + Askama 0.13 + Tailwind CSS。

---

## 一、系统安全增强

### 1.1 API 限流中间件

**文件**: `apps/web/src/middleware/rate_limit.rs`

- 滑动窗口算法实现
- 默认限制：100次/分钟/IP
- 返回 429 状态码和限流响应头
- 响应头：`X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`

### 1.2 CSRF 保护

**文件**: `apps/web/src/middleware/csrf.rs`

- 32字节安全随机令牌
- 所有 POST/PUT/DELETE 表单验证
- 验证失败返回 403

### 1.3 安全响应头

**文件**: `apps/web/src/middleware/security_headers.rs`

| 响应头 | 值 |
|--------|-----|
| X-Content-Type-Options | nosniff |
| X-Frame-Options | DENY |
| X-XSS-Protection | 1; mode=block |
| Content-Security-Policy | 允许 Tailwind CDN、jQuery、Select2 |
| Referrer-Policy | strict-origin-when-cross-origin |

### 1.4 SQL 注入防护

- 所有数据库查询使用 SQLx 参数化查询
- 无原始 SQL 拼接

---

## 二、功能优化

### 2.1 分页优化

**文件**: 
- `crates/db/src/replays.rs`
- `apps/web/migrations/001_add_replays_indexes.sql`

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

### 2.2 标题去重

**文件**: `crates/db/src/replays.rs`

- 公开复盘标题唯一性检查
- 私有复盘允许标题重复
- 重复标题返回友好错误提示

### 2.3 错误页面

**文件**: 
- `apps/web/templates/404.html`
- `apps/web/templates/500.html`

自定义错误页面，样式与整体一致。

---

## 三、导入导出功能

### 3.1 导出功能

**文件**: `apps/web/src/routes/export.rs`

| 路由 | 方法 | 说明 |
|------|------|------|
| `/my/export/json` | GET | 导出所有个人复盘为 JSON |
| `/my/export/csv` | GET | 导出所有个人复盘为 CSV |
| `/my/export/{id}/json` | GET | 导出单个复盘为 JSON |
| `/my/export/{id}/csv` | GET | 导出单个复盘为 CSV |

**导出字段**:
- title, trade_date, market_sentiment, market_view
- sector_focus, watchlist, plan, execution, review
- is_public, is_prediction, expiry_date, tags
- created_at, updated_at

### 3.2 导入功能

**文件**: `apps/web/src/routes/import.rs`

| 路由 | 方法 | 说明 |
|------|------|------|
| `/my/import` | POST | 导入 JSON/CSV 文件 |

**特性**:
- 支持 JSON 和 CSV 格式
- 文件大小限制 10MB
- 批量插入，事务处理
- 详细错误报告

### 3.3 发布页面导入

**文件**: `apps/web/templates/new_replay.html`

- 从文件导入 JSON
- 从文本框导入 JSON
- 自动填充表单字段

---

## 四、管理员功能

### 4.1 管理员登录

**文件**: 
- `apps/web/src/routes/admin.rs`
- `apps/web/templates/admin_login.html`

| 路由 | 说明 |
|------|------|
| `/admin/login` | 管理员登录页面 |

**默认账号**:
- 用户名: `admin`
- 密码: `admin@123`

### 4.2 管理员权限

**数据库**: `users.is_admin` 字段

**权限**:
- 删除任意复盘记录
- 批量导出复盘
- 批量删除复盘

### 4.3 管理员 API

**文件**: `apps/web/src/routes/export.rs`, `apps/web/src/routes/replay.rs`

| 路由 | 方法 | 说明 |
|------|------|------|
| `/admin/export/{id}/json` | GET | 导出任意单个复盘 JSON |
| `/admin/export/{id}/csv` | GET | 导出任意单个复盘 CSV |
| `/admin/export/batch/json` | POST | 批量导出 JSON |
| `/admin/export/batch/csv` | POST | 批量导出 CSV |
| `/admin/replays/batch-delete` | POST | 批量删除 |

---

## 五、点赞功能

### 5.1 点赞 API

**文件**: `apps/web/src/routes/replay.rs`

| 路由 | 方法 | 说明 |
|------|------|------|
| `/replays/{id}/like` | POST | 点赞/取消点赞 |

**响应**:
```json
{
  "liked": true,
  "like_count": 10
}
```

### 5.2 数据库层

**文件**: `crates/db/src/likes.rs`

- `create()` - 创建点赞
- `delete()` - 删除点赞
- `exists()` - 检查是否已点赞
- `count_by_replay()` - 获取点赞数
- `get_user_likes_for_replays()` - 批量获取用户点赞

---

## 六、搜索和排序

### 6.1 搜索字段选择

**文件**: `apps/web/templates/replays.html`

支持选择特定字段搜索：
- 全部字段
- 标题
- 大盘观点
- 主线板块
- 关注个股
- 交易计划
- 实际执行
- 复盘总结

### 6.2 排序功能

支持排序字段：
- 发布时间 (created_at)
- 交易日期 (trade_date)
- 点赞数 (like_count)
- 评论数 (comment_count)
- 有效期 (expiry_date)

点击切换升序/降序。

### 6.3 数据库层

**文件**: `crates/db/src/replays.rs`

- `search_with_filter()` - 带字段筛选和排序的搜索

---

## 七、主页优化

**文件**: `apps/web/templates/home.html`

### 7.1 Hero 区域

- 渐变色背景
- 醒目标题和行动按钮
- 登录/注册按钮

### 7.2 统计卡片

- 复盘记录
- 预测追踪
- 数据分析
- 社区交流

### 7.3 功能介绍

- 每日复盘
- 预测验证
- 数据分析

### 7.4 快速链接

- 热门复盘
- 预测验证

---

## 八、Session 管理

### 8.1 Session 格式

```
user_id:username:is_admin
```

### 8.2 用户信息获取

所有路由文件中的 `get_current_user()` 函数统一返回：
```rust
Option<(i64, String, bool)>  // (user_id, username, is_admin)
```

---

## 九、模板文件清单

| 文件 | 说明 |
|------|------|
| `base.html` | 基础模板 |
| `home.html` | 主页 |
| `login.html` | 登录页 |
| `register.html` | 注册页 |
| `admin_login.html` | 管理员登录 |
| `replays.html` | 复盘广场 |
| `replay_detail.html` | 复盘详情 |
| `my_replays.html` | 我的复盘 |
| `new_replay.html` | 发布复盘 |
| `edit_replay.html` | 编辑复盘 |
| `statistics.html` | 统计页面 |
| `import_result.html` | 导入结果 |
| `404.html` | 404 错误页 |
| `500.html` | 500 错误页 |

---

## 十、路由文件清单

| 文件 | 说明 |
|------|------|
| `home.rs` | 主页路由 |
| `auth.rs` | 认证路由 |
| `admin.rs` | 管理员路由 |
| `replay.rs` | 复盘路由 |
| `dashboard.rs` | 控制台路由 |
| `export.rs` | 导出路由 |
| `import.rs` | 导入路由 |
| `market.rs` | 行情路由 |
| `stocks.rs` | 股票路由 |

---

## 十一、中间件清单

| 文件 | 说明 |
|------|------|
| `rate_limit.rs` | API 限流 |
| `csrf.rs` | CSRF 保护 |
| `security_headers.rs` | 安全响应头 |

---

## 十二、部署注意事项

1. **数据库迁移**: 运行 `apps/web/migrations/` 下的 SQL 文件
2. **环境变量**: 配置数据库连接、端口等
3. **HTTPS**: 生产环境建议启用 HTTPS
4. **CSP 策略**: 根据实际 CDN 调整 Content-Security-Policy

---

## 十三、后续优化建议

1. **单元测试**: 增加核心功能测试覆盖
2. **集成测试**: 完整用户流程测试
3. **性能优化**: 大数据量分页性能
4. **监控告警**: 添加日志分析和监控
5. **备份恢复**: 数据库备份方案
