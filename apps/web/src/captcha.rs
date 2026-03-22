use rand::Rng;
use tower_cookies::{Cookie, Cookies};

pub const CAPTCHA_COOKIE_NAME: &str = "captcha_code";
pub const CAPTCHA_EXPIRY_SECS: i64 = 300; // 5分钟有效期

/// 生成6位随机数字验证码
pub fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    let code: u32 = rng.gen_range(100000..999999);
    code.to_string()
}

/// 生成SVG验证码图片
pub fn generate_svg(code: &str) -> String {
    let chars: Vec<char> = code.chars().collect();
    let width = 180;
    let height = 50;
    
    // 生成干扰线
    let mut lines = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..6 {
        let x1 = rng.gen_range(0..width);
        let y1 = rng.gen_range(0..height);
        let x2 = rng.gen_range(0..width);
        let y2 = rng.gen_range(0..height);
        let r = rng.gen_range(100..200);
        let g = rng.gen_range(100..200);
        let b = rng.gen_range(100..200);
        lines.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="rgb({},{},{})" stroke-width="1"/>"#,
            x1, y1, x2, y2, r, g, b
        ));
    }
    
    // 生成干扰点
    let mut dots = String::new();
    for _ in 0..50 {
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);
        let r = rng.gen_range(0..255);
        let g = rng.gen_range(0..255);
        let b = rng.gen_range(0..255);
        dots.push_str(&format!(
            r#"<circle cx="{}" cy="{}" r="1" fill="rgb({},{},{})"/>"#,
            x, y, r, g, b
        ));
    }
    
    // 生成每个字符
    let mut text_elements = String::new();
    for (i, &ch) in chars.iter().enumerate() {
        let x = 20 + i * 28;
        let y = 35 + rng.gen_range(-5..5);
        let rotation = rng.gen_range(-15..15);
        let r = rng.gen_range(20..80);
        let g = rng.gen_range(20..80);
        let b = rng.gen_range(20..80);
        let font_size = 28 + rng.gen_range(-4..4);
        
        text_elements.push_str(&format!(
            r#"<text x="{}" y="{}" font-family="Arial, sans-serif" font-size="{}" font-weight="bold" fill="rgb({},{},{})" transform="rotate({} {} {})">{}</text>"#,
            x, y, font_size, r, g, b, rotation, x, y, ch
        ));
    }
    
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
            <rect width="100%" height="100%" fill="#f0f0f0"/>
            {}
            {}
            {}
        </svg>"##,
        width, height, width, height, lines, dots, text_elements
    )
}

/// 设置验证码cookie
pub fn set_captcha_cookie(cookies: &Cookies, code: &str) {
    // 将验证码和过期时间戳一起存储
    let expiry = chrono::Utc::now().timestamp() + CAPTCHA_EXPIRY_SECS;
    let value = format!("{}:{}", code, expiry);
    
    let cookie = Cookie::build((CAPTCHA_COOKIE_NAME, value))
        .path("/")
        .secure(false)
        .http_only(true)
        .max_age(time::Duration::seconds(CAPTCHA_EXPIRY_SECS))
        .build();
    cookies.add(cookie);
}

/// 验证验证码
pub fn verify_captcha(cookies: &Cookies, input_code: &str) -> bool {
    if input_code.is_empty() {
        return false;
    }
    
    let Some(cookie) = cookies.get(CAPTCHA_COOKIE_NAME) else {
        return false;
    };
    
    let value = cookie.value();
    let parts: Vec<&str> = value.split(':').collect();
    
    if parts.len() != 2 {
        return false;
    }
    
    let stored_code = parts[0];
    let expiry_str = parts[1];
    
    // 检查是否过期
    if let Ok(expiry) = expiry_str.parse::<i64>() {
        if chrono::Utc::now().timestamp() > expiry {
            return false;
        }
    } else {
        return false;
    }
    
    // 验证码比较（忽略大小写）
    let result = stored_code.eq_ignore_ascii_case(input_code);
    
    // 验证后删除cookie（一次性使用）
    if result {
        cookies.remove(Cookie::build((CAPTCHA_COOKIE_NAME, "")).path("/").build());
    }
    
    result
}

/// 清除验证码cookie
pub fn clear_captcha_cookie(cookies: &Cookies) {
    cookies.remove(Cookie::build((CAPTCHA_COOKIE_NAME, "")).path("/").build());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code() {
        let code = generate_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_generate_svg() {
        let code = "123456";
        let svg = generate_svg(code);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("123456"));
        assert!(svg.contains("</svg>"));
    }
}
