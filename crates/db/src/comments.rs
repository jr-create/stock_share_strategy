use domain::{Comment, NewComment, CommentWithAuthor};
use sqlx::{PgPool, FromRow};
use anyhow::Result;

#[derive(FromRow)]
struct CommentRow {
    id: i64,
    replay_id: i64,
    user_id: i64,
    content: String,
    created_at: chrono::DateTime<chrono::Utc>,
    author_username: String,
    author_avatar: String,
}

impl From<CommentRow> for (Comment, String, String) {
    fn from(row: CommentRow) -> Self {
        let comment = Comment {
            id: row.id,
            replay_id: row.replay_id,
            user_id: row.user_id,
            content: row.content,
            created_at: row.created_at,
        };
        (comment, row.author_username, row.author_avatar)
    }
}

pub async fn create(pool: &PgPool, user_id: i64, new_comment: &NewComment) -> Result<Comment> {
    let comment = sqlx::query_as::<_, Comment>(
        r#"
        INSERT INTO comments (replay_id, user_id, content)
        VALUES ($1, $2, $3)
        RETURNING *
        "#
    )
    .bind(new_comment.replay_id)
    .bind(user_id)
    .bind(&new_comment.content)
    .fetch_one(pool)
    .await?;

    Ok(comment)
}

pub async fn list_by_replay(pool: &PgPool, replay_id: i64) -> Result<Vec<CommentWithAuthor>> {
    let results = sqlx::query_as::<_, CommentRow>(
        r#"
        SELECT c.*, u.username as author_username, u.avatar_url as author_avatar
        FROM comments c
        JOIN users u ON c.user_id = u.id
        WHERE c.replay_id = $1
        ORDER BY c.created_at ASC
        "#
    )
    .bind(replay_id)
    .fetch_all(pool)
    .await?;

    Ok(results
        .into_iter()
        .map(|row| {
            let (comment, author_username, author_avatar) = row.into();
            CommentWithAuthor {
                comment,
                author_username,
                author_avatar,
            }
        })
        .collect())
}

pub async fn delete(pool: &PgPool, id: i64, user_id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM comments WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn count_by_replay(pool: &PgPool, replay_id: i64) -> Result<i64> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM comments WHERE replay_id = $1"
    )
    .bind(replay_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}
