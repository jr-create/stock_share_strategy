use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{interval, Duration};
use tracing::{info, error};

#[derive(Clone)]
pub struct SyncScheduler {
    db: PgPool,
    running: Arc<AtomicBool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StockInfo {
    code: String,
    name: String,
}

impl SyncScheduler {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
    
    pub async fn start(&self) {
        let db = self.db.clone();
        let running = self.running.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60 * 60 * 24));
            
            loop {
                interval.tick().await;
                
                if running.load(Ordering::Relaxed) {
                    info!("开始执行定时股票数据同步...");
                    if let Err(e) = Self::sync_stocks(&db).await {
                        error!("股票数据同步失败: {}", e);
                    }
                }
            }
        });
    }
    
    pub async fn trigger_sync(&self) -> Result<usize> {
        Self::sync_stocks(&self.db).await
    }
    
    async fn sync_stocks(db: &PgPool) -> Result<usize> {
        info!("正在从东方财富获取股票列表...");
        
        let stocks_data = match Self::fetch_from_eastmoney().await {
            Ok(data) => data,
            Err(e) => {
                info!("东方财富API失败: {}, 尝试备用数据源...", e);
                Self::fetch_from_sina().await?
            }
        };
        
        info!("获取到 {} 条股票数据", stocks_data.len());
        
        let mut count = 0;
        for (code, name) in stocks_data {
            let market = Self::determine_market(&code);
            
            let result = sqlx::query!(
                r#"
                INSERT INTO stocks (code, name, market, sector, updated_at)
                VALUES ($1, $2, $3, '', NOW())
                ON CONFLICT (code) DO UPDATE SET
                    name = EXCLUDED.name,
                    market = COALESCE(NULLIF(EXCLUDED.market, ''), stocks.market),
                    updated_at = NOW()
                "#,
                code,
                name,
                market
            )
            .execute(db)
            .await;
            
            if result.is_ok() {
                count += 1;
            }
        }
        
        info!("成功同步 {} 条股票数据", count);
        Ok(count)
    }
    
    async fn fetch_from_eastmoney() -> Result<Vec<(String, String)>> {
        let url = "https://push2.eastmoney.com/api/qt/clist/get";
        
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| anyhow::anyhow!("创建HTTP客户端失败: {}", e))?;
        
        let mut all_stocks = Vec::new();
        
        let market_configs = [
            ("m:0+t:6,m:0+t:80", "深证A股"),
            ("m:1+t:2,m:1+t:23", "上证A股"),
            ("m:0+t:81", "创业板"),
            ("m:0+t:85", "科创板"),
            ("m:0+t:83", "北交所"),
        ];
        
        for (fs, market_name) in market_configs {
            info!("正在获取 {} 股票列表...", market_name);
            
            let page_size = 500;
            let mut page = 1;
            
            loop {
                let params = [
                    ("pn", page.to_string()),
                    ("pz", page_size.to_string()),
                    ("po", "1".to_string()),
                    ("np", "1".to_string()),
                    ("fltt", "2".to_string()),
                    ("invt", "2".to_string()),
                    ("fid", "f3".to_string()),
                    ("fs", fs.to_string()),
                    ("fields", "f12,f14".to_string()),
                ];
                
                let response = client
                    .get(url)
                    .query(&params)
                    .header("Accept", "application/json, text/plain, */*")
                    .header("Referer", "https://quote.eastmoney.com/")
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("东方财富请求失败: {}", e))?;
                
                let status = response.status();
                if !status.is_success() {
                    return Err(anyhow::anyhow!("HTTP状态错误: {}", status));
                }
                
                let json: serde_json::Value = response.json().await
                    .map_err(|e| anyhow::anyhow!("解析JSON失败: {}", e))?;
                
                // 打印原始响应中的关键字段
                let data_exists = json["data"].is_object();
                let total = json["data"]["total"].as_i64().unwrap_or(0);
                let diff_exists = json["data"]["diff"].is_array();
                let diff_count = json["data"]["diff"].as_array().map(|a| a.len()).unwrap_or(0);
                
                info!("{} 第 {} 页API响应: data存在={}, total={}, diff存在={}, diff数量={}", 
                    market_name, page, data_exists, total, diff_exists, diff_count);
                
                let stocks = json["data"]["diff"].as_array();
                
                if let Some(stocks) = stocks {
                    let count = stocks.len();
                    info!("{} 第 {} 页获取 {} 条股票数据", market_name, page, count);
                    
                    for s in stocks {
                        if let (Some(code), Some(name)) = (s["f12"].as_str(), s["f14"].as_str()) {
                            if !code.is_empty() && !name.is_empty() {
                                all_stocks.push((code.to_string(), name.to_string()));
                            }
                        }
                    }
                    
                    info!("{} 第 {} 页获取 {} 条，当前总计 {} 条", market_name, page, count, all_stocks.len());
                    
                    // 继续获取直到所有数据
                    if count == 0 || (page * page_size) >= total as usize {
                        info!("{} 获取完成: count={}, page*page_size={}, total={}", 
                            market_name, count, page * page_size, total);
                        break;
                    }
                    page += 1;
                } else {
                    info!("{} 第 {} 页没有diff数据，退出", market_name, page);
                    break;
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
        
        all_stocks.sort_by(|a, b| a.0.cmp(&b.0));
        all_stocks.dedup_by(|a, b| a.0 == b.0);
        
        info!("东方财富共获取 {} 条去重后的股票数据", all_stocks.len());
        Ok(all_stocks)
    }
    
    async fn fetch_from_sina() -> Result<Vec<(String, String)>> {
        let url = "https://vip.stock.finance.sina.com.cn/quotes_service/api/json_v2.php/Market_Center.getHQNodeData";
        
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(std::time::Duration::from_secs(60))
            .build()?;
        
        let mut all_stocks = Vec::new();
        
        for page in 1..=60 {
            let page_str = page.to_string();
            let num_str = "100".to_string();
            let params: [(&str, &str); 7] = [
                ("page", &page_str),
                ("num", &num_str),
                ("sort", "symbol"),
                ("asc", "1"),
                ("node", "hs_a"),
                ("symbol", ""),
                ("_s_r_a", "page"),
            ];
            
            info!("正在请求新浪财经第 {} 页", page);
            
            let response = client
                .get(url)
                .query(&params)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("新浪请求失败: {}", e))?;
            
            let text = response.text().await?;
            
            if text.contains("null") || text.is_empty() {
                break;
            }
            
            let json: serde_json::Value = serde_json::from_str(&text)
                .map_err(|e| anyhow::anyhow!("解析新浪数据失败: {}", e))?;
            
            if let Some(arr) = json.as_array() {
                if arr.is_empty() {
                    break;
                }
                for stock in arr {
                    if let (Some(code), Some(name)) = (stock["symbol"].as_str(), stock["name"].as_str()) {
                        all_stocks.push((code.to_string(), name.to_string()));
                    }
                }
            }
        }
        
        Ok(all_stocks)
    }
    
    fn determine_market(code: &str) -> String {
        if code.starts_with('6') {
            "沪".to_string()
        } else if code.starts_with('0') || code.starts_with('3') {
            "深".to_string()
        } else if code.starts_with('8') || code.starts_with('4') {
            "北".to_string()
        } else {
            "".to_string()
        }
    }
    
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Relaxed);
    }
    
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}
