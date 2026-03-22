use domain::Like;
use sqlx::PgPool;
use anyhow::Result;

pub async fn create(pool: &PgPool, user_id: i64, replay_id: i64) -> Result<Like> {
    let like = sqlx::query_as::<_, Like>(
        r#"
        INSERT INTO likes (replay_id, user_id)
        VALUES ($1, $2)
        RETURNING *
        "#
    )
    .bind(replay_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(like)
}

pub async fn delete(pool: &PgPool, user_id: i64, replay_id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM likes WHERE replay_id = $1 AND user_id = $2")
        .bind(replay_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn exists(pool: &PgPool, user_id: i64, replay_id: i64) -> Result<bool> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM likes WHERE replay_id = $1 AND user_id = $2"
    )
    .bind(replay_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0 > 0)
}

pub async fn count_by_replay(pool: &PgPool, replay_id: i64) -> Result<i64> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM likes WHERE replay_id = $1"
    )
    .bind(replay_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

pub async fn get_user_likes_for_replays(pool: &PgPool, user_id: i64, replay_ids: Vec<i64>) -> Result<Vec<i64>> {
    if replay_ids.is_empty() {
        return Ok(Vec::new());
    }
    
    let likes: Vec<(i64,)> = sqlx::query_as(
        "SELECT replay_id FROM likes WHERE user_id = $1 AND replay_id = ANY($2)"
    )
    .bind(user_id)
    .bind(&replay_ids)
    .fetch_all(pool)
    .await?;

    Ok(likes.into_iter().map(|(id,)| id).collect())
}
