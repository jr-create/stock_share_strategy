.PHONY: help dev build test clean db-up db-down db-reset migrate db-create-local

help:
	@echo "Stock Replay - 开发命令"
	@echo ""
	@echo "开发命令:"
	@echo "  make dev          - 启动开发服务器"
	@echo "  make build        - 构建生产版本"
	@echo "  make test         - 运行测试"
	@echo "  make clean        - 清理构建产物"
	@echo ""
	@echo "数据库命令 (Docker):"
	@echo "  make db-up        - 启动 PostgreSQL Docker 容器"
	@echo "  make db-down      - 停止 PostgreSQL Docker 容器"
	@echo "  make db-reset     - 重置数据库 (删除数据)"
	@echo ""
	@echo "数据库命令 (本地):"
	@echo "  make db-create-local - 创建本地数据库 (需要本地 PostgreSQL)"
	@echo ""
	@echo "迁移命令:"
	@echo "  make migrate      - 运行数据库迁移"
	@echo ""
	@echo "使用说明:"
	@echo "  1. 本地 PostgreSQL: 确保 PostgreSQL 运行中，然后运行 make db-create-local"
	@echo "  2. Docker PostgreSQL: 运行 make db-up"
	@echo "  3. 切换数据库: 编辑 .env 文件中的 DATABASE_URL"

dev:
	cargo run -p stock-replay-web

build:
	cargo build --release

test:
	cargo test --workspace

clean:
	cargo clean

db-up:
	@echo "启动 Docker PostgreSQL..."
	docker-compose up -d
	@echo "等待数据库就绪..."
	@sleep 3
	@echo "数据库已启动。如需切换到 Docker 数据库，请修改 .env 文件:"
	@echo "  DATABASE_URL=postgres://stock_replay:stock_replay_dev@localhost:5432/stock_replay"

db-down:
	docker-compose down

db-reset:
	docker-compose down -v
	docker-compose up -d

db-create-local:
	@echo "创建本地数据库..."
	@echo "请确保本地 PostgreSQL 已运行，然后执行以下 SQL:"
	@echo ""
	@echo "  psql -U postgres -c \"CREATE DATABASE stock_replay;\""
	@echo ""
	@echo "或者使用 createdb 命令:"
	@echo "  createdb -U postgres stock_replay"
	@echo ""
	@echo "创建数据库后，运行 make migrate 来初始化表结构"

migrate:
	cargo run -p stock-replay-web -- migrate
