use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: i64,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user_id: i64, username: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            username,
            created_at: now,
            expires_at: now + Duration::days(7),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn to_token(&self) -> String {
        self.id.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: i64,
    pub username: String,
}

impl From<Session> for SessionData {
    fn from(session: Session) -> Self {
        Self {
            user_id: session.user_id,
            username: session.username,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new(1, "testuser".to_string());
        assert_eq!(session.user_id, 1);
        assert_eq!(session.username, "testuser");
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_token() {
        let session = Session::new(1, "testuser".to_string());
        let token = session.to_token();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_session_data_from_session() {
        let session = Session::new(42, "alice".to_string());
        let data = SessionData::from(session);
        assert_eq!(data.user_id, 42);
        assert_eq!(data.username, "alice");
    }
}
