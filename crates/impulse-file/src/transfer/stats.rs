//! Download statistics tracking
//!
//! This module provides download statistics tracking similar to upload statistics,
//! allowing the BBS to track download activity and enforce quotas.

/// Download statistics tracking
#[derive(Debug, Clone, Default)]
pub struct DownloadStats {
    /// Total files downloaded
    pub total_files: u32,

    /// Total bytes downloaded
    pub total_bytes: u64,

    /// Files downloaded today
    pub files_today: u32,

    /// Bytes downloaded today
    pub bytes_today: u64,

    /// Files downloaded this session
    pub files_session: u32,

    /// Bytes downloaded this session
    pub bytes_session: u64,

    /// Total time spent downloading (seconds)
    pub total_time_secs: u64,

    /// Downloads that used resume
    pub resumed_transfers: u32,
}

impl DownloadStats {
    /// Create new download statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a download
    pub fn record_download(&mut self, size_bytes: u64, duration_secs: u64, was_resumed: bool) {
        self.total_files += 1;
        self.total_bytes += size_bytes;
        self.files_today += 1;
        self.bytes_today += size_bytes;
        self.files_session += 1;
        self.bytes_session += size_bytes;
        self.total_time_secs += duration_secs;

        if was_resumed {
            self.resumed_transfers += 1;
        }
    }

    /// Reset daily statistics
    pub fn reset_daily(&mut self) {
        self.files_today = 0;
        self.bytes_today = 0;
    }

    /// Reset session statistics
    pub fn reset_session(&mut self) {
        self.files_session = 0;
        self.bytes_session = 0;
    }

    /// Check if daily file quota would be exceeded
    pub fn would_exceed_file_quota(&self, max_files: u32) -> bool {
        self.files_today >= max_files
    }

    /// Check if daily byte quota would be exceeded
    pub fn would_exceed_byte_quota(&self, size_bytes: u64, max_bytes: u64) -> bool {
        self.bytes_today + size_bytes > max_bytes
    }

    /// Get average download speed (bytes per second)
    pub fn average_speed(&self) -> u64 {
        self.total_bytes
            .checked_div(self.total_time_secs)
            .unwrap_or(0)
    }

    /// Get resume rate as percentage
    pub fn resume_rate(&self) -> f64 {
        if self.total_files > 0 {
            (self.resumed_transfers as f64 / self.total_files as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate download/upload ratio
    ///
    /// A ratio > 1.0 means more downloads than uploads
    pub fn ratio(&self, upload_bytes: u64) -> f64 {
        if upload_bytes > 0 {
            self.total_bytes as f64 / upload_bytes as f64
        } else if self.total_bytes > 0 {
            f64::INFINITY
        } else {
            1.0
        }
    }

    /// Check if user exceeds ratio limit
    ///
    /// Returns true if the download/upload ratio exceeds the limit
    pub fn exceeds_ratio_limit(&self, upload_bytes: u64, max_ratio: f64) -> bool {
        self.ratio(upload_bytes) > max_ratio
    }
}

/// Combined transfer statistics
#[derive(Debug, Clone, Default)]
pub struct TransferStats {
    /// Download statistics
    pub downloads: DownloadStats,

    /// Total uploads for ratio calculation
    pub upload_bytes: u64,

    /// Upload file count
    pub upload_files: u32,
}

impl TransferStats {
    /// Create new combined statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an upload (for ratio tracking)
    pub fn record_upload(&mut self, size_bytes: u64) {
        self.upload_bytes += size_bytes;
        self.upload_files += 1;
    }

    /// Record a download
    pub fn record_download(&mut self, size_bytes: u64, duration_secs: u64, was_resumed: bool) {
        self.downloads
            .record_download(size_bytes, duration_secs, was_resumed);
    }

    /// Get the current download/upload ratio
    pub fn ratio(&self) -> f64 {
        self.downloads.ratio(self.upload_bytes)
    }

    /// Check if additional download would exceed ratio limit
    pub fn can_download(&self, size_bytes: u64, max_ratio: f64) -> bool {
        let projected_download = self.downloads.total_bytes + size_bytes;
        if self.upload_bytes > 0 {
            (projected_download as f64 / self.upload_bytes as f64) <= max_ratio
        } else {
            // Allow some leeway for new users
            projected_download <= 10 * 1024 * 1024 // 10MB grace period
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_stats_new() {
        let stats = DownloadStats::new();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_bytes, 0);
        assert_eq!(stats.files_today, 0);
        assert_eq!(stats.bytes_today, 0);
        assert_eq!(stats.files_session, 0);
        assert_eq!(stats.bytes_session, 0);
    }

    #[test]
    fn test_download_stats_record() {
        let mut stats = DownloadStats::new();

        stats.record_download(1024, 10, false);
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.total_bytes, 1024);
        assert_eq!(stats.files_today, 1);
        assert_eq!(stats.bytes_today, 1024);
        assert_eq!(stats.files_session, 1);
        assert_eq!(stats.bytes_session, 1024);
        assert_eq!(stats.total_time_secs, 10);
        assert_eq!(stats.resumed_transfers, 0);

        stats.record_download(2048, 20, true);
        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.total_bytes, 3072);
        assert_eq!(stats.resumed_transfers, 1);
    }

    #[test]
    fn test_download_stats_reset_daily() {
        let mut stats = DownloadStats::new();
        stats.record_download(1024, 10, false);
        stats.record_download(2048, 20, false);

        stats.reset_daily();

        assert_eq!(stats.total_files, 2); // Preserved
        assert_eq!(stats.total_bytes, 3072); // Preserved
        assert_eq!(stats.files_today, 0); // Reset
        assert_eq!(stats.bytes_today, 0); // Reset
        assert_eq!(stats.files_session, 2); // Preserved
    }

    #[test]
    fn test_download_stats_reset_session() {
        let mut stats = DownloadStats::new();
        stats.record_download(1024, 10, false);
        stats.record_download(2048, 20, false);

        stats.reset_session();

        assert_eq!(stats.total_files, 2); // Preserved
        assert_eq!(stats.files_session, 0); // Reset
        assert_eq!(stats.bytes_session, 0); // Reset
    }

    #[test]
    fn test_download_stats_quotas() {
        let mut stats = DownloadStats::new();
        stats.files_today = 9;
        stats.bytes_today = 5000;

        assert!(!stats.would_exceed_file_quota(10));
        stats.files_today = 10;
        assert!(stats.would_exceed_file_quota(10));

        stats.bytes_today = 5000;
        assert!(!stats.would_exceed_byte_quota(4000, 10000));
        assert!(stats.would_exceed_byte_quota(6000, 10000));
    }

    #[test]
    fn test_download_stats_average_speed() {
        let mut stats = DownloadStats::new();
        stats.total_bytes = 10240;
        stats.total_time_secs = 10;

        assert_eq!(stats.average_speed(), 1024);
    }

    #[test]
    fn test_download_stats_average_speed_zero_time() {
        let stats = DownloadStats::new();
        assert_eq!(stats.average_speed(), 0);
    }

    #[test]
    fn test_download_stats_resume_rate() {
        let mut stats = DownloadStats::new();
        stats.total_files = 10;
        stats.resumed_transfers = 3;

        assert!((stats.resume_rate() - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_download_stats_ratio() {
        let mut stats = DownloadStats::new();
        stats.total_bytes = 10240;

        assert!((stats.ratio(5120) - 2.0).abs() < 0.01);
        assert_eq!(stats.ratio(0), f64::INFINITY);

        stats.total_bytes = 0;
        assert!((stats.ratio(0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_download_stats_exceeds_ratio() {
        let mut stats = DownloadStats::new();
        stats.total_bytes = 10240;

        assert!(stats.exceeds_ratio_limit(5120, 1.5)); // 2.0 > 1.5
        assert!(!stats.exceeds_ratio_limit(5120, 2.5)); // 2.0 < 2.5
    }

    #[test]
    fn test_transfer_stats_new() {
        let stats = TransferStats::new();
        assert_eq!(stats.downloads.total_files, 0);
        assert_eq!(stats.upload_bytes, 0);
        assert_eq!(stats.upload_files, 0);
    }

    #[test]
    fn test_transfer_stats_record() {
        let mut stats = TransferStats::new();

        stats.record_upload(2048);
        assert_eq!(stats.upload_bytes, 2048);
        assert_eq!(stats.upload_files, 1);

        stats.record_download(4096, 10, false);
        assert_eq!(stats.downloads.total_bytes, 4096);
        assert!((stats.ratio() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_transfer_stats_can_download() {
        let mut stats = TransferStats::new();
        stats.upload_bytes = 5120;

        // Current: 0 download, 5120 upload, ratio = 0
        assert!(stats.can_download(10240, 3.0)); // Would be 2.0 ratio

        stats.downloads.total_bytes = 10240;
        // Current: 10240 download, 5120 upload, ratio = 2.0
        assert!(!stats.can_download(10240, 3.0)); // Would be 4.0 ratio
    }

    #[test]
    fn test_transfer_stats_can_download_new_user() {
        let stats = TransferStats::new();
        // New user with no uploads gets 10MB grace period
        assert!(stats.can_download(5 * 1024 * 1024, 3.0));
        assert!(!stats.can_download(15 * 1024 * 1024, 3.0));
    }
}
