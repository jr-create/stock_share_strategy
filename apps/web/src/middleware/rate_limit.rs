use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderValue},
    middleware::Next,
    response::Response,
};

/// 基于内存的限流器
#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    /// 创建新的限流器
    /// 
    /// # 参数
    /// - `max_requests`: 时间窗口内允许的最大请求数
    /// - `window`: 时间窗口
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }
    
    /// 检查请求是否被允许
    /// 
    /// # 返回
    /// - `bool`: 是否允许请求
    /// - `usize`: 最大请求数
    /// - `usize`: 剩余请求数
    pub async fn check(&self, key: &str) -> (bool, usize, usize) {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();
        let cutoff = now - self.window;
        
        // 获取或创建该key的请求记录
        let entry = requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        // 清理过期的请求记录
        entry.retain(|&t| t > cutoff);
        
        let remaining = self.max_requests.saturating_sub(entry.len());
        let is_allowed = entry.len() < self.max_requests;
        
        // 如果允许请求，记录当前请求时间
        if is_allowed {
            entry.push(now);
        }
        
        (is_allowed, self.max_requests, remaining)
    }
}

/// 限流中间件层
pub async fn rate_limit_layer(
    State(limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 获取客户端IP地址，优先从代理头获取
    let key = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    let (allowed, limit, remaining) = limiter.check(&key).await;
    
    if !allowed {
        tracing::warn!(
            client_ip = %key,
            limit = limit,
            "请求被限流"
        );
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    let mut response = next.run(request).await;
    
    // 添加限流相关的响应头
    response.headers_mut().insert(
        "X-RateLimit-Limit",
        HeaderValue::from_str(&limit.to_string()).unwrap(),
    );
    response.headers_mut().insert(
        "X-RateLimit-Remaining",
        HeaderValue::from_str(&remaining.to_string()).unwrap(),
    );
    
    Ok(response)
}
