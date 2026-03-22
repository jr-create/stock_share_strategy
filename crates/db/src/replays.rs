use domain::{Replay, NewReplay, UpdateReplay, ReplayFilter, ReplayWithAuthor, PaginatedResult, PaginationMeta};
use sqlx::{PgPool, FromRow};
use anyhow::Result;

#[derive(FromRow)]
struct ReplayRow {
    id: i64,
    user_id: i64,
    title: String,
    trade_date: chrono::NaiveDate,
    market_sentiment: String,
    market_view: String,
    sector_focus: String,
    watchlist: String,
    plan: String,
    execution: String,
    review: String,
    is_public: bool,
    expiry_date: Option<chrono::NaiveDate>,
    is_prediction: bool,
    is_pinned: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    author_username: String,
    author_avatar: String,
    like_count: i64,
    comment_count: i64,
}

impl From<ReplayRow> for (Replay, String, String, i64, i64) {
    fn from(row: ReplayRow) -> Self {
        let replay = Replay {
            id: row.id,
            user_id: row.user_id,
            title: row.title,
            trade_date: row.trade_date,
            market_sentiment: row.market_sentiment,
            market_view: row.market_view,
            sector_focus: row.sector_focus,
            watchlist: row.watchlist,
            plan: row.plan,
            execution: row.execution,
            review: row.review,
            is_public: row.is_public,
            expiry_date: row.expiry_date,
            is_prediction: row.is_prediction,
            is_pinned: row.is_pinned,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };
        (replay, row.author_username, row.author_avatar, row.like_count, row.comment_count)
    }
}

pub async fn create(pool: &PgPool, user_id: i64, new_replay: &NewReplay) -> Result<Replay> {
    let mut tx = pool.begin().await?;
    
    let replay = sqlx::query_as::<_, Replay>(
        r#"
        INSERT INTO replays (user_id, title, trade_date, market_sentiment, market_view, 
            sector_focus, watchlist, plan, execution, review, is_public, expiry_date, is_prediction)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(&new_replay.title)
    .bind(new_replay.trade_date)
    .bind(&new_replay.market_sentiment)
    .bind(&new_replay.market_view)
    .bind(&new_replay.sector_focus)
    .bind(&new_replay.watchlist)
    .bind(&new_replay.plan)
    .bind(&new_replay.execution)
    .bind(&new_replay.review)
    .bind(new_replay.is_public)
    .bind(new_replay.expiry_date)
    .bind(new_replay.is_prediction)
    .fetch_one(&mut *tx)
    .await?;

    for tag in &new_replay.tags {
        sqlx::query(
            "INSERT INTO replay_tags (replay_id, tag) VALUES ($1, $2)"
        )
        .bind(replay.id)
        .bind(tag)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(replay)
}

pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<Replay>> {
    let replay = sqlx::query_as::<_, Replay>(
        "SELECT * FROM replays WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(replay)
}

pub async fn find_with_author(pool: &PgPool, id: i64) -> Result<Option<ReplayWithAuthor>> {
    let result = sqlx::query_as::<_, ReplayRow>(
        r#"
        SELECT r.*, u.username as author_username, u.avatar_url as author_avatar,
            (SELECT COUNT(*) FROM likes WHERE replay_id = r.id) as like_count,
            (SELECT COUNT(*) FROM comments WHERE replay_id = r.id) as comment_count
        FROM replays r
        JOIN users u ON r.user_id = u.id
        WHERE r.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    match result {
        Some(row) => {
            let (replay, author_username, author_avatar, like_count, comment_count) = row.into();
            let tags = find_tags(pool, replay.id).await?;
            Ok(Some(ReplayWithAuthor {
                replay,
                author_username,
                author_avatar,
                like_count,
                comment_count,
                tags,
            }))
        }
        None => Ok(None),
    }
}

pub async fn find_tags(pool: &PgPool, replay_id: i64) -> Result<Vec<String>> {
    let tags: Vec<(String,)> = sqlx::query_as(
        "SELECT tag FROM replay_tags WHERE replay_id = $1"
    )
    .bind(replay_id)
    .fetch_all(pool)
    .await?;

    Ok(tags.into_iter().map(|t| t.0).collect())
}

pub async fn list(pool: &PgPool, filter: &ReplayFilter) -> Result<PaginatedResult<ReplayWithAuthor>> {
    let offset = filter.offset();
    let limit = filter.per_page();
    let page = filter.page();
    
    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM replays r
        WHERE ($1::bigint IS NULL OR r.user_id = $1)
          AND ($2::varchar IS NULL OR EXISTS (
              SELECT 1 FROM replay_tags WHERE replay_id = r.id AND tag = $2
          ))
          AND ($3::date IS NULL OR r.trade_date >= $3)
          AND ($4::date IS NULL OR r.trade_date <= $4)
          AND ($5::boolean IS NULL OR r.is_public = $5)
        "#
    )
    .bind(filter.user_id)
    .bind(&filter.tag)
    .bind(filter.start_date)
    .bind(filter.end_date)
    .bind(filter.is_public)
    .fetch_one(pool)
    .await?;

    let results = sqlx::query_as::<_, ReplayRow>(
        r#"
        SELECT r.*, u.username as author_username, u.avatar_url as author_avatar,
            (SELECT COUNT(*) FROM likes WHERE replay_id = r.id) as like_count,
            (SELECT COUNT(*) FROM comments WHERE replay_id = r.id) as comment_count
        FROM replays r
        JOIN users u ON r.user_id = u.id
        WHERE ($1::bigint IS NULL OR r.user_id = $1)
          AND ($2::varchar IS NULL OR EXISTS (
              SELECT 1 FROM replay_tags WHERE replay_id = r.id AND tag = $2
          ))
          AND ($3::date IS NULL OR r.trade_date >= $3)
          AND ($4::date IS NULL OR r.trade_date <= $4)
          AND ($5::boolean IS NULL OR r.is_public = $5)
        ORDER BY r.is_pinned DESC, r.created_at DESC
        LIMIT $6 OFFSET $7
        "#
    )
    .bind(filter.user_id)
    .bind(&filter.tag)
    .bind(filter.start_date)
    .bind(filter.end_date)
    .bind(filter.is_public)
    .bind(limit as i32)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut replays = Vec::new();
    for row in results {
        let (replay, author_username, author_avatar, like_count, comment_count) = row.into();
        let tags = find_tags(pool, replay.id).await?;
        replays.push(ReplayWithAuthor {
            replay,
            author_username,
            author_avatar,
            like_count,
            comment_count,
            tags,
        });
    }

    let pagination = PaginationMeta::new(total, page, limit);
    
    Ok(PaginatedResult {
        items: replays,
        pagination,
    })
}

pub async fn update(pool: &PgPool, id: i64, update: &UpdateReplay) -> Result<Option<Replay>> {
    let replay = sqlx::query_as::<_, Replay>(
        r#"
        UPDATE replays SET
            title = COALESCE($1, title),
            trade_date = COALESCE($2, trade_date),
            market_sentiment = COALESCE($3, market_sentiment),
            market_view = COALESCE($4, market_view),
            sector_focus = COALESCE($5, sector_focus),
            watchlist = COALESCE($6, watchlist),
            plan = COALESCE($7, plan),
            execution = COALESCE($8, execution),
            review = COALESCE($9, review),
            is_public = COALESCE($10, is_public),
            expiry_date = COALESCE($11, expiry_date),
            is_prediction = COALESCE($12, is_prediction)
        WHERE id = $13
        RETURNING *
        "#
    )
    .bind(&update.title)
    .bind(update.trade_date)
    .bind(&update.market_sentiment)
    .bind(&update.market_view)
    .bind(&update.sector_focus)
    .bind(&update.watchlist)
    .bind(&update.plan)
    .bind(&update.execution)
    .bind(&update.review)
    .bind(update.is_public)
    .bind(update.expiry_date)
    .bind(update.is_prediction)
    .bind(id)
    .fetch_optional(pool)
    .await?;

    if let Some(ref replay) = replay {
        if let Some(ref tags) = update.tags {
            sqlx::query("DELETE FROM replay_tags WHERE replay_id = $1")
                .bind(replay.id)
                .execute(pool)
                .await?;

            for tag in tags {
                sqlx::query("INSERT INTO replay_tags (replay_id, tag) VALUES ($1, $2)")
                    .bind(replay.id)
                    .bind(tag)
                    .execute(pool)
                    .await?;
            }
        }
    }

    Ok(replay)
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM replays WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn title_exists(pool: &PgPool, title: &str, exclude_id: Option<i64>) -> Result<bool> {
    let exists = match exclude_id {
        Some(id) => {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM replays WHERE title = $1 AND id != $2 AND is_public = true"
            )
            .bind(title)
            .bind(id)
            .fetch_one(pool)
            .await?
        }
        None => {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM replays WHERE title = $1 AND is_public = true"
            )
            .bind(title)
            .fetch_one(pool)
            .await?
        }
    };
    Ok(exists > 0)
}

pub async fn search(
    pool: &PgPool,
    query: &str,
    user_id: Option<i64>,
    page: i32,
    per_page: i32,
) -> Result<PaginatedResult<ReplayWithAuthor>> {
    let search_pattern = format!("%{}%", query);
    let offset = (page - 1) * per_page;
    let is_public_filter = if user_id.is_some() { None } else { Some(true) };
    
    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM replays r
        WHERE ($1::bigint IS NULL OR r.user_id = $1)
          AND (
              r.title ILIKE $2
              OR r.market_view ILIKE $2
              OR r.sector_focus ILIKE $2
              OR r.watchlist ILIKE $2
              OR r.review ILIKE $2
          )
          AND ($3::boolean IS NULL OR r.is_public = $3)
        "#
    )
    .bind(user_id)
    .bind(&search_pattern)
    .bind(is_public_filter)
    .fetch_one(pool)
    .await?;
    
    let results = sqlx::query_as::<_, ReplayRow>(
        r#"
        SELECT r.*, u.username as author_username, u.avatar_url as author_avatar,
            (SELECT COUNT(*) FROM likes WHERE replay_id = r.id) as like_count,
            (SELECT COUNT(*) FROM comments WHERE replay_id = r.id) as comment_count
        FROM replays r
        JOIN users u ON r.user_id = u.id
        WHERE ($1::bigint IS NULL OR r.user_id = $1)
          AND (
              r.title ILIKE $2
              OR r.market_view ILIKE $2
              OR r.sector_focus ILIKE $2
              OR r.watchlist ILIKE $2
              OR r.review ILIKE $2
          )
          AND ($3::boolean IS NULL OR r.is_public = $3)
        ORDER BY r.is_pinned DESC, r.created_at DESC
        LIMIT $4 OFFSET $5
        "#
    )
    .bind(user_id)
    .bind(&search_pattern)
    .bind(is_public_filter)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut replays = Vec::new();
    for row in results {
        let (replay, author_username, author_avatar, like_count, comment_count) = row.into();
        let tags = find_tags(pool, replay.id).await?;
        replays.push(ReplayWithAuthor {
            replay,
            author_username,
            author_avatar,
            like_count,
            comment_count,
            tags,
        });
    }

    let pagination = PaginationMeta::new(total, page, per_page);
    
    Ok(PaginatedResult {
        items: replays,
        pagination,
    })
}

pub async fn search_with_filter(
    pool: &PgPool,
    query: &str,
    search_field: &str,
    sort_field: &str,
    sort_order: &str,
    user_id: Option<i64>,
    per_page: i32,
    offset: i32,
) -> Result<PaginatedResult<ReplayWithAuthor>> {
    let search_pattern = format!("%{}%", query);
    let is_public_filter = if user_id.is_some() { None } else { Some(true) };
    
    // 构建搜索条件
    let search_condition = match search_field {
        "title" => "r.title ILIKE $2",
        "market_view" => "r.market_view ILIKE $2",
        "sector_focus" => "r.sector_focus ILIKE $2",
        "watchlist" => "r.watchlist ILIKE $2",
        "review" => "r.review ILIKE $2",
        "plan" => "r.plan ILIKE $2",
        "execution" => "r.execution ILIKE $2",
        _ => "r.title ILIKE $2 OR r.market_view ILIKE $2 OR r.sector_focus ILIKE $2 OR r.watchlist ILIKE $2 OR r.review ILIKE $2",
    };
    
    // 构建排序条件
    let order_clause = match sort_field {
        "created_at" => "r.created_at",
        "trade_date" => "r.trade_date",
        "expiry_date" => "r.expiry_date",
        "like_count" => "like_count",
        "comment_count" => "comment_count",
        _ => "r.created_at",
    };
    let order_direction = if sort_order == "asc" { "ASC" } else { "DESC" };
    // 置顶记录始终排在最前面
    let order_by = format!("r.is_pinned DESC, {} {}", order_clause, order_direction);
    
    let total: i64 = sqlx::query_scalar(&format!(
        r#"
        SELECT COUNT(*) FROM replays r
        WHERE ($1::bigint IS NULL OR r.user_id = $1)
          AND ({})
          AND ($3::boolean IS NULL OR r.is_public = $3)
        "#, search_condition
    ))
    .bind(user_id)
    .bind(&search_pattern)
    .bind(is_public_filter)
    .fetch_one(pool)
    .await?;
    
    let results = sqlx::query_as::<_, ReplayRow>(&format!(
        r#"
        SELECT r.*, u.username as author_username, u.avatar_url as author_avatar,
            (SELECT COUNT(*) FROM likes WHERE replay_id = r.id) as like_count,
            (SELECT COUNT(*) FROM comments WHERE replay_id = r.id) as comment_count
        FROM replays r
        JOIN users u ON r.user_id = u.id
        WHERE ($1::bigint IS NULL OR r.user_id = $1)
          AND ({})
          AND ($3::boolean IS NULL OR r.is_public = $3)
        ORDER BY {}
        LIMIT $4 OFFSET $5
        "#, search_condition, order_by
    ))
    .bind(user_id)
    .bind(&search_pattern)
    .bind(is_public_filter)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut replays = Vec::new();
    for row in results {
        let (replay, author_username, author_avatar, like_count, comment_count) = row.into();
        let tags = find_tags(pool, replay.id).await?;
        replays.push(ReplayWithAuthor {
            replay,
            author_username,
            author_avatar,
            like_count,
            comment_count,
            tags,
        });
    }

    let page = (offset / per_page) + 1;
    let pagination = PaginationMeta::new(total, page, per_page);
    
    Ok(PaginatedResult {
        items: replays,
        pagination,
    })
}

/// 设置复盘记录的置顶状态
pub async fn set_pinned(pool: &PgPool, id: i64, pinned: bool) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE replays SET is_pinned = $1 WHERE id = $2"
    )
    .bind(pinned)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
