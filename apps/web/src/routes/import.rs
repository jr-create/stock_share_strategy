use axum::{
    extract::State,
    routing::post,
    Router,
    response::{Html, IntoResponse, Redirect, Response},
};
use askama::Template;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use crate::state::AppState;
use crate::error::AppError;

const SESSION_COOKIE_NAME: &str = "session";
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

fn get_current_user(cookies: &Cookies) -> Option<(i64, String, bool)> {
    cookies.get(SESSION_COOKIE_NAME).and_then(|c| {
        let value = c.value();
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() >= 2 {
            let is_admin = parts.get(2).map(|&s| s == "1").unwrap_or(false);
            Some((parts[0].parse().ok()?, parts[1].to_string(), is_admin))
        } else {
            None
        }
    })
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ReplayImport {
    pub title: String,
    pub trade_date: String,
    pub market_sentiment: Option<String>,
    pub market_view: Option<String>,
    pub sector_focus: Option<String>,
    pub watchlist: Option<String>,
    pub plan: Option<String>,
    pub execution: Option<String>,
    pub review: Option<String>,
    pub is_public: Option<bool>,
    pub is_prediction: Option<bool>,
    pub expiry_date: Option<String>,
    pub tags: Option<String>,
}

impl ReplayImport {
    fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("标题不能为空".to_string());
        }
        
        chrono::NaiveDate::parse_from_str(&self.trade_date, "%Y-%m-%d")
            .map_err(|_| format!("日期格式错误: {}, 正确格式为 YYYY-MM-DD", self.trade_date))?;
        
        if let Some(ref expiry) = self.expiry_date {
            if !expiry.trim().is_empty() {
                chrono::NaiveDate::parse_from_str(expiry, "%Y-%m-%d")
                    .map_err(|_| format!("有效期日期格式错误: {}, 正确格式为 YYYY-MM-DD", expiry))?;
            }
        }
        
        Ok(())
    }
    
    fn to_new_replay(&self) -> Result<domain::NewReplay, String> {
        self.validate()?;
        
        let trade_date = chrono::NaiveDate::parse_from_str(&self.trade_date, "%Y-%m-%d")
            .map_err(|e| e.to_string())?;
        
        let expiry_date = self.expiry_date
            .as_ref()
            .filter(|d| !d.trim().is_empty())
            .map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d"))
            .transpose()
            .map_err(|e| e.to_string())?;
        
        let tags: Vec<String> = self.tags
            .as_ref()
            .map(|t| t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default();
        
        Ok(domain::NewReplay {
            title: self.title.trim().to_string(),
            trade_date,
            market_sentiment: self.market_sentiment.clone().unwrap_or_default(),
            market_view: self.market_view.clone().unwrap_or_default(),
            sector_focus: self.sector_focus.clone().unwrap_or_default(),
            watchlist: self.watchlist.clone().unwrap_or_default(),
            plan: self.plan.clone().unwrap_or_default(),
            execution: self.execution.clone().unwrap_or_default(),
            review: self.review.clone().unwrap_or_default(),
            is_public: self.is_public.unwrap_or(false),
            expiry_date,
            is_prediction: self.is_prediction.unwrap_or(false),
            tags,
        })
    }
}

#[derive(Debug, Serialize)]
struct ImportError {
    row: usize,
    title: String,
    error: String,
}

#[derive(Template)]
#[template(path = "import_result.html")]
struct ImportResultTemplate {
    logged_in: bool,
    username: String,
    success_count: usize,
    failed_count: usize,
    errors: Vec<ImportError>,
}

enum ImportResponse {
    Html(Html<String>),
    Redirect(Redirect),
}

impl IntoResponse for ImportResponse {
    fn into_response(self) -> Response {
        match self {
            ImportResponse::Html(html) => html.into_response(),
            ImportResponse::Redirect(redirect) => redirect.into_response(),
        }
    }
}

fn detect_format(content: &str) -> Result<FileFormat, AppError> {
    let trimmed = content.trim();
    
    if trimmed.starts_with('[') || trimmed.starts_with('{') {
        return Ok(FileFormat::Json);
    }
    
    if trimmed.contains(',') && trimmed.split('\n').count() > 0 {
        let first_line = trimmed.lines().next().unwrap_or("");
        if first_line.contains("title") || first_line.contains("trade_date") {
            return Ok(FileFormat::Csv);
        }
    }
    
    Err(AppError::BadRequest("无法识别文件格式，请上传 JSON 或 CSV 文件".to_string()))
}

#[derive(Debug, Clone, Copy)]
enum FileFormat {
    Json,
    Csv,
}

fn parse_json(content: &str) -> Result<Vec<ReplayImport>, AppError> {
    let trimmed = content.trim();
    
    if trimmed.starts_with('[') {
        let items: Vec<ReplayImport> = serde_json::from_str(trimmed)
            .map_err(|e| AppError::BadRequest(format!("JSON 解析错误: {}", e)))?;
        Ok(items)
    } else if trimmed.starts_with('{') {
        let item: ReplayImport = serde_json::from_str(trimmed)
            .map_err(|e| AppError::BadRequest(format!("JSON 解析错误: {}", e)))?;
        Ok(vec![item])
    } else {
        Err(AppError::BadRequest("JSON 格式错误，应以 [ 或 { 开头".to_string()))
    }
}

fn parse_csv(content: &str) -> Result<Vec<ReplayImport>, AppError> {
    let mut reader = csv::Reader::from_reader(content.as_bytes());
    let mut items = Vec::new();
    
    for result in reader.deserialize() {
        let record: ReplayImport = result
            .map_err(|e| AppError::BadRequest(format!("CSV 解析错误: {}", e)))?;
        items.push(record);
    }
    
    if items.is_empty() {
        return Err(AppError::BadRequest("CSV 文件为空或格式不正确".to_string()));
    }
    
    Ok(items)
}

async fn import_replays(
    State(state): State<AppState>,
    cookies: Cookies,
    mut multipart: axum::extract::Multipart,
) -> Result<ImportResponse, AppError> {
    let user_info = get_current_user(&cookies);
    
    if user_info.is_none() {
        return Ok(ImportResponse::Redirect(Redirect::to("/login")));
    }
    
    let (user_id, username, _) = user_info.unwrap();
    
    let mut content = String::new();
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("文件上传错误: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            let data = field.bytes().await.map_err(|e| {
                AppError::BadRequest(format!("读取文件内容错误: {}", e))
            })?;
            
            if data.len() > MAX_FILE_SIZE {
                return Err(AppError::BadRequest(format!(
                    "文件大小超过限制 (最大 {} MB)",
                    MAX_FILE_SIZE / 1024 / 1024
                )));
            }
            
            content = String::from_utf8(data.to_vec())
                .map_err(|e| AppError::BadRequest(format!("文件编码错误，请使用 UTF-8 编码: {}", e)))?;
        }
    }
    
    if content.is_empty() {
        return Err(AppError::BadRequest("请选择要上传的文件".to_string()));
    }
    
    let format = detect_format(&content)?;
    
    let items = match format {
        FileFormat::Json => parse_json(&content)?,
        FileFormat::Csv => parse_csv(&content)?,
    };
    
    if items.is_empty() {
        return Err(AppError::BadRequest("文件中没有有效的复盘数据".to_string()));
    }
    
    let mut success_count = 0;
    let mut errors = Vec::new();
    
    for (index, item) in items.into_iter().enumerate() {
        let row = index + 1;
        let title = item.title.clone();
        
        match item.to_new_replay() {
            Ok(new_replay) => {
                match db::replays::create(&state.db, user_id, &new_replay).await {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        errors.push(ImportError {
                            row,
                            title,
                            error: format!("数据库错误: {}", e),
                        });
                    }
                }
            }
            Err(e) => {
                errors.push(ImportError {
                    row,
                    title,
                    error: e,
                });
            }
        }
    }
    
    let failed_count = errors.len();
    
    let template = ImportResultTemplate {
        logged_in: true,
        username,
        success_count,
        failed_count,
        errors,
    };
    
    Ok(ImportResponse::Html(Html(template.render().unwrap())))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/my/import", post(import_replays))
}
