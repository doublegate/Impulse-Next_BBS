//! Performance benchmarks for authentication operations
//!
//! Measures the performance of password hashing and session management.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use impulse_auth::{PasswordHasher, SessionManager};
use impulse_types::user::UserId;
use std::hint::black_box;
use std::time::Duration;

fn benchmark_password_hashing(c: &mut Criterion) {
    let hasher = PasswordHasher::default();
    let password = "SecurePassword123!";

    c.bench_function("password_hash", |b| {
        b.iter(|| {
            hasher
                .hash_password(black_box(password))
                .expect("Should hash")
        })
    });
}

fn benchmark_password_verification(c: &mut Criterion) {
    let hasher = PasswordHasher::default();
    let password = "SecurePassword123!";
    let hash = hasher.hash_password(password).expect("Should hash");

    c.bench_function("password_verify_correct", |b| {
        b.iter(|| {
            hasher
                .verify_password(black_box(password), black_box(&hash))
                .expect("Should verify")
        })
    });

    let wrong_password = "WrongPassword123!";

    c.bench_function("password_verify_incorrect", |b| {
        b.iter(|| {
            let _ = hasher.verify_password(black_box(wrong_password), black_box(&hash));
        })
    });
}

fn benchmark_session_creation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("session_create", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mgr = SessionManager::new(Duration::from_secs(3600));
                let user_id = UserId::new();
                mgr.create_session(black_box(user_id)).await
            })
        })
    });
}

fn benchmark_session_validation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("session_validate", |b| {
        b.iter_batched(
            || {
                let mgr = SessionManager::new(Duration::from_secs(3600));
                let user_id = UserId::new();
                let token = rt.block_on(async { mgr.create_session(user_id).await });
                (mgr, token)
            },
            |(mgr, token)| {
                rt.block_on(async {
                    mgr.get_session(black_box(&token))
                        .await
                        .expect("Should get session")
                })
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn benchmark_session_logout(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("session_logout", |b| {
        b.iter_batched(
            || {
                let mgr = SessionManager::new(Duration::from_secs(3600));
                let user_id = UserId::new();
                let token = rt.block_on(async { mgr.create_session(user_id).await });
                (mgr, token)
            },
            |(mgr, token)| rt.block_on(async { mgr.remove_session(black_box(&token)).await }),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn benchmark_session_cleanup(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("session_cleanup");

    for count in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter_batched(
                || {
                    let mgr = SessionManager::new(Duration::from_millis(1));

                    // Create expired sessions
                    rt.block_on(async {
                        let manager = mgr;
                        for _i in 0..count {
                            let user_id = UserId::new();
                            let _ = manager.create_session(user_id).await;
                        }
                        std::thread::sleep(Duration::from_millis(10)); // Ensure expiry
                        manager
                    })
                },
                |mgr| rt.block_on(async { mgr.cleanup_expired().await }),
                criterion::BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn benchmark_concurrent_session_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("session_concurrent_create_10", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mgr = SessionManager::new(Duration::from_secs(3600));

                let mut tasks = Vec::new();
                for _i in 0..10 {
                    let mgr_clone = mgr.clone();
                    tasks.push(tokio::spawn(async move {
                        let user_id = UserId::new();
                        mgr_clone.create_session(user_id).await
                    }));
                }

                for task in tasks {
                    task.await.expect("Task should complete");
                }
            })
        })
    });
}

criterion_group!(
    benches,
    benchmark_password_hashing,
    benchmark_password_verification,
    benchmark_session_creation,
    benchmark_session_validation,
    benchmark_session_logout,
    benchmark_session_cleanup,
    benchmark_concurrent_session_operations,
);

criterion_main!(benches);
