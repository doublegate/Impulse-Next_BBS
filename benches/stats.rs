//! Statistics tracking performance benchmarks
//!
//! This benchmark suite measures the performance of:
//! - Statistics tracking operations
//! - Ratio calculations
//! - Achievement checking
//! - User settings operations

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use impulse_user::{
    achievements::{Achievement, AchievementTracker},
    settings::UserSettings,
    stats::StatsTracker,
};
use impulse_types::user::User;

fn benchmark_record_login(c: &mut Criterion) {
    let mut tracker = StatsTracker::new();

    c.bench_function("record_login", |b| {
        b.iter(|| tracker.record_login())
    });
}

fn benchmark_record_message_operations(c: &mut Criterion) {
    let mut tracker = StatsTracker::new();

    c.bench_function("record_message_post", |b| {
        b.iter(|| tracker.record_message_post())
    });

    c.bench_function("record_message_read", |b| {
        b.iter(|| tracker.record_message_read(black_box(5)))
    });
}

fn benchmark_record_file_operations(c: &mut Criterion) {
    let mut tracker = StatsTracker::new();

    c.bench_function("record_file_upload", |b| {
        b.iter(|| tracker.record_file_upload(black_box(1024)))
    });

    c.bench_function("record_file_download", |b| {
        b.iter(|| tracker.record_file_download(black_box(2048)))
    });
}

fn benchmark_get_stats(c: &mut Criterion) {
    let mut tracker = StatsTracker::new();

    // Add some data
    for _ in 0..10 {
        tracker.record_login();
        tracker.record_message_post();
        tracker.record_message_read(5);
        tracker.record_file_upload(1024);
        tracker.record_file_download(2048);
    }

    c.bench_function("get_stats", |b| b.iter(|| tracker.get_stats()));
}

fn benchmark_ratio_calculation(c: &mut Criterion) {
    let mut tracker = StatsTracker::new();

    // Setup upload/download data
    tracker.record_file_upload(10240);
    tracker.record_file_download(5120);

    c.bench_function("calculate_ratio", |b| {
        b.iter(|| {
            let stats = tracker.get_stats();
            let _ratio = black_box(stats.upload_bytes() as f64 / stats.download_bytes() as f64);
        })
    });
}

fn benchmark_session_time_tracking(c: &mut Criterion) {
    let mut tracker = StatsTracker::new();
    let duration = std::time::Duration::from_secs(600);

    c.bench_function("record_session_duration", |b| {
        b.iter(|| tracker.record_session_duration(black_box(duration)))
    });
}

fn benchmark_award_achievement(c: &mut Criterion) {
    let achievement = Achievement::new("test_ach", "Test Achievement", "Description");

    c.bench_function("award_achievement", |b| {
        b.iter(|| {
            let mut tracker = AchievementTracker::new();
            tracker.award(black_box(&achievement))
        })
    });
}

fn benchmark_check_achievement(c: &mut Criterion) {
    let mut tracker = AchievementTracker::new();
    let achievement = Achievement::new("test_ach", "Test Achievement", "Description");
    tracker.award(&achievement).unwrap();

    c.bench_function("has_achievement", |b| {
        b.iter(|| tracker.has_achievement(black_box("test_ach")))
    });
}

fn benchmark_get_achievements(c: &mut Criterion) {
    let mut tracker = AchievementTracker::new();

    // Award 10 achievements
    for i in 0..10 {
        let ach = Achievement::new(
            &format!("ach_{}", i),
            &format!("Achievement {}", i),
            "Description",
        );
        tracker.award(&ach).unwrap();
    }

    c.bench_function("get_achievements", |b| {
        b.iter(|| tracker.get_achievements())
    });
}

fn benchmark_user_settings_from_user(c: &mut Criterion) {
    let user = User::new("testuser").unwrap();

    c.bench_function("settings_from_user", |b| {
        b.iter(|| UserSettings::from_user(black_box(&user)))
    });
}

fn benchmark_user_settings_apply(c: &mut Criterion) {
    let mut user = User::new("testuser").unwrap();
    let mut settings = UserSettings::from_user(&user);
    settings.set_screen_width(132);
    settings.set_screen_height(43);

    c.bench_function("settings_apply_to_user", |b| {
        b.iter(|| settings.apply_to_user(black_box(&mut user)))
    });
}

fn benchmark_cumulative_statistics(c: &mut Criterion) {
    c.bench_function("stats_100_operations", |b| {
        b.iter(|| {
            let mut tracker = StatsTracker::new();
            for i in 0..100 {
                tracker.record_login();
                tracker.record_message_post();
                tracker.record_message_read(black_box(i % 10));
                tracker.record_file_upload(black_box(1024 * (i + 1)));
                tracker.record_file_download(black_box(512 * (i + 1)));
            }
            tracker.get_stats()
        })
    });
}

fn benchmark_achievement_progression(c: &mut Criterion) {
    c.bench_function("achievement_progression", |b| {
        b.iter(|| {
            let mut tracker = AchievementTracker::new();
            for i in 0..10 {
                let ach = Achievement::new(
                    &format!("level_{}", i),
                    &format!("Level {}", i),
                    &format!("Reached level {}", i),
                );
                tracker.award(&ach).unwrap();
            }
            tracker.get_achievements()
        })
    });
}

criterion_group!(
    benches,
    benchmark_record_login,
    benchmark_record_message_operations,
    benchmark_record_file_operations,
    benchmark_get_stats,
    benchmark_ratio_calculation,
    benchmark_session_time_tracking,
    benchmark_award_achievement,
    benchmark_check_achievement,
    benchmark_get_achievements,
    benchmark_user_settings_from_user,
    benchmark_user_settings_apply,
    benchmark_cumulative_statistics,
    benchmark_achievement_progression
);
criterion_main!(benches);
