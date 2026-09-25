//! Test fixtures and utilities for integration testing
//!
//! Provides reusable test infrastructure including:
//! - BbsTestFixture: Clean isolated test environment
//! - UserFactory: Test user creation with various configurations
//! - Test data generators
//! - Helper functions for common test operations

mod bbs_fixture;
mod user_factory;

pub use bbs_fixture::BbsTestFixture;
pub use user_factory::UserFactory;

use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::RngExt;
use std::path::PathBuf;

/// Simplified User struct for integration testing
#[derive(Debug, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub security_level: u8,
    pub total_calls: u32,
    pub upload_limit_kb: u32,
    pub download_limit_kb: u32,
    pub time_limit: u16,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Generate a unique test username
pub fn generate_test_username() -> String {
    let mut rng = rand::rng();
    format!("testuser_{}", rng.random::<u32>())
}

/// Generate a random test email address
pub fn generate_test_email(username: &str) -> String {
    format!("{}@test.local", username)
}

/// Create a test password hash (using a weak but fast hash for testing)
pub fn test_password_hash() -> String {
    // In production, this would be Argon2id, but for tests we use a simple hash
    "$argon2id$v=19$m=4096,t=1,p=1$test$test".to_string()
}

/// Get a temporary directory for test files
pub fn get_test_temp_dir() -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join(format!("impulse-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    Ok(temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_username() {
        let username = generate_test_username();
        assert!(username.starts_with("testuser_"));
        assert!(username.len() > 9);
    }

    #[test]
    fn test_generate_email() {
        let email = generate_test_email("testuser");
        assert_eq!(email, "testuser@test.local");
    }

    #[test]
    fn test_password_hash_format() {
        let hash = test_password_hash();
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_temp_dir_creation() {
        let dir = get_test_temp_dir().unwrap();
        assert!(dir.exists());
        // Cleanup
        let _ = std::fs::remove_dir_all(dir);
    }
}
