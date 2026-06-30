use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::AppError;

/// Supported metric unit types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MetricUnit {
    Count,
    Percent,
    Bytes,
    Milliseconds,
    RequestsPerSecond,
}

impl MetricUnit {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::Percent => "percent",
            Self::Bytes => "bytes",
            Self::Milliseconds => "milliseconds",
            Self::RequestsPerSecond => "requests_per_second",
        }
    }
}

impl fmt::Display for MetricUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for MetricUnit {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "count" => Ok(Self::Count),
            "percent" | "percentage" => Ok(Self::Percent),
            "bytes" => Ok(Self::Bytes),
            "milliseconds" | "ms" => Ok(Self::Milliseconds),
            "requests_per_second" | "rps" => Ok(Self::RequestsPerSecond),
            other => Err(AppError::Validation(format!("Invalid metric unit: {other}"))),
        }
    }
}
