use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::AppError;

/// Trace/span completion status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TraceStatus {
    #[default]
    Ok,
    Error,
    Unset,
}

impl TraceStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Unset => "unset",
        }
    }
}

impl fmt::Display for TraceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TraceStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ok" => Ok(Self::Ok),
            "error" => Ok(Self::Error),
            "unset" => Ok(Self::Unset),
            other => Err(AppError::Validation(format!("Invalid trace status: {other}"))),
        }
    }
}
