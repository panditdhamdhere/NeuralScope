use std::net::SocketAddr;

use neuralscope_server::{
    ai::infrastructure::create_llm_provider, api, db, events::application::EventBus, vector,
    AppConfig,
};
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;
    config.validate()?;

    if std::env::args().any(|arg| arg == "--migrate-only") {
        return db::run_migrations_only(&config).await.map_err(Into::into);
    }

    let fmt_layer = if config.use_json_logs() {
        tracing_subscriber::fmt::layer().json().boxed()
    } else {
        tracing_subscriber::fmt::layer().boxed()
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                if config.use_json_logs() {
                    "neuralscope_server=info,tower_http=info,sqlx=warn".into()
                } else {
                    "neuralscope_server=debug,tower_http=debug,sqlx=warn".into()
                }
            }),
        )
        .with(fmt_layer)
        .init();

    let addr: SocketAddr = config.server_addr().parse()?;

    info!(
        version = neuralscope_server::VERSION,
        environment = %config.environment,
        %addr,
        "Starting NeuralScope server"
    );

    let bundle = db::connect(&config).await?;
    let events = EventBus::new();

    let ai_provider = match create_llm_provider(&config) {
        Ok(provider) => {
            info!(provider = provider.name(), "AI provider initialized");
            Some(provider)
        }
        Err(error) => {
            tracing::warn!(%error, "AI provider not configured — chat endpoints disabled");
            None
        }
    };

    let embedding_provider = vector::infrastructure::create_embedding_provider(&config);
    let vector_service = std::sync::Arc::new(vector::application::VectorService::from_parts(
        embedding_provider,
        &config.qdrant_url,
        bundle.pool.clone(),
    ));

    match vector_service.health_check().await {
        Ok(()) => info!(url = %config.qdrant_url, "Qdrant vector store connected"),
        Err(error) => {
            tracing::warn!(%error, url = %config.qdrant_url, "Qdrant unavailable — vector search may fail");
        }
    }

    let state = api::AppState::new(
        config,
        bundle.pool,
        bundle.redis,
        events,
        ai_provider,
        Some(vector_service),
    );
    let app = api::create_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("NeuralScope server listening on {addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("NeuralScope server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => info!("Received Ctrl+C, shutting down"),
        () = terminate => info!("Received SIGTERM, shutting down"),
    }
}
