use domain::{User, NewUser};
use sqlx::PgPool;
use anyhow::Result;

pub async fn create(pool: &PgPool, new_user: &NewUser, password_hash: &str) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING *
        "#
    )
    .bind(&new_user.username)
    .bind(&new_user.email)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = $1"
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn update_bio(pool: &PgPool, id: i64, bio: &str) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET bio = $1 WHERE id = $2 RETURNING *"
    )
    .bind(bio)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn update_avatar(pool: &PgPool, id: i64, avatar_url: &str) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET avatar_url = $1 WHERE id = $2 RETURNING *"
    )
    .bind(avatar_url)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}
