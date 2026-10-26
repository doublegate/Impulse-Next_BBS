//! Statistics tracking performance benchmarks
//!
//! This benchmark suite measures the performance of:
//! - Statistics tracking operations
//! - Ratio calculations
//! - Achievement checking
//! - User settings operations

use criterion::{Criterion, criterion_group, criterion_main};
use impulse_types::{user::User, user_prefs::UserPreferences};
use impulse_user::{
    InMemoryUserManager, UserManager,
    achievements::{Achievement, AchievementChecker, UserAchievement},
    settings::SettingsManager,
    stats::StatsTracker,
};
use std::hint::black_box;

fn benchmark_record_login(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (mut tracker, user_id) = rt.block_on(async {
        let mut manager = InMemoryUserManager::new();
        let user = User::new("testuser").unwrap();
        let user_id = user.id();
        manager.create_user(user).await.unwrap();
        (StatsTracker::new(manager), user_id)
    });

    c.bench_function("record_login", |b| {
        b.iter(|| rt.block_on(async { tracker.record_login(black_box(user_id)).await }))
    });
}

fn benchmark_record_message_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (mut tracker, user_id) = rt.block_on(async {
        let mut manager = InMemoryUserManager::new();
        let user = User::new("testuser").unwrap();
        let user_id = user.id();
        manager.create_user(user).await.unwrap();
        (StatsTracker::new(manager), user_id)
    });

    c.bench_function("record_message_post", |b| {
        b.iter(|| rt.block_on(async { tracker.record_post(black_box(user_id)).await }))
    });

    c.bench_function("record_email", |b| {
        b.iter(|| rt.block_on(async { tracker.record_email(black_box(user_id)).await }))
    });
}

fn benchmark_record_file_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (mut tracker, user_id) = rt.block_on(async {
        let mut manager = InMemoryUserManager::new();
        let user = User::new("testuser").unwrap();
        let user_id = user.id();
        manager.create_user(user).await.unwrap();
        (StatsTracker::new(manager), user_id)
    });

    c.bench_function("record_file_upload", |b| {
        b.iter(|| {
            rt.block_on(async {
                tracker
                    .record_upload(black_box(user_id), black_box(1), black_box(1024))
                    .await
            })
        })
    });

    c.bench_function("record_file_download", |b| {
        b.iter(|| {
            rt.block_on(async {
                tracker
                    .record_download(black_box(user_id), black_box(1), black_box(2048))
                    .await
            })
        })
    });
}

fn benchmark_ratio_calculation(c: &mut Criterion) {
    let mut user = User::new("testuser").unwrap();
    user.stats.record_upload(10, 10240);
    user.stats.record_download(5, 5120);

    c.bench_function("calculate_ratio", |b| {
        b.iter(|| black_box(&user.stats).ul_dl_ratio())
    });
}

fn benchmark_check_achievements(c: &mut Criterion) {
    let mut user = User::new("testuser").unwrap();
    user.stats.record_post();
    user.stats.record_upload(15, 20480);
    user.stats.record_download(10, 10240);
    let earned = vec![UserAchievement::new(user.id(), Achievement::FirstPost)];

    c.bench_function("check_achievements", |b| {
        b.iter(|| AchievementChecker::check_all(black_box(&user), black_box(&earned)))
    });
}

fn benchmark_user_settings_update(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (mut settings_mgr, user_id) = rt.block_on(async {
        let mut manager = InMemoryUserManager::new();
        let user = User::new("testuser").unwrap();
        let user_id = user.id();
        manager.create_user(user).await.unwrap();
        (SettingsManager::new(manager), user_id)
    });

    c.bench_function("settings_update_preferences", |b| {
        b.iter(|| {
            rt.block_on(async {
                settings_mgr
                    .update_preferences(black_box(user_id), UserPreferences::expert())
                    .await
            })
        })
    });
}

criterion_group!(
    benches,
    benchmark_record_login,
    benchmark_record_message_operations,
    benchmark_record_file_operations,
    benchmark_ratio_calculation,
    benchmark_check_achievements,
    benchmark_user_settings_update,
);
criterion_main!(benches);
