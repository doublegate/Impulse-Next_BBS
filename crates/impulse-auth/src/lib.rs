//! Authentication and session management for the BBS
//!
//! This module provides password hashing, session management, authentication
//! services, rate limiting, account lockout, and input validation for user
//! login/logout operations.
//!
//! # Modules
//!
//! - [`flows`] - High-level authentication workflows (login, register, logout)
//! - [`rate_limit`] - Rate limiting for login attempts
//! - [`lockout`] - Account lockout after repeated failures
//! - [`validation`] - Input validation for usernames, passwords, and emails
//!
//! # Examples
//!
//! ## Complete authentication flow
//!
//! ```no_run
//! use impulse_auth::{AuthService, rate_limit::RateLimiter, lockout::AccountLockout};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create authentication service with rate limiting and lockout
//! let auth = AuthService::new_with_protection(
//!     Duration::from_secs(1800), // 30 min session timeout
//!     RateLimiter::new_default(),
//!     AccountLockout::new_default(),
//! );
//!
//! // Authenticate user
//! let username = "johndoe";
//! let password = "SecureP@ss123";
//!
//! match auth.authenticate(username, password, "stored_hash").await {
//!     Ok(token) => println!("Login successful! Token: {}", token),
//!     Err(e) => println!("Login failed: {}", e),
//! }
//! # Ok(())
//! # }
//! ```

pub mod flows;
pub mod lockout;
pub mod rate_limit;
pub mod validation;

use argon2::{Argon2, PasswordHash, PasswordHasher as Argon2PasswordHasher, PasswordVerifier};
use impulse_types::{
    Error,
    user::{User, UserId},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::sync::RwLock;

/// Authentication-specific errors
#[derive(Debug, Error)]
pub enum AuthError {
    /// Invalid credentials provided
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    /// Session expired
    #[error("Session expired: {0}")]
    SessionExpired(String),

    /// Password hashing error
    #[error("Password hashing error: {0}")]
    HashingError(String),

    /// Generic authentication error
    #[error("Authentication error: {0}")]
    Generic(String),
}

impl From<AuthError> for Error {
    fn from(err: AuthError) -> Self {
        Error::Authentication(err.to_string())
    }
}

/// Session token - unique identifier for a user session
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionToken(String);

impl SessionToken {
    /// Create a new random session token
    pub fn new() -> Self {
        use sha2::{Digest, Sha256};
        let random_bytes: [u8; 32] = rand::random();
        let hash = Sha256::digest(random_bytes);
        // sha2 0.11 digests no longer implement `LowerHex`; encode byte by byte.
        let hex = hash.iter().fold(String::with_capacity(64), |mut acc, b| {
            use std::fmt::Write as _;
            let _ = write!(acc, "{b:02x}");
            acc
        });
        SessionToken(hex)
    }

    /// Get the token string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionToken {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Active user session
#[derive(Debug, Clone)]
pub struct Session {
    /// Session token
    token: SessionToken,

    /// User ID associated with this session
    user_id: UserId,

    /// Session creation time
    created_at: SystemTime,

    /// Last activity time
    last_activity: SystemTime,

    /// Session timeout duration
    timeout: Duration,
}

impl Session {
    /// Create a new session
    pub fn new(user_id: UserId, timeout: Duration) -> Self {
        let now = SystemTime::now();
        Self {
            token: SessionToken::new(),
            user_id,
            created_at: now,
            last_activity: now,
            timeout,
        }
    }

    /// Get the session token
    pub fn token(&self) -> &SessionToken {
        &self.token
    }

    /// Get the user ID
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    /// Get creation time
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }

    /// Get last activity time
    pub fn last_activity(&self) -> SystemTime {
        self.last_activity
    }

    /// Update last activity time
    pub fn touch(&mut self) {
        self.last_activity = SystemTime::now();
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        if let Ok(elapsed) = self.last_activity.elapsed() {
            elapsed > self.timeout
        } else {
            true // If we can't determine elapsed time, consider expired
        }
    }

    /// Get remaining time until expiration
    pub fn remaining_time(&self) -> Option<Duration> {
        if let Ok(elapsed) = self.last_activity.elapsed() {
            self.timeout.checked_sub(elapsed)
        } else {
            None
        }
    }
}

/// Password hasher using Argon2id
pub struct PasswordHasher {
    argon2: Argon2<'static>,
}

impl PasswordHasher {
    /// Create a new password hasher with default parameters
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    /// Hash a password
    ///
    /// # Errors
    ///
    /// Returns `AuthError::HashingError` if hashing fails
    pub fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        // argon2 0.6 draws a 16-byte salt from the OS RNG (its `getrandom` feature).
        let password_hash = Argon2PasswordHasher::hash_password(&self.argon2, password.as_bytes())
            .map_err(|e| AuthError::HashingError(e.to_string()))?;
        Ok(password_hash.to_string())
    }

    /// Verify a password against a hash
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InvalidCredentials` if verification fails
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<(), AuthError> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| AuthError::HashingError(e.to_string()))?;

        self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AuthError::InvalidCredentials)
    }
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Session information
///
/// Contains metadata about an active session.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// User ID associated with this session
    pub user_id: UserId,

    /// When the session was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,

    /// When the session will expire
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Session manager for tracking active sessions
pub struct SessionManager {
    /// Active sessions by token
    sessions: Arc<RwLock<HashMap<SessionToken, Session>>>,

    /// Default session timeout
    default_timeout: Duration,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(default_timeout: Duration) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            default_timeout,
        }
    }

    /// Create a new session for a user
    pub async fn create_session(&self, user_id: UserId) -> SessionToken {
        let session = Session::new(user_id, self.default_timeout);
        let token = session.token().clone();

        let mut sessions = self.sessions.write().await;
        sessions.insert(token.clone(), session);

        token
    }

    /// Get a session by token
    ///
    /// # Errors
    ///
    /// Returns `AuthError::SessionNotFound` if session doesn't exist
    /// Returns `AuthError::SessionExpired` if session is expired
    pub async fn get_session(&self, token: &SessionToken) -> Result<Session, AuthError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(token)
            .ok_or_else(|| AuthError::SessionNotFound(token.to_string()))?;

        if session.is_expired() {
            return Err(AuthError::SessionExpired(token.to_string()));
        }

        Ok(session.clone())
    }

    /// Update session activity time
    ///
    /// # Errors
    ///
    /// Returns `AuthError::SessionNotFound` if session doesn't exist
    /// Returns `AuthError::SessionExpired` if session is expired
    pub async fn touch_session(&self, token: &SessionToken) -> Result<(), AuthError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(token)
            .ok_or_else(|| AuthError::SessionNotFound(token.to_string()))?;

        if session.is_expired() {
            return Err(AuthError::SessionExpired(token.to_string()));
        }

        session.touch();
        Ok(())
    }

    /// Remove a session
    pub async fn remove_session(&self, token: &SessionToken) -> bool {
        let mut sessions = self.sessions.write().await;
        sessions.remove(token).is_some()
    }

    /// Get all active sessions for a user
    pub async fn get_user_sessions(&self, user_id: UserId) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.user_id() == user_id && !s.is_expired())
            .cloned()
            .collect()
    }

    /// Remove all sessions for a user
    pub async fn remove_user_sessions(&self, user_id: UserId) -> usize {
        let mut sessions = self.sessions.write().await;
        let tokens: Vec<_> = sessions
            .iter()
            .filter(|(_, s)| s.user_id() == user_id)
            .map(|(t, _)| t.clone())
            .collect();

        let count = tokens.len();
        for token in tokens {
            sessions.remove(&token);
        }
        count
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let expired_tokens: Vec<_> = sessions
            .iter()
            .filter(|(_, s)| s.is_expired())
            .map(|(t, _)| t.clone())
            .collect();

        let count = expired_tokens.len();
        for token in expired_tokens {
            sessions.remove(&token);
        }
        count
    }

    /// Get count of active sessions
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.values().filter(|s| !s.is_expired()).count()
    }

    /// Check if session is still valid (not expired)
    ///
    /// # Arguments
    ///
    /// * `token` - Session token to check
    ///
    /// # Returns
    ///
    /// Returns `true` if session exists and is not expired
    pub async fn is_valid(&self, token: &SessionToken) -> bool {
        if let Ok(session) = self.get_session(token).await {
            !session.is_expired()
        } else {
            false
        }
    }

    /// Refresh session - update last activity timestamp
    ///
    /// This is an alias for `touch_session` for consistency with
    /// common session management terminology.
    ///
    /// # Arguments
    ///
    /// * `token` - Session token to refresh
    ///
    /// # Errors
    ///
    /// Returns `AuthError::SessionNotFound` if session doesn't exist
    /// Returns `AuthError::SessionExpired` if session is expired
    pub async fn refresh(&self, token: &SessionToken) -> Result<(), AuthError> {
        self.touch_session(token).await
    }

    /// Get session info (user_id, created_at, last_activity, expires_at)
    ///
    /// # Arguments
    ///
    /// * `token` - Session token to get info for
    ///
    /// # Returns
    ///
    /// Returns `Some(SessionInfo)` if session exists and is valid,
    /// `None` if session doesn't exist or is expired
    pub async fn get_info(&self, token: &SessionToken) -> Option<SessionInfo> {
        match self.get_session(token).await {
            Ok(session) => {
                let created_at = session
                    .created_at()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?;
                let last_activity = session
                    .last_activity()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?;
                let expires_at = last_activity + session.timeout;

                Some(SessionInfo {
                    user_id: session.user_id(),
                    created_at: chrono::DateTime::from_timestamp(
                        created_at.as_secs() as i64,
                        created_at.subsec_nanos(),
                    )?,
                    last_activity: chrono::DateTime::from_timestamp(
                        last_activity.as_secs() as i64,
                        last_activity.subsec_nanos(),
                    )?,
                    expires_at: chrono::DateTime::from_timestamp(
                        expires_at.as_secs() as i64,
                        expires_at.subsec_nanos(),
                    )?,
                })
            }
            Err(_) => None,
        }
    }

    /// Get active session count for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID to count sessions for
    ///
    /// # Returns
    ///
    /// Returns the number of active (non-expired) sessions for the user
    pub async fn active_sessions_for_user(&self, user_id: UserId) -> u32 {
        self.get_user_sessions(user_id).await.len() as u32
    }
}

impl Clone for SessionManager {
    fn clone(&self) -> Self {
        Self {
            sessions: Arc::clone(&self.sessions),
            default_timeout: self.default_timeout,
        }
    }
}

/// Authentication service combining password verification, session management,
/// rate limiting, and account lockout
pub struct AuthService {
    /// Password hasher
    hasher: PasswordHasher,

    /// Session manager
    sessions: SessionManager,

    /// Rate limiter (optional)
    rate_limiter: Option<rate_limit::RateLimiter>,

    /// Account lockout (optional)
    lockout: Option<lockout::AccountLockout>,
}

impl AuthService {
    /// Create a new authentication service without protection mechanisms
    pub fn new(session_timeout: Duration) -> Self {
        Self {
            hasher: PasswordHasher::new(),
            sessions: SessionManager::new(session_timeout),
            rate_limiter: None,
            lockout: None,
        }
    }

    /// Create a new authentication service with rate limiting and account lockout
    ///
    /// # Examples
    ///
    /// ```
    /// use impulse_auth::{AuthService, rate_limit::RateLimiter, lockout::AccountLockout};
    /// use std::time::Duration;
    ///
    /// let auth = AuthService::new_with_protection(
    ///     Duration::from_secs(1800),
    ///     RateLimiter::new_default(),
    ///     AccountLockout::new_default(),
    /// );
    /// ```
    pub fn new_with_protection(
        session_timeout: Duration,
        rate_limiter: rate_limit::RateLimiter,
        lockout: lockout::AccountLockout,
    ) -> Self {
        Self {
            hasher: PasswordHasher::new(),
            sessions: SessionManager::new(session_timeout),
            rate_limiter: Some(rate_limiter),
            lockout: Some(lockout),
        }
    }

    /// Hash a password for storage
    ///
    /// # Errors
    ///
    /// Returns `AuthError::HashingError` if hashing fails
    pub fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        self.hasher.hash_password(password)
    }

    /// Authenticate a user and create a session
    ///
    /// Checks rate limiting and account lockout before attempting authentication.
    /// Records failures and successful logins for security tracking.
    ///
    /// # Arguments
    ///
    /// * `user` - The user to authenticate
    /// * `password` - The password to verify
    /// * `stored_hash` - The stored password hash for this user
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InvalidCredentials` if password doesn't match
    /// Returns errors from rate limiting or account lockout if applicable
    pub async fn login(
        &self,
        user: &User,
        password: &str,
        stored_hash: &str,
    ) -> Result<SessionToken, AuthError> {
        let username = user.username();

        // Check account lockout first
        if let Some(ref lockout) = self.lockout
            && let Err(e) = lockout.check(username).await
        {
            tracing::warn!(
                username = %username,
                error = %e,
                "Login blocked: account locked"
            );
            return Err(AuthError::Generic(e.to_string()));
        }

        // Check rate limiting
        if let Some(ref rate_limiter) = self.rate_limiter
            && let Err(e) = rate_limiter.check(username).await
        {
            tracing::warn!(
                username = %username,
                error = %e,
                "Login blocked: rate limit exceeded"
            );
            return Err(AuthError::Generic(e.to_string()));
        }

        // Verify password
        match self.hasher.verify_password(password, stored_hash) {
            Ok(()) => {
                // Clear rate limiting and lockout attempts on success
                if let Some(ref rate_limiter) = self.rate_limiter {
                    rate_limiter.record_success(username).await;
                }
                if let Some(ref lockout) = self.lockout {
                    lockout.record_success(username).await;
                }

                // Create session
                let token = self.sessions.create_session(user.id()).await;
                tracing::info!(
                    user_id = ?user.id(),
                    username = %username,
                    token = %token,
                    "User logged in successfully"
                );
                Ok(token)
            }
            Err(e) => {
                // Record failed attempt for rate limiting and lockout
                if let Some(ref rate_limiter) = self.rate_limiter {
                    rate_limiter.record_attempt(username).await;
                }
                if let Some(ref lockout) = self.lockout {
                    lockout.record_failure(username).await;
                }

                tracing::warn!(
                    user_id = ?user.id(),
                    username = %username,
                    "Login failed: invalid credentials"
                );
                Err(e)
            }
        }
    }

    /// Authenticate a user with username (convenience method)
    ///
    /// Looks up user and calls login. Includes rate limiting and account lockout.
    ///
    /// # Arguments
    ///
    /// * `username` - Username to authenticate
    /// * `password` - Password to verify
    /// * `stored_hash` - The stored password hash
    ///
    /// # Errors
    ///
    /// Returns authentication errors from login attempt
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use impulse_auth::AuthService;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let auth = AuthService::new(Duration::from_secs(1800));
    /// let token = auth.authenticate("johndoe", "password", "hash").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
        stored_hash: &str,
    ) -> Result<SessionToken, AuthError> {
        // Create a temporary user for authentication
        // In production, this would look up the actual user from storage
        let user = User::new(username)
            .map_err(|e| AuthError::Generic(format!("Invalid username: {}", e)))?;

        self.login(&user, password, stored_hash).await
    }

    /// Validate a session token and return the user ID
    ///
    /// # Errors
    ///
    /// Returns `AuthError::SessionNotFound` if session doesn't exist
    /// Returns `AuthError::SessionExpired` if session is expired
    pub async fn validate_session(&self, token: &SessionToken) -> Result<UserId, AuthError> {
        match self.sessions.get_session(token).await {
            Ok(session) => {
                let user_id = session.user_id();
                match self.sessions.touch_session(token).await {
                    Ok(()) => {
                        tracing::debug!(
                            user_id = ?user_id,
                            token = %token,
                            "Session validated successfully"
                        );
                        Ok(user_id)
                    }
                    Err(e) => {
                        tracing::warn!(
                            user_id = ?user_id,
                            token = %token,
                            error = %e,
                            "Session validation failed: could not update activity"
                        );
                        Err(e)
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    token = %token,
                    error = %e,
                    "Session validation failed"
                );
                Err(e)
            }
        }
    }

    /// Log out a user by removing their session
    pub async fn logout(&self, token: &SessionToken) -> bool {
        let result = self.sessions.remove_session(token).await;
        if result {
            tracing::info!(
                token = %token,
                "User logged out successfully"
            );
        } else {
            tracing::warn!(
                token = %token,
                "Logout failed: session not found"
            );
        }
        result
    }

    /// Log out all sessions for a user
    pub async fn logout_all(&self, user_id: UserId) -> usize {
        let count = self.sessions.remove_user_sessions(user_id).await;
        tracing::info!(
            user_id = ?user_id,
            session_count = count,
            "Logged out all user sessions"
        );
        count
    }

    /// Get active sessions for a user
    pub async fn get_user_sessions(&self, user_id: UserId) -> Vec<Session> {
        self.sessions.get_user_sessions(user_id).await
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let count = self.sessions.cleanup_expired().await;
        if count > 0 {
            tracing::info!(expired_count = count, "Cleaned up expired sessions");
        } else {
            tracing::debug!("No expired sessions to clean up");
        }
        count
    }

    /// Get count of active sessions
    pub async fn active_session_count(&self) -> usize {
        self.sessions.session_count().await
    }
}

impl Clone for AuthService {
    fn clone(&self) -> Self {
        Self {
            hasher: PasswordHasher::new(),
            sessions: self.sessions.clone(),
            rate_limiter: self.rate_limiter.clone(),
            lockout: self.lockout.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let hasher = PasswordHasher::new();
        let password = "test_password_123";

        // Hash password
        let hash = hasher.hash_password(password).unwrap();
        assert!(!hash.is_empty());

        // Verify correct password
        assert!(hasher.verify_password(password, &hash).is_ok());

        // Verify incorrect password
        assert!(hasher.verify_password("wrong_password", &hash).is_err());
    }

    #[test]
    fn test_password_hash_uniqueness() {
        let hasher = PasswordHasher::new();
        let password = "test_password";

        let hash1 = hasher.hash_password(password).unwrap();
        let hash2 = hasher.hash_password(password).unwrap();

        // Hashes should be different due to random salt
        assert_ne!(hash1, hash2);

        // Both should verify correctly
        assert!(hasher.verify_password(password, &hash1).is_ok());
        assert!(hasher.verify_password(password, &hash2).is_ok());
    }

    /// Hashes stored before the argon2 0.5 -> 0.6 upgrade must keep verifying.
    /// This PHC string was produced by argon2 0.5.3 (`SaltString::generate` +
    /// `Argon2::default()`), which is what `hash_password` used until then.
    #[test]
    fn test_verifies_hash_from_argon2_0_5() {
        let legacy = "$argon2id$v=19$m=19456,t=2,p=1$Wp2JvjCIQZh82WWD8MlDjA$\
                      WVN+hmV71NsV4gQGfElEqvSzq/L0kjx3WYLGA2ieQEk";
        let hasher = PasswordHasher::new();
        assert!(hasher.verify_password("legacy-pass-0.5", legacy).is_ok());
        assert!(hasher.verify_password("legacy-pass-0.6", legacy).is_err());
    }

    /// New hashes keep the same PHC shape and default parameters as before.
    #[test]
    fn test_hash_format_unchanged() {
        let hash = PasswordHasher::new().hash_password("shape").unwrap();
        assert!(
            hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"),
            "{hash}"
        );
    }

    #[test]
    fn test_session_token_is_64_lowercase_hex() {
        let token = SessionToken::new();
        assert_eq!(token.as_str().len(), 64);
        assert!(
            token
                .as_str()
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
        );
    }

    #[test]
    fn test_session_token_creation() {
        let token1 = SessionToken::new();
        let token2 = SessionToken::new();

        // Tokens should be unique
        assert_ne!(token1, token2);

        // Tokens should be non-empty
        assert!(!token1.as_str().is_empty());
        assert!(!token2.as_str().is_empty());
    }

    #[test]
    fn test_session_creation() {
        let user_id = UserId::new();
        let timeout = Duration::from_secs(3600);
        let session = Session::new(user_id, timeout);

        assert_eq!(session.user_id(), user_id);
        assert!(!session.is_expired());
        assert!(session.remaining_time().is_some());
    }

    #[test]
    fn test_session_expiration() {
        let user_id = UserId::new();
        // Use longer timeout to account for CI and tarpaulin overhead
        let timeout = Duration::from_millis(500);
        let session = Session::new(user_id, timeout);

        // Session should not be expired immediately
        assert!(!session.is_expired());
        assert!(session.remaining_time().is_some());

        // Wait for expiration (with margin for timing variability and tarpaulin overhead)
        std::thread::sleep(Duration::from_millis(600));

        // Session should now be expired
        assert!(session.is_expired());
        assert!(session.remaining_time().is_none());
    }

    #[tokio::test]
    async fn test_session_manager_create() {
        let manager = SessionManager::new(Duration::from_secs(3600));
        let user_id = UserId::new();

        let token = manager.create_session(user_id).await;
        let session = manager.get_session(&token).await.unwrap();

        assert_eq!(session.user_id(), user_id);
        assert!(!session.is_expired());
    }

    #[tokio::test]
    async fn test_session_manager_touch() {
        let manager = SessionManager::new(Duration::from_secs(3600));
        let user_id = UserId::new();
        let token = manager.create_session(user_id).await;

        let session_before = manager.get_session(&token).await.unwrap();

        // Wait a bit and touch session
        tokio::time::sleep(Duration::from_millis(100)).await;
        manager.touch_session(&token).await.unwrap();

        let session_after = manager.get_session(&token).await.unwrap();

        // Last activity should be updated
        assert!(session_after.last_activity() > session_before.last_activity());
    }

    #[tokio::test]
    async fn test_session_manager_remove() {
        let manager = SessionManager::new(Duration::from_secs(3600));
        let user_id = UserId::new();
        let token = manager.create_session(user_id).await;

        // Session should exist
        assert!(manager.get_session(&token).await.is_ok());

        // Remove session
        assert!(manager.remove_session(&token).await);

        // Session should not exist
        assert!(manager.get_session(&token).await.is_err());
    }

    #[tokio::test]
    async fn test_session_manager_user_sessions() {
        let manager = SessionManager::new(Duration::from_secs(3600));
        let user_id = UserId::new();

        // Create multiple sessions for same user
        manager.create_session(user_id).await;
        manager.create_session(user_id).await;
        manager.create_session(user_id).await;

        let sessions = manager.get_user_sessions(user_id).await;
        assert_eq!(sessions.len(), 3);
    }

    #[tokio::test]
    async fn test_session_manager_remove_user_sessions() {
        let manager = SessionManager::new(Duration::from_secs(3600));
        let user_id = UserId::new();

        // Create sessions
        manager.create_session(user_id).await;
        manager.create_session(user_id).await;

        // Remove all user sessions
        let count = manager.remove_user_sessions(user_id).await;
        assert_eq!(count, 2);

        // No sessions should remain
        let sessions = manager.get_user_sessions(user_id).await;
        assert_eq!(sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_session_manager_cleanup() {
        let manager = SessionManager::new(Duration::from_millis(10));
        let user_id = UserId::new();

        // Create sessions
        manager.create_session(user_id).await;
        manager.create_session(user_id).await;

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Cleanup expired sessions
        let count = manager.cleanup_expired().await;
        assert_eq!(count, 2);

        // Session count should be 0
        assert_eq!(manager.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_auth_service_login() {
        let auth = AuthService::new(Duration::from_secs(3600));
        let password = "test_password";

        // Create a user and hash password separately
        let hash = auth.hash_password(password).unwrap();
        let user = User::new("testuser").unwrap();

        // Login should succeed
        let token = auth.login(&user, password, &hash).await.unwrap();
        assert!(!token.as_str().is_empty());

        // Session should be valid
        let user_id = auth.validate_session(&token).await.unwrap();
        assert_eq!(user_id, user.id());
    }

    #[tokio::test]
    async fn test_auth_service_invalid_password() {
        let auth = AuthService::new(Duration::from_secs(3600));

        let hash = auth.hash_password("correct_password").unwrap();
        let user = User::new("testuser").unwrap();

        // Login with wrong password should fail
        let result = auth.login(&user, "wrong_password", &hash).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auth_service_logout() {
        let auth = AuthService::new(Duration::from_secs(3600));
        let password = "test_password";

        let hash = auth.hash_password(password).unwrap();
        let user = User::new("testuser").unwrap();

        // Login
        let token = auth.login(&user, password, &hash).await.unwrap();

        // Logout
        assert!(auth.logout(&token).await);

        // Session should be invalid
        assert!(auth.validate_session(&token).await.is_err());
    }

    #[tokio::test]
    async fn test_auth_service_logout_all() {
        let auth = AuthService::new(Duration::from_secs(3600));
        let password = "test_password";

        let hash = auth.hash_password(password).unwrap();
        let user = User::new("testuser").unwrap();

        // Create multiple sessions
        auth.login(&user, password, &hash).await.unwrap();
        auth.login(&user, password, &hash).await.unwrap();
        auth.login(&user, password, &hash).await.unwrap();

        // Verify sessions exist
        let sessions = auth.get_user_sessions(user.id()).await;
        assert_eq!(sessions.len(), 3);

        // Logout all
        let count = auth.logout_all(user.id()).await;
        assert_eq!(count, 3);

        // No sessions should remain
        let sessions = auth.get_user_sessions(user.id()).await;
        assert_eq!(sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_auth_service_cleanup() {
        let auth = AuthService::new(Duration::from_millis(10));
        let password = "test_password";

        let hash = auth.hash_password(password).unwrap();
        let user = User::new("testuser").unwrap();

        // Create sessions
        auth.login(&user, password, &hash).await.unwrap();
        auth.login(&user, password, &hash).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Cleanup
        let count = auth.cleanup_expired_sessions().await;
        assert_eq!(count, 2);
        assert_eq!(auth.active_session_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_manager_is_valid() {
        let manager = SessionManager::new(Duration::from_secs(3600));
        let user_id = UserId::new();
        let token = manager.create_session(user_id).await;

        // Session should be valid
        assert!(manager.is_valid(&token).await);

        // Remove session
        manager.remove_session(&token).await;

        // Session should not be valid
        assert!(!manager.is_valid(&token).await);
    }

    #[tokio::test]
    async fn test_session_manager_refresh() {
        let manager = SessionManager::new(Duration::from_secs(3600));
        let user_id = UserId::new();
        let token = manager.create_session(user_id).await;

        // Get initial session
        let session_before = manager.get_session(&token).await.unwrap();

        // Wait a bit
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Refresh session
        assert!(manager.refresh(&token).await.is_ok());

        // Get updated session
        let session_after = manager.get_session(&token).await.unwrap();

        // Last activity should be updated
        assert!(session_after.last_activity() > session_before.last_activity());
    }

    #[tokio::test]
    async fn test_session_manager_get_info() {
        let manager = SessionManager::new(Duration::from_secs(3600));
        let user_id = UserId::new();
        let token = manager.create_session(user_id).await;

        // Get session info
        let info = manager.get_info(&token).await;
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.user_id, user_id);
        assert!(info.created_at <= chrono::Utc::now());
        assert!(info.last_activity <= chrono::Utc::now());
        assert!(info.expires_at > chrono::Utc::now());
    }

    #[tokio::test]
    async fn test_session_manager_active_sessions_for_user() {
        let manager = SessionManager::new(Duration::from_secs(3600));
        let user_id = UserId::new();

        // Initially no sessions
        assert_eq!(manager.active_sessions_for_user(user_id).await, 0);

        // Create sessions
        manager.create_session(user_id).await;
        manager.create_session(user_id).await;
        manager.create_session(user_id).await;

        // Should have 3 active sessions
        assert_eq!(manager.active_sessions_for_user(user_id).await, 3);
    }

    #[tokio::test]
    async fn test_session_info_for_non_existent_session() {
        let manager = SessionManager::new(Duration::from_secs(3600));
        let token = SessionToken::new();

        // Should return None for non-existent session
        assert!(manager.get_info(&token).await.is_none());
    }
}
