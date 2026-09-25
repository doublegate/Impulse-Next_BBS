//! Load testing benchmarks using Criterion
//!
//! Run with: cargo bench --package impulse-integration-tests

use criterion::{Criterion, criterion_group, criterion_main};
use impulse_integration_tests::{fixtures::BbsTestFixture, stress::LoadGenerator};
use std::hint::black_box;

/// Benchmark user creation
fn benchmark_user_creation(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("create_user", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let fixture = BbsTestFixture::new().await.unwrap();
                let user = fixture
                    .create_test_user("bench_user", black_box(10))
                    .await
                    .unwrap();
                black_box(user);
            })
        });
    });
}

/// Benchmark concurrent user sessions (10 users, 5 seconds)
fn benchmark_concurrent_sessions(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("concurrent_10_users_5s", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let generator = LoadGenerator::new().await.unwrap();
                let metrics = generator
                    .run_load_test(black_box(10), black_box(5))
                    .await
                    .unwrap();
                black_box(metrics);
            })
        });
    });
}

/// Benchmark fixture creation
fn benchmark_fixture_creation(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("create_fixture", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let fixture = BbsTestFixture::new().await.unwrap();
                black_box(fixture);
            })
        });
    });
}

criterion_group!(
    benches,
    benchmark_user_creation,
    benchmark_fixture_creation,
    benchmark_concurrent_sessions,
);
criterion_main!(benches);
