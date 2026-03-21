use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub bio: String,
    pub avatar_url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

impl User {
    pub fn validate_username(username: &str) -> Result<(), String> {
        if username.len() < 3 || username.len() > 32 {
            return Err("Username must be between 3 and 32 characters".to_string());
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err("Username can only contain alphanumeric characters and underscores".to_string());
        }
        Ok(())
    }

    pub fn validate_email(email: &str) -> Result<(), String> {
        if !email.contains('@') || email.len() > 128 {
            return Err("Invalid email format".to_string());
        }
        Ok(())
    }

    pub fn validate_password(password: &str) -> Result<(), String> {
        if password.len() < 8 {
            return Err("Password must be at least 8 characters".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username_valid() {
        assert!(User::validate_username("user123").is_ok());
        assert!(User::validate_username("test_user").is_ok());
        assert!(User::validate_username("abc").is_ok());
    }

    #[test]
    fn test_validate_username_invalid() {
        assert!(User::validate_username("ab").is_err());
        assert!(User::validate_username("a".repeat(33).as_str()).is_err());
        assert!(User::validate_username("user@name").is_err());
    }

    #[test]
    fn test_validate_email_valid() {
        assert!(User::validate_email("test@example.com").is_ok());
        assert!(User::validate_email("user.name@domain.org").is_ok());
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(User::validate_email("invalid").is_err());
        assert!(User::validate_email("").is_err());
    }

    #[test]
    fn test_validate_password_valid() {
        assert!(User::validate_password("password123").is_ok());
        assert!(User::validate_password("12345678").is_ok());
    }

    #[test]
    fn test_validate_password_invalid() {
        assert!(User::validate_password("short").is_err());
        assert!(User::validate_password("1234567").is_err());
    }
}
