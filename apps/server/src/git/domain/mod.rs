use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A Git commit linked to a project.
#[derive(Debug, Clone)]
pub struct Commit {
    pub id: Uuid,
    pub project_id: Uuid,
    pub sha: String,
    pub author: String,
    pub message: String,
    pub branch: String,
    pub committed_at: DateTime<Utc>,
}

/// A deployment event correlated with a Git commit.
#[derive(Debug, Clone)]
pub struct Deployment {
    pub id: Uuid,
    pub project_id: Uuid,
    pub commit_sha: String,
    pub environment: String,
    pub deployed_at: DateTime<Utc>,
    pub deployed_by: Option<String>,
}
