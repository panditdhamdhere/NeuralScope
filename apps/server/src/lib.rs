//! NeuralScope Server
//!
//! AI-powered developer observability platform backend.

pub mod ai;
pub mod api;
pub mod architecture;
pub mod auth;
pub mod chat;
pub mod common;
pub mod db;
pub mod events;
pub mod git;
pub mod incidents;
pub mod logs;
pub mod metrics;
pub mod network;
pub mod overview;
pub mod security;
pub mod traces;
pub mod vector;

pub use api::AppState;
pub use common::config::AppConfig;
pub use common::error::AppError;

/// Current server version from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
