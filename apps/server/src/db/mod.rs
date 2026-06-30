//! Database layer — connection pools, migrations, and shared repository utilities.

pub mod bootstrap;
pub mod migrate;
pub mod pool;
pub mod redis_client;

pub use bootstrap::{connect, run_migrations_only};
