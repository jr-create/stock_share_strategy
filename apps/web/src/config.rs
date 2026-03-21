use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_addr: String,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://stock_replay:stock_replay_dev@localhost:5432/stock_replay".to_string()),
            server_addr: env::var("SERVER_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:3000".to_string()),
            rust_log: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,stock_replay_web=debug,tower_http=debug".to_string()),
        })
    }
}
