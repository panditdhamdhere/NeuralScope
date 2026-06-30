use std::sync::Arc;
use std::time::Instant;

use ::redis::aio::ConnectionManager;
use sqlx::PgPool;

use crate::ai::domain::LlmProvider;
use crate::common::config::AppConfig;
use crate::events::application::EventBus;
use crate::vector::application::VectorService;

/// Shared application state injected into all HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub events: EventBus,
    pub ai_provider: Option<Arc<dyn LlmProvider>>,
    pub vector: Option<Arc<VectorService>>,
    pub start_time: Instant,
}

impl AppState {
    #[must_use]
    pub fn new(
        config: AppConfig,
        db: PgPool,
        redis: ConnectionManager,
        events: EventBus,
        ai_provider: Option<Arc<dyn LlmProvider>>,
        vector: Option<Arc<VectorService>>,
    ) -> Self {
        Self {
            config,
            db,
            redis,
            events,
            ai_provider,
            vector,
            start_time: Instant::now(),
        }
    }

    #[must_use]
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
