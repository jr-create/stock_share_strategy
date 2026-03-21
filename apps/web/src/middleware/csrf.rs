use tower_cookies::Cookies;

pub const CSRF_COOKIE_NAME: &str = "csrf_token";
pub const CSRF_FORM_FIELD: &str = "csrf_token";

pub fn generate_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let token: [u8; 32] = rng.gen();
    hex::encode(token)
}

pub fn set_csrf_cookie(cookies: &Cookies) -> String {
    let token = generate_token();
    let cookie = tower_cookies::Cookie::build((CSRF_COOKIE_NAME, token.clone()))
        .path("/")
        .secure(false)
        .http_only(false)
        .build();
    cookies.add(cookie);
    token
}

pub fn verify_csrf(cookies: &Cookies, form_token: &str) -> bool {
    cookies
        .get(CSRF_COOKIE_NAME)
        .map(|c| c.value() == form_token)
        .unwrap_or(false)
}
