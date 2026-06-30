//! Auth application layer: register, login, sessions, API keys, and projects.

pub mod access;
pub mod api_key_service;
pub mod auth_service;
pub mod crypto;
pub mod project_service;

pub use access::{ensure_project_admin, ensure_project_member, ensure_project_writer};

pub use api_key_service::ApiKeyService;
pub use auth_service::AuthService;
pub use project_service::ProjectService;
