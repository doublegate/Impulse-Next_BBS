//! Authentication performance benchmarks
//!
//! This benchmark suite measures the performance of:
//! - Password hashing (Argon2id)
//! - Password verification
//! - Session token generation
//! - Session validation

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use impulse_auth::{
    flows::AuthService, lockout::AccountLockout, rate_limit::RateLimiter, PasswordHasher,
    SessionToken,
};
use std::time::Duration;

fn benchmark_password_hashing(c: &mut Criterion) {
    let hasher = PasswordHasher::new();
    let password = "SecureP@ss123";

    c.bench_function("password_hash", |b| {
        b.iter(|| hasher.hash_password(black_box(password)))
    });
}

fn benchmark_password_verification(c: &mut Criterion) {
    let hasher = PasswordHasher::new();
    let password = "SecureP@ss123";
    let hash = hasher
        .hash_password(password)
        .expect("Failed to hash password");

    c.bench_function("password_verify", |b| {
        b.iter(|| hasher.verify_password(black_box(password), black_box(&hash)))
    });
}

fn benchmark_session_token_generation(c: &mut Criterion) {
    c.bench_function("session_token_new", |b| {
        b.iter(|| SessionToken::new())
    });
}

fn benchmark_login_flow(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("login_flow", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut auth = AuthService::new_with_protection(
                    Duration::from_secs(1800),
                    RateLimiter::new_default(),
                    AccountLockout::new_default(),
                );

                let hasher = PasswordHasher::new();
                let password = "SecureP@ss123";
                let hash = hasher.hash_password(password).unwrap();

                auth.authenticate(
                    black_box("testuser"),
                    black_box(password),
                    black_box(&hash),
                )
                .await
            })
        })
    });
}

fn benchmark_session_validation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: create a session
    let (auth, token) = rt.block_on(async {
        let mut auth = AuthService::new_with_protection(
            Duration::from_secs(1800),
            RateLimiter::new_default(),
            AccountLockout::new_default(),
        );

        let hasher = PasswordHasher::new();
        let password = "SecureP@ss123";
        let hash = hasher.hash_password(password).unwrap();
        let token = auth.authenticate("testuser", password, &hash).await.unwrap();

        (auth, token)
    });

    c.bench_function("session_validate", |b| {
        b.iter(|| rt.block_on(async { auth.validate_session(black_box(&token)).await }))
    });
}

fn benchmark_logout(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("logout", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut auth = AuthService::new_with_protection(
                    Duration::from_secs(1800),
                    RateLimiter::new_default(),
                    AccountLockout::new_default(),
                );

                let hasher = PasswordHasher::new();
                let password = "SecureP@ss123";
                let hash = hasher.hash_password(password).unwrap();
                let token = auth.authenticate("testuser", password, &hash).await.unwrap();

                auth.logout(black_box(&token)).await
            })
        })
    });
}

criterion_group!(
    benches,
    benchmark_password_hashing,
    benchmark_password_verification,
    benchmark_session_token_generation,
    benchmark_login_flow,
    benchmark_session_validation,
    benchmark_logout
);
criterion_main!(benches);
