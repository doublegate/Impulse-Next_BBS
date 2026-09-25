//! Load generator for concurrent user simulation

use anyhow::Result;
use rand::RngExt;
use std::sync::Arc;
use std::time::Instant;
use tokio::task::JoinSet;

use super::LoadMetrics;
use crate::fixtures::BbsTestFixture;

/// Load generator for stress testing
///
/// Simulates multiple concurrent users performing realistic
/// BBS operations to test system performance under load.
pub struct LoadGenerator {
    fixture: Arc<BbsTestFixture>,
    metrics: Arc<LoadMetrics>,
}

impl LoadGenerator {
    /// Create a new load generator
    pub async fn new() -> Result<Self> {
        let fixture = Arc::new(BbsTestFixture::new().await?);
        let metrics = LoadMetrics::new_shared();

        Ok(Self { fixture, metrics })
    }

    /// Get reference to the test fixture
    pub fn fixture(&self) -> &Arc<BbsTestFixture> {
        &self.fixture
    }

    /// Get reference to load metrics
    pub fn metrics(&self) -> &Arc<LoadMetrics> {
        &self.metrics
    }

    /// Run load test with specified parameters
    ///
    /// # Arguments
    ///
    /// * `num_users` - Number of concurrent users to simulate
    /// * `duration_secs` - Test duration in seconds
    ///
    /// # Returns
    ///
    /// LoadMetrics containing test results
    pub async fn run_load_test(&self, num_users: usize, duration_secs: u64) -> Result<LoadMetrics> {
        tracing::info!(
            "Starting load test: {} users for {}s",
            num_users,
            duration_secs
        );

        // Create test users
        let users = self
            .fixture
            .user_factory
            .create_users_batch(num_users, 20)
            .await?;

        tracing::info!("Created {} test users", users.len());

        // Spawn user simulation tasks
        let mut tasks = JoinSet::new();

        for user in users {
            let metrics = Arc::clone(&self.metrics);
            let fixture = Arc::clone(&self.fixture);

            tasks.spawn(async move {
                Self::simulate_user(user.username, metrics, fixture, duration_secs).await
            });
        }

        // Wait for all tasks to complete
        while let Some(result) = tasks.join_next().await {
            if let Err(e) = result {
                tracing::error!("Task error: {}", e);
                self.metrics.record_error();
            }
        }

        tracing::info!("Load test completed");
        self.metrics.print_summary();

        // Return metrics (clone atomic values)
        Ok(LoadMetrics {
            total_connections: AtomicU64::new(self.metrics.get_total_connections()),
            successful_logins: AtomicU64::new(self.metrics.get_successful_logins()),
            failed_logins: AtomicU64::new(self.metrics.get_failed_logins()),
            messages_posted: AtomicU64::new(
                self.metrics
                    .messages_posted
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            files_uploaded: AtomicU64::new(
                self.metrics
                    .files_uploaded
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            files_downloaded: AtomicU64::new(
                self.metrics
                    .files_downloaded
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            doors_launched: AtomicU64::new(
                self.metrics
                    .doors_launched
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            total_latency_ms: AtomicU64::new(
                self.metrics
                    .total_latency_ms
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            errors: AtomicU64::new(self.metrics.get_errors()),
            operations_completed: AtomicU64::new(
                self.metrics
                    .operations_completed
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
        })
    }

    /// Simulate individual user behavior
    async fn simulate_user(
        username: String,
        metrics: Arc<LoadMetrics>,
        fixture: Arc<BbsTestFixture>,
        duration_secs: u64,
    ) -> Result<()> {
        let start_time = Instant::now();
        let end_time = start_time + std::time::Duration::from_secs(duration_secs);

        while Instant::now() < end_time {
            // Record connection attempt
            metrics.increment_connections();

            let operation_start = Instant::now();

            // Simulate login
            if Self::simulate_login(&username, &metrics, &fixture)
                .await
                .is_ok()
            {
                metrics.record_successful_login();

                // Simulate random user actions (generate random number before await)
                let action = {
                    let mut rng = rand::rng();
                    rng.random::<u8>() % 4
                };

                match action {
                    0 => {
                        // Post message
                        Self::simulate_message_post(&username, &metrics, &fixture).await?;
                        metrics.record_message_posted();
                    }
                    1 => {
                        // Upload file
                        Self::simulate_file_upload(&username, &metrics, &fixture).await?;
                        metrics.record_file_uploaded();
                    }
                    2 => {
                        // Download file
                        Self::simulate_file_download(&username, &metrics, &fixture).await?;
                        metrics.record_file_downloaded();
                    }
                    3 => {
                        // Launch door game
                        Self::simulate_door_launch(&username, &metrics, &fixture).await?;
                        metrics.record_door_launched();
                    }
                    _ => {}
                }

                // Record operation latency
                let latency = operation_start.elapsed().as_millis() as u64;
                metrics.add_latency(latency);
                metrics.record_operation_completed();
            } else {
                metrics.record_failed_login();
            }

            // Random think time (50-200ms) - generate before await
            let think_time = {
                let mut rng = rand::rng();
                50 + (rng.random::<u64>() % 150)
            };
            tokio::time::sleep(tokio::time::Duration::from_millis(think_time)).await;
        }

        Ok(())
    }

    /// Simulate user login
    async fn simulate_login(
        username: &str,
        _metrics: &Arc<LoadMetrics>,
        fixture: &Arc<BbsTestFixture>,
    ) -> Result<()> {
        // Find or create user
        let user = fixture.user_factory.find_by_username(username).await;

        if user.is_some() {
            tracing::trace!("User {} logged in", username);
            Ok(())
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
    }

    /// Simulate message posting
    async fn simulate_message_post(
        username: &str,
        _metrics: &Arc<LoadMetrics>,
        fixture: &Arc<BbsTestFixture>,
    ) -> Result<()> {
        let message_base = fixture.message_base_path();
        if message_base.exists() {
            tracing::trace!("{} posted message", username);
        }
        Ok(())
    }

    /// Simulate file upload
    async fn simulate_file_upload(
        username: &str,
        _metrics: &Arc<LoadMetrics>,
        fixture: &Arc<BbsTestFixture>,
    ) -> Result<()> {
        let file_area = fixture.file_areas_path();
        if file_area.exists() {
            tracing::trace!("{} uploaded file", username);
        }
        Ok(())
    }

    /// Simulate file download
    async fn simulate_file_download(
        username: &str,
        _metrics: &Arc<LoadMetrics>,
        fixture: &Arc<BbsTestFixture>,
    ) -> Result<()> {
        let file_area = fixture.file_areas_path();
        if file_area.exists() {
            tracing::trace!("{} downloaded file", username);
        }
        Ok(())
    }

    /// Simulate door game launch
    async fn simulate_door_launch(
        username: &str,
        _metrics: &Arc<LoadMetrics>,
        fixture: &Arc<BbsTestFixture>,
    ) -> Result<()> {
        let door_dir = fixture.door_directory_path();
        if door_dir.exists() {
            tracing::trace!("{} launched door", username);
        }
        Ok(())
    }
}

use std::sync::atomic::AtomicU64;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_generator_creation() {
        let generator = LoadGenerator::new().await.unwrap();
        assert!(generator.fixture().temp_path().exists());
        assert_eq!(generator.metrics().get_total_connections(), 0);
    }

    #[tokio::test]
    async fn test_simulate_single_user() {
        let generator = LoadGenerator::new().await.unwrap();
        let user = generator.fixture().create_regular_user().await.unwrap();

        let metrics = generator.metrics().clone();
        let fixture = generator.fixture().clone();

        let result = LoadGenerator::simulate_user(
            user.username,
            metrics,
            fixture,
            1, // 1 second
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Expensive test
    async fn test_small_load_test() {
        let generator = LoadGenerator::new().await.unwrap();
        let metrics = generator.run_load_test(3, 2).await.unwrap();

        // Verify some operations occurred
        assert!(metrics.get_total_connections() > 0);
    }

    #[tokio::test]
    #[ignore] // Very expensive test
    async fn test_stress_test_50_users() {
        let generator = LoadGenerator::new().await.unwrap();
        let metrics = generator.run_load_test(50, 60).await.unwrap();

        // Verify metrics
        assert!(metrics.get_total_connections() > 0);
        assert!(metrics.get_successful_logins() > 0);

        // Print summary
        metrics.print_summary();
    }
}
