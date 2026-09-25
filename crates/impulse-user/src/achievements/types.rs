//! Achievement types and definitions
//!
//! This module defines the various achievements users can earn on the BBS,
//! along with supporting types for tracking progress and awarded achievements.

use chrono::{DateTime, Utc};
use impulse_types::user::{User, UserId};
use serde::{Deserialize, Serialize};

/// Achievement types
///
/// Defines all possible achievements that can be earned by users.
///
/// # Examples
///
/// ```
/// use impulse_user::achievements::types::Achievement;
///
/// let achievement = Achievement::FirstPost;
/// assert_eq!(achievement.name(), "First Post");
/// assert_eq!(achievement.description(), "Posted your first message");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Achievement {
    /// Posted first message
    FirstPost,
    /// Uploaded 10 files
    Upload10,
    /// Uploaded 100 files
    Upload100,
    /// Uploaded 1000 files
    Upload1000,
    /// Downloaded 10 files
    Download10,
    /// Downloaded 100 files
    Download100,
    /// Downloaded 1000 files
    Download1000,
    /// Logged in 10 times
    Calls10,
    /// Logged in 100 times
    Calls100,
    /// Logged in 1000 times
    Calls1000,
    /// Member for 1 month
    Member1Month,
    /// Member for 6 months
    Member6Months,
    /// Member for 1 year (loyal member)
    LoyalMember,
    /// Member for 5 years
    VeteranMember,
    /// Posted 100 messages
    Posts100,
    /// Posted 500 messages
    Posts500,
    /// Posted 1000 messages
    Posts1000,
    /// Spent 10 hours online
    Time10Hours,
    /// Spent 100 hours online
    Time100Hours,
    /// Spent 1000 hours online
    Time1000Hours,
    /// Maintained 1:1 upload/download ratio
    Ratio1to1,
    /// Maintained 1:2 upload/download ratio
    Ratio1to2,
    /// Sent 50 emails
    Email50,
    /// Earned 1000 file points
    FilePoints1000,
    /// Earned 10000 file points
    FilePoints10000,
}

impl Achievement {
    /// Get the human-readable name of the achievement
    ///
    /// # Examples
    ///
    /// ```
    /// use impulse_user::achievements::types::Achievement;
    ///
    /// assert_eq!(Achievement::FirstPost.name(), "First Post");
    /// assert_eq!(Achievement::LoyalMember.name(), "Loyal Member");
    /// ```
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Achievement::FirstPost => "First Post",
            Achievement::Upload10 => "Upload Beginner",
            Achievement::Upload100 => "Upload Expert",
            Achievement::Upload1000 => "Upload Master",
            Achievement::Download10 => "Download Beginner",
            Achievement::Download100 => "Download Expert",
            Achievement::Download1000 => "Download Master",
            Achievement::Calls10 => "Regular Visitor",
            Achievement::Calls100 => "Frequent Visitor",
            Achievement::Calls1000 => "Super Visitor",
            Achievement::Member1Month => "New Member",
            Achievement::Member6Months => "Established Member",
            Achievement::LoyalMember => "Loyal Member",
            Achievement::VeteranMember => "Veteran Member",
            Achievement::Posts100 => "Active Poster",
            Achievement::Posts500 => "Prolific Poster",
            Achievement::Posts1000 => "Master Poster",
            Achievement::Time10Hours => "Time Spender",
            Achievement::Time100Hours => "Time Devotee",
            Achievement::Time1000Hours => "Time Master",
            Achievement::Ratio1to1 => "Fair Trader",
            Achievement::Ratio1to2 => "Generous Trader",
            Achievement::Email50 => "Communicator",
            Achievement::FilePoints1000 => "Points Earner",
            Achievement::FilePoints10000 => "Points Master",
        }
    }

    /// Get the description of the achievement
    ///
    /// # Examples
    ///
    /// ```
    /// use impulse_user::achievements::types::Achievement;
    ///
    /// assert_eq!(Achievement::FirstPost.description(), "Posted your first message");
    /// ```
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Achievement::FirstPost => "Posted your first message",
            Achievement::Upload10 => "Uploaded 10 files",
            Achievement::Upload100 => "Uploaded 100 files",
            Achievement::Upload1000 => "Uploaded 1000 files",
            Achievement::Download10 => "Downloaded 10 files",
            Achievement::Download100 => "Downloaded 100 files",
            Achievement::Download1000 => "Downloaded 1000 files",
            Achievement::Calls10 => "Logged in 10 times",
            Achievement::Calls100 => "Logged in 100 times",
            Achievement::Calls1000 => "Logged in 1000 times",
            Achievement::Member1Month => "Member for 1 month",
            Achievement::Member6Months => "Member for 6 months",
            Achievement::LoyalMember => "Member for 1 year",
            Achievement::VeteranMember => "Member for 5 years",
            Achievement::Posts100 => "Posted 100 messages",
            Achievement::Posts500 => "Posted 500 messages",
            Achievement::Posts1000 => "Posted 1000 messages",
            Achievement::Time10Hours => "Spent 10 hours online",
            Achievement::Time100Hours => "Spent 100 hours online",
            Achievement::Time1000Hours => "Spent 1000 hours online",
            Achievement::Ratio1to1 => "Maintained 1:1 upload/download ratio",
            Achievement::Ratio1to2 => "Maintained 1:2 upload/download ratio",
            Achievement::Email50 => "Sent 50 emails",
            Achievement::FilePoints1000 => "Earned 1000 file points",
            Achievement::FilePoints10000 => "Earned 10000 file points",
        }
    }

    /// Get the badge icon for the achievement (ASCII art)
    ///
    /// # Examples
    ///
    /// ```
    /// use impulse_user::achievements::types::Achievement;
    ///
    /// assert_eq!(Achievement::FirstPost.badge(), "[*]");
    /// assert_eq!(Achievement::LoyalMember.badge(), "[★]");
    /// ```
    #[must_use]
    pub fn badge(&self) -> &'static str {
        match self {
            Achievement::FirstPost => "[*]",
            Achievement::Upload10 | Achievement::Download10 | Achievement::Calls10 => "[+]",
            Achievement::Upload100 | Achievement::Download100 | Achievement::Calls100 => "[++]",
            Achievement::Upload1000 | Achievement::Download1000 | Achievement::Calls1000 => "[★]",
            Achievement::Member1Month => "[#]",
            Achievement::Member6Months => "[##]",
            Achievement::LoyalMember => "[★]",
            Achievement::VeteranMember => "[★★]",
            Achievement::Posts100 => "[M]",
            Achievement::Posts500 => "[MM]",
            Achievement::Posts1000 => "[MMM]",
            Achievement::Time10Hours => "[T]",
            Achievement::Time100Hours => "[TT]",
            Achievement::Time1000Hours => "[TTT]",
            Achievement::Ratio1to1 => "[=]",
            Achievement::Ratio1to2 => "[==]",
            Achievement::Email50 => "[@]",
            Achievement::FilePoints1000 => "[P]",
            Achievement::FilePoints10000 => "[PP]",
        }
    }

    /// Get all possible achievements
    ///
    /// # Examples
    ///
    /// ```
    /// use impulse_user::achievements::types::Achievement;
    ///
    /// let all = Achievement::all();
    /// assert!(all.len() > 0);
    /// assert!(all.contains(&Achievement::FirstPost));
    /// ```
    #[must_use]
    pub fn all() -> Vec<Achievement> {
        vec![
            Achievement::FirstPost,
            Achievement::Upload10,
            Achievement::Upload100,
            Achievement::Upload1000,
            Achievement::Download10,
            Achievement::Download100,
            Achievement::Download1000,
            Achievement::Calls10,
            Achievement::Calls100,
            Achievement::Calls1000,
            Achievement::Member1Month,
            Achievement::Member6Months,
            Achievement::LoyalMember,
            Achievement::VeteranMember,
            Achievement::Posts100,
            Achievement::Posts500,
            Achievement::Posts1000,
            Achievement::Time10Hours,
            Achievement::Time100Hours,
            Achievement::Time1000Hours,
            Achievement::Ratio1to1,
            Achievement::Ratio1to2,
            Achievement::Email50,
            Achievement::FilePoints1000,
            Achievement::FilePoints10000,
        ]
    }
}

/// A user's earned achievement
///
/// Represents an achievement that has been awarded to a user, including when it was earned.
///
/// # Examples
///
/// ```
/// use impulse_user::achievements::types::{Achievement, UserAchievement};
/// use impulse_types::user::UserId;
///
/// let user_id = UserId::new();
/// let achievement = UserAchievement::new(user_id, Achievement::FirstPost);
///
/// assert_eq!(achievement.achievement, Achievement::FirstPost);
/// assert_eq!(achievement.user_id, user_id);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAchievement {
    /// User who earned the achievement
    pub user_id: UserId,

    /// The achievement earned
    pub achievement: Achievement,

    /// When the achievement was earned
    pub earned_at: DateTime<Utc>,
}

impl UserAchievement {
    /// Create a new user achievement with current timestamp
    ///
    /// # Examples
    ///
    /// ```
    /// use impulse_user::achievements::types::{Achievement, UserAchievement};
    /// use impulse_types::user::UserId;
    ///
    /// let user_id = UserId::new();
    /// let ua = UserAchievement::new(user_id, Achievement::FirstPost);
    /// assert_eq!(ua.achievement, Achievement::FirstPost);
    /// ```
    #[must_use]
    pub fn new(user_id: UserId, achievement: Achievement) -> Self {
        UserAchievement {
            user_id,
            achievement,
            earned_at: Utc::now(),
        }
    }

    /// Create a new user achievement with a specific timestamp
    ///
    /// # Examples
    ///
    /// ```
    /// use impulse_user::achievements::types::{Achievement, UserAchievement};
    /// use impulse_types::user::UserId;
    /// use chrono::Utc;
    ///
    /// let user_id = UserId::new();
    /// let timestamp = Utc::now();
    /// let ua = UserAchievement::with_timestamp(user_id, Achievement::FirstPost, timestamp);
    /// assert_eq!(ua.earned_at, timestamp);
    /// ```
    #[must_use]
    pub fn with_timestamp(
        user_id: UserId,
        achievement: Achievement,
        earned_at: DateTime<Utc>,
    ) -> Self {
        UserAchievement {
            user_id,
            achievement,
            earned_at,
        }
    }
}

/// Progress toward an achievement
///
/// Tracks how close a user is to earning a specific achievement.
///
/// # Examples
///
/// ```
/// use impulse_user::achievements::types::{Achievement, AchievementProgress};
/// use impulse_types::user::User;
///
/// let user = User::new("testuser").unwrap();
/// let progress = AchievementProgress::calculate(&user, Achievement::Upload10);
///
/// assert_eq!(progress.achievement, Achievement::Upload10);
/// assert_eq!(progress.current, 0); // No uploads yet
/// assert_eq!(progress.required, 10);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct AchievementProgress {
    /// The achievement being tracked
    pub achievement: Achievement,

    /// Current progress value
    pub current: u64,

    /// Required value to earn achievement
    pub required: u64,
}

impl AchievementProgress {
    /// Calculate progress toward an achievement for a user
    ///
    /// # Examples
    ///
    /// ```
    /// use impulse_user::achievements::types::{Achievement, AchievementProgress};
    /// use impulse_types::user::User;
    ///
    /// let mut user = User::new("testuser").unwrap();
    /// user.stats.record_upload(5, 1000);
    ///
    /// let progress = AchievementProgress::calculate(&user, Achievement::Upload10);
    /// assert_eq!(progress.current, 5);
    /// assert_eq!(progress.required, 10);
    /// assert!(!progress.is_complete());
    /// ```
    #[must_use]
    pub fn calculate(user: &User, achievement: Achievement) -> Self {
        use std::time::SystemTime;

        let (current, required) = match achievement {
            Achievement::FirstPost => (user.stats.posts as u64, 1),
            Achievement::Upload10 => (user.stats.uploads as u64, 10),
            Achievement::Upload100 => (user.stats.uploads as u64, 100),
            Achievement::Upload1000 => (user.stats.uploads as u64, 1000),
            Achievement::Download10 => (user.stats.downloads as u64, 10),
            Achievement::Download100 => (user.stats.downloads as u64, 100),
            Achievement::Download1000 => (user.stats.downloads as u64, 1000),
            Achievement::Calls10 => (user.stats.logins as u64, 10),
            Achievement::Calls100 => (user.stats.logins as u64, 100),
            Achievement::Calls1000 => (user.stats.logins as u64, 1000),
            Achievement::Member1Month => {
                let days = user
                    .created_at
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    / 86400;
                (days, 30)
            }
            Achievement::Member6Months => {
                let days = user
                    .created_at
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    / 86400;
                (days, 180)
            }
            Achievement::LoyalMember => {
                let days = user
                    .created_at
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    / 86400;
                (days, 365)
            }
            Achievement::VeteranMember => {
                let days = user
                    .created_at
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    / 86400;
                (days, 1825)
            }
            Achievement::Posts100 => (user.stats.posts as u64, 100),
            Achievement::Posts500 => (user.stats.posts as u64, 500),
            Achievement::Posts1000 => (user.stats.posts as u64, 1000),
            Achievement::Time10Hours => (user.stats.total_time_minutes as u64, 600),
            Achievement::Time100Hours => (user.stats.total_time_minutes as u64, 6000),
            Achievement::Time1000Hours => (user.stats.total_time_minutes as u64, 60000),
            Achievement::Ratio1to1 => {
                let ratio = user.stats.ul_dl_ratio().unwrap_or(0.0);
                ((ratio * 100.0) as u64, 100)
            }
            Achievement::Ratio1to2 => {
                let ratio = user.stats.ul_dl_ratio().unwrap_or(0.0);
                ((ratio * 100.0) as u64, 50)
            }
            Achievement::Email50 => (user.stats.emails_sent as u64, 50),
            Achievement::FilePoints1000 => (user.stats.file_points.max(0) as u64, 1000),
            Achievement::FilePoints10000 => (user.stats.file_points.max(0) as u64, 10000),
        };

        AchievementProgress {
            achievement,
            current,
            required,
        }
    }

    /// Check if the achievement is complete
    ///
    /// # Examples
    ///
    /// ```
    /// use impulse_user::achievements::types::{Achievement, AchievementProgress};
    /// use impulse_types::user::User;
    ///
    /// let mut user = User::new("testuser").unwrap();
    /// user.stats.record_post(); // 1 post
    ///
    /// let progress = AchievementProgress::calculate(&user, Achievement::FirstPost);
    /// assert!(progress.is_complete());
    /// ```
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.current >= self.required
    }

    /// Get percentage complete (0-100)
    ///
    /// # Examples
    ///
    /// ```
    /// use impulse_user::achievements::types::{Achievement, AchievementProgress};
    /// use impulse_types::user::User;
    ///
    /// let mut user = User::new("testuser").unwrap();
    /// user.stats.record_upload(5, 1000);
    ///
    /// let progress = AchievementProgress::calculate(&user, Achievement::Upload10);
    /// assert_eq!(progress.percentage(), 50);
    /// ```
    #[must_use]
    pub fn percentage(&self) -> u8 {
        // Widen to u128 so `current * 100` cannot overflow for large u64 values.
        // A zero requirement is trivially complete.
        (u128::from(self.current) * 100)
            .checked_div(u128::from(self.required))
            .map_or(100, |pct| pct.min(100) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_name() {
        assert_eq!(Achievement::FirstPost.name(), "First Post");
        assert_eq!(Achievement::LoyalMember.name(), "Loyal Member");
        assert_eq!(Achievement::Upload100.name(), "Upload Expert");
    }

    #[test]
    fn test_achievement_description() {
        assert_eq!(
            Achievement::FirstPost.description(),
            "Posted your first message"
        );
        assert_eq!(Achievement::Calls100.description(), "Logged in 100 times");
    }

    #[test]
    fn test_achievement_badge() {
        assert_eq!(Achievement::FirstPost.badge(), "[*]");
        assert_eq!(Achievement::LoyalMember.badge(), "[★]");
        assert_eq!(Achievement::Upload10.badge(), "[+]");
    }

    #[test]
    fn test_achievement_all() {
        let all = Achievement::all();
        assert!(all.len() > 20);
        assert!(all.contains(&Achievement::FirstPost));
        assert!(all.contains(&Achievement::LoyalMember));
    }

    #[test]
    fn test_user_achievement_new() {
        let user_id = UserId::new();
        let ua = UserAchievement::new(user_id, Achievement::FirstPost);

        assert_eq!(ua.user_id, user_id);
        assert_eq!(ua.achievement, Achievement::FirstPost);
    }

    #[test]
    fn test_user_achievement_with_timestamp() {
        let user_id = UserId::new();
        let timestamp = Utc::now();
        let ua = UserAchievement::with_timestamp(user_id, Achievement::FirstPost, timestamp);

        assert_eq!(ua.earned_at, timestamp);
    }

    #[test]
    fn test_progress_first_post() {
        let mut user = User::new("testuser").unwrap();
        let progress = AchievementProgress::calculate(&user, Achievement::FirstPost);

        assert_eq!(progress.current, 0);
        assert_eq!(progress.required, 1);
        assert!(!progress.is_complete());

        user.stats.record_post();
        let progress = AchievementProgress::calculate(&user, Achievement::FirstPost);
        assert!(progress.is_complete());
    }

    #[test]
    fn test_progress_uploads() {
        let mut user = User::new("testuser").unwrap();
        user.stats.record_upload(5, 1000);

        let progress = AchievementProgress::calculate(&user, Achievement::Upload10);
        assert_eq!(progress.current, 5);
        assert_eq!(progress.required, 10);
        assert_eq!(progress.percentage(), 50);
        assert!(!progress.is_complete());

        user.stats.record_upload(5, 1000);
        let progress = AchievementProgress::calculate(&user, Achievement::Upload10);
        assert!(progress.is_complete());
    }

    #[test]
    fn test_progress_percentage() {
        let mut user = User::new("testuser").unwrap();
        user.stats.record_upload(25, 1000);

        let progress = AchievementProgress::calculate(&user, Achievement::Upload100);
        assert_eq!(progress.percentage(), 25);

        user.stats.record_upload(75, 5000);
        let progress = AchievementProgress::calculate(&user, Achievement::Upload100);
        assert_eq!(progress.percentage(), 100);
    }

    #[test]
    fn test_progress_percentage_extremes() {
        let at = |current, required| AchievementProgress {
            achievement: Achievement::Upload100,
            current,
            required,
        };
        // A zero requirement counts as complete.
        assert_eq!(at(0, 0).percentage(), 100);
        // `current * 100` would overflow u64 here; the result must still be exact.
        assert_eq!(at(u64::MAX, u64::MAX).percentage(), 100);
        assert_eq!(at(u64::MAX / 2, u64::MAX).percentage(), 49);
        assert_eq!(at(u64::MAX, 1).percentage(), 100);
    }

    #[test]
    fn test_serialization() {
        let user_id = UserId::new();
        let ua = UserAchievement::new(user_id, Achievement::FirstPost);

        // Test serialization roundtrip
        let serialized = serde_json::to_string(&ua).expect("Failed to serialize");
        let deserialized: UserAchievement =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(deserialized.user_id, ua.user_id);
        assert_eq!(deserialized.achievement, ua.achievement);
    }
}
