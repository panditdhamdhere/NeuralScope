use serde::{Deserialize, Serialize};

/// Project-scoped role for authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectRole {
    Owner,
    Admin,
    Viewer,
}

impl ProjectRole {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Viewer => "viewer",
        }
    }

    #[must_use]
    pub fn can_write(self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    #[must_use]
    pub fn can_admin(self) -> bool {
        self == Self::Owner
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "viewer" => Ok(Self::Viewer),
            other => Err(format!("Invalid role: {other}")),
        }
    }
}
