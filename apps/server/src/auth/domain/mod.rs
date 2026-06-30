//! Auth domain: entities, value objects, and repository traits.

mod entities;
mod role;

pub use entities::{ApiKey, Project, Session, User};
pub use role::ProjectRole;
