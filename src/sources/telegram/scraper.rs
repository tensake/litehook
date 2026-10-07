use anyhow::anyhow;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{Duration, sleep};
use tokio_util::sync::CancellationToken;

use crate::events::Event;
use crate::sources::{create_client, fetch_url};

use super::TelegramScraperConfig;
use super::parser;

pub struct TelegramScraper {
    pub cfg: Arc<RwLock<TelegramScraperConfig>>,

    tx: mpsc::Sender<Event>,
    client: RwLock<reqwest::Client>,
    shutdown: CancellationToken,
}

impl TelegramScraper {
    pub async fn new(cfg: TelegramScraperConfig, tx: mpsc::Sender<Event>) -> anyhow::Result<Self> {
        tracing::info!("initializing listener {}", cfg.id);
        let client = create_client().await?;
        Ok(Self {
            cfg: Arc::new(RwLock::new(cfg)),
            tx,
            client: RwLock::new(client),
            shutdown: CancellationToken::new(),
        })
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let mut failures: u32 = 0;

        loop {
            let channel_url = self.cfg.read().await.channel_url.clone();

            match self.poll(&channel_url).await {
                Ok(()) => {
                    if failures > 0 {
                        tracing::info!(
                            "listener {} recovered after {} consecutive failures",
                            self.cfg.read().await.id,
                            failures
                        );
                    }
                    failures = 0;
                }
                Err(e) => {
                    failures += 1;
                    if failures == 1 {
                        tracing::warn!("poll failed for {}: {e}", self.cfg.read().await.id);
                    }

                    // Refresh the client for the next attempt.
                    match create_client().await {
                        Ok(client) => *self.client.write().await = client,
                        Err(e) => tracing::warn!("failed to refresh client: {e}"),
                    }
                }
            }

            // Back off exponentially
            let interval: u32 = self
                .cfg
                .read()
                .await
                .poll_interval
                .try_into()
                .unwrap_or(600)
                .max(1);
            let delay = if failures == 0 {
                interval
            } else {
                (2u32).saturating_pow(failures.min(10)).min(interval).max(1)
            };

            tokio::select! {
                // Shutdown handler
                _ = self.shutdown.cancelled() => {
                    self.stop().await?;
                    return Ok(());
                }
                _ = sleep(Duration::from_secs(delay.into())) => {}
            }
        }
    }

    pub async fn stop(&self) -> anyhow::Result<()> {
        let id = self.cfg.read().await.id.clone();
        tracing::info!("stopping listener with id {}", id);
        self.shutdown.cancel();
        Ok(())
    }

    /// Poll URL, parses the channel info and posts,
    /// stores state in database, and sends webhook notifications.
    async fn poll(&self, url: &str) -> anyhow::Result<()> {
        let client = self.client.read().await;
        let html = fetch_url(&client, url).await?;
        let page = match parser::parse_page(&html)? {
            Some(p) => p,
            None => return Err(anyhow!("invalid channel: {}", url)),
        };

        let webhook_url = self.cfg.read().await.webhook_url.clone();
        self.tx
            .send(Event::NewPosts(Box::new(page), webhook_url))
            .await?;

        Ok(())
    }
}
