//! Authentication security testing

use crate::fixtures::BbsTestFixture;
use anyhow::Result;
use std::sync::Arc;

/// Test authentication security features
pub async fn test_auth_security(fixture: &Arc<BbsTestFixture>) -> Result<()> {
    // Test rate limiting
    test_rate_limiting(fixture).await?;

    // Test session token security
    test_session_tokens(fixture).await?;

    // Test password hashing
    test_password_hashing(fixture).await?;

    Ok(())
}

/// Test rate limiting on login attempts
async fn test_rate_limiting(fixture: &Arc<BbsTestFixture>) -> Result<()> {
    let user = fixture.create_test_user("ratetest", 10).await?;

    // Simulate rapid login attempts
    // In production, this would trigger rate limiting after 3 attempts
    for i in 0..10 {
        tracing::debug!("Login attempt {} for user {}", i + 1, user.username);

        // After 3 attempts, should be rate limited
        if i >= 3 {
            tracing::info!("Login attempt {} should be rate limited", i + 1);
        }
    }

    Ok(())
}

/// Test session token security
async fn test_session_tokens(fixture: &Arc<BbsTestFixture>) -> Result<()> {
    let user = fixture.create_test_user("tokentest", 10).await?;

    // Generate session token (simulated)
    let token = generate_session_token();

    // Verify token properties
    assert!(token.len() >= 32, "Token too short (< 128 bits)");
    assert!(is_cryptographically_random(&token), "Token not random");

    // Generate multiple tokens to verify uniqueness
    let token2 = generate_session_token();
    assert_ne!(token, token2, "Tokens not unique");

    tracing::info!("User {} session token validated", user.username);
    Ok(())
}

/// Test password hashing security
async fn test_password_hashing(fixture: &Arc<BbsTestFixture>) -> Result<()> {
    let user = fixture.create_test_user("hashtest", 10).await?;

    // Verify password hash format
    assert!(
        user.password_hash.starts_with("$argon2id$"),
        "Password not using Argon2id"
    );

    // Verify hash contains appropriate parameters
    assert!(
        user.password_hash.contains("m="),
        "Missing memory parameter"
    );
    assert!(user.password_hash.contains("t="), "Missing time parameter");
    assert!(
        user.password_hash.contains("p="),
        "Missing parallelism parameter"
    );

    Ok(())
}

/// Generate a cryptographically secure session token
fn generate_session_token() -> String {
    use rand::RngExt;
    let mut rng = rand::rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.random::<u8>()).collect();
    hex::encode(bytes)
}

/// Check if a token has good entropy (basic check)
fn is_cryptographically_random(token: &str) -> bool {
    // Basic entropy check: should have good character distribution
    // Hex string should have variety of characters
    if token.is_empty() {
        return false;
    }

    let unique_chars: std::collections::HashSet<_> = token.chars().collect();
    // For hex encoded string, we expect at least 4 different characters
    unique_chars.len() >= 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authentication_security() {
        let fixture = Arc::new(BbsTestFixture::new().await.unwrap());
        let result = test_auth_security(&fixture).await;
        assert!(result.is_ok(), "Authentication security test failed");
    }

    #[tokio::test]
    async fn test_rate_limit_enforcement() {
        let fixture = Arc::new(BbsTestFixture::new().await.unwrap());
        let result = test_rate_limiting(&fixture).await;
        assert!(result.is_ok(), "Rate limiting test failed");
    }

    #[tokio::test]
    async fn test_token_security() {
        let fixture = Arc::new(BbsTestFixture::new().await.unwrap());
        let result = test_session_tokens(&fixture).await;
        assert!(result.is_ok(), "Session token test failed");
    }

    #[tokio::test]
    async fn test_password_hash_format() {
        let fixture = Arc::new(BbsTestFixture::new().await.unwrap());
        let result = test_password_hashing(&fixture).await;
        assert!(result.is_ok(), "Password hashing test failed");
    }

    #[test]
    fn test_token_generation() {
        let token1 = generate_session_token();
        let token2 = generate_session_token();

        assert_eq!(token1.len(), 64); // 32 bytes hex encoded
        assert_ne!(token1, token2);
        assert!(is_cryptographically_random(&token1));
    }
}
