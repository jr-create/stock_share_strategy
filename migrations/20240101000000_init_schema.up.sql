-- Users table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(32) UNIQUE NOT NULL,
    email VARCHAR(128) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    bio TEXT NOT NULL DEFAULT '',
    avatar_url TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Replays table
CREATE TABLE IF NOT EXISTS replays (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(200) NOT NULL,
    trade_date DATE NOT NULL,
    market_sentiment VARCHAR(32) NOT NULL DEFAULT '',
    market_view TEXT NOT NULL DEFAULT '',
    sector_focus TEXT NOT NULL DEFAULT '',
    watchlist TEXT NOT NULL DEFAULT '',
    plan TEXT NOT NULL DEFAULT '',
    execution TEXT NOT NULL DEFAULT '',
    review TEXT NOT NULL DEFAULT '',
    is_public BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Replay tags table
CREATE TABLE IF NOT EXISTS replay_tags (
    replay_id BIGINT NOT NULL REFERENCES replays(id) ON DELETE CASCADE,
    tag VARCHAR(50) NOT NULL,
    PRIMARY KEY (replay_id, tag)
);

-- Comments table
CREATE TABLE IF NOT EXISTS comments (
    id BIGSERIAL PRIMARY KEY,
    replay_id BIGINT NOT NULL REFERENCES replays(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Likes table
CREATE TABLE IF NOT EXISTS likes (
    replay_id BIGINT NOT NULL REFERENCES replays(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (replay_id, user_id)
);

-- Stocks table
CREATE TABLE IF NOT EXISTS stocks (
    code VARCHAR(16) PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    market VARCHAR(16) NOT NULL DEFAULT '',
    sector VARCHAR(64) NOT NULL DEFAULT '',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_replays_user_id ON replays(user_id);
CREATE INDEX IF NOT EXISTS idx_replays_trade_date ON replays(trade_date DESC);
CREATE INDEX IF NOT EXISTS idx_replays_is_public ON replays(is_public);
CREATE INDEX IF NOT EXISTS idx_replay_tags_tag ON replay_tags(tag);
CREATE INDEX IF NOT EXISTS idx_comments_replay_id ON comments(replay_id);
CREATE INDEX IF NOT EXISTS idx_likes_replay_id ON likes(replay_id);
CREATE INDEX IF NOT EXISTS idx_stocks_name ON stocks(name);
CREATE INDEX IF NOT EXISTS idx_stocks_sector ON stocks(sector);

-- Updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger for replays table
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_replays_updated_at') THEN
        CREATE TRIGGER update_replays_updated_at
            BEFORE UPDATE ON replays
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
    END IF;
END $$;

-- Trigger for stocks table
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_stocks_updated_at') THEN
        CREATE TRIGGER update_stocks_updated_at
            BEFORE UPDATE ON stocks
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
    END IF;
END $$;
