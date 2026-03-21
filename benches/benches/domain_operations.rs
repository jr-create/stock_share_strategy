use criterion::{black_box, criterion_group, criterion_main, Criterion};
use domain::{User, NewReplay, ReplayFilter};
use chrono::NaiveDate;

fn bench_user_validation(c: &mut Criterion) {
    c.bench_function("validate_username_valid", |b| {
        b.iter(|| User::validate_username(black_box("valid_user123")))
    });

    c.bench_function("validate_email_valid", |b| {
        b.iter(|| User::validate_email(black_box("test@example.com")))
    });

    c.bench_function("validate_password_valid", |b| {
        b.iter(|| User::validate_password(black_box("password123")))
    });
}

fn bench_replay_creation(c: &mut Criterion) {
    c.bench_function("create_replay_struct", |b| {
        b.iter(|| {
            NewReplay {
                title: "测试复盘".to_string(),
                trade_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                market_sentiment: "震荡".to_string(),
                market_view: "大盘震荡整理".to_string(),
                sector_focus: "AI、新能源".to_string(),
                watchlist: "000001,600000".to_string(),
                plan: "低吸高抛".to_string(),
                execution: "按计划执行".to_string(),
                review: "执行良好".to_string(),
                is_public: true,
                tags: vec!["AI".to_string(), "新能源".to_string()],
            }
        })
    });
}

fn bench_filter_operations(c: &mut Criterion) {
    c.bench_function("filter_pagination", |b| {
        b.iter(|| {
            let filter = ReplayFilter {
                page: Some(5),
                per_page: Some(20),
                ..Default::default()
            };
            (filter.page(), filter.per_page(), filter.offset())
        })
    });
}

criterion_group!(user_benches, bench_user_validation);
criterion_group!(replay_benches, bench_replay_creation, bench_filter_operations);
criterion_main!(user_benches, replay_benches);
