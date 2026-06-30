use std::env;
use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

/// Deployment environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Staging => "staging",
            Self::Production => "production",
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "staging" => Ok(Self::Staging),
            "production" | "prod" => Ok(Self::Production),
            other => Err(format!("Unknown environment: {other}")),
        }
    }
}

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub qdrant_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub environment: Environment,
    pub run_migrations: bool,
    pub ai_default_provider: String,
    pub gemini_api_key: Option<String>,
    pub groq_api_key: Option<String>,
    pub openrouter_api_key: Option<String>,
    pub ollama_base_url: String,
    pub jina_api_key: Option<String>,
    pub better_auth_secret: String,
    pub better_auth_url: String,
    pub cors_allowed_origins: Vec<String>,
    pub rate_limit_per_minute: u32,
    pub rate_limit_burst: u32,
}

const WEAK_SECRETS: &[&str] = &[
    "dev-secret-change-in-production",
    "change-me-in-production",
    "change-me",
    "secret",
    "changeme",
];

const DEFAULT_DATABASE_URL: &str = "postgres://neuralscope:neuralscope@localhost:5432/neuralscope";

impl AppConfig {
    /// Load configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are invalid.
    pub fn from_env() -> anyhow::Result<Self> {
        let environment = env::var("APP_ENV")
            .or_else(|_| env::var("RUST_ENV"))
            .unwrap_or_else(|_| "development".into())
            .parse()
            .map_err(|e: String| anyhow::anyhow!(e))?;

        let run_migrations = env::var("RUN_MIGRATIONS")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);

        Ok(Self {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://neuralscope:neuralscope@localhost:5432/neuralscope".into()
            }),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into()),
            qdrant_url: env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".into()),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid SERVER_PORT: {e}"))?,
            environment,
            run_migrations,
            ai_default_provider: env::var("AI_DEFAULT_PROVIDER").unwrap_or_else(|_| "gemini".into()),
            gemini_api_key: env::var("GEMINI_API_KEY").ok().filter(|k| !k.is_empty()),
            groq_api_key: env::var("GROQ_API_KEY").ok().filter(|k| !k.is_empty()),
            openrouter_api_key: env::var("OPENROUTER_API_KEY").ok().filter(|k| !k.is_empty()),
            ollama_base_url: env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".into()),
            jina_api_key: env::var("JINA_API_KEY").ok().filter(|k| !k.is_empty()),
            better_auth_secret: env::var("BETTER_AUTH_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-in-production".into()),
            better_auth_url: env::var("BETTER_AUTH_URL")
                .unwrap_or_else(|_| "http://localhost:3000".into()),
            cors_allowed_origins: parse_csv_env("CORS_ALLOWED_ORIGINS"),
            rate_limit_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
                .unwrap_or_else(|_| "120".into())
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid RATE_LIMIT_PER_MINUTE: {e}"))?,
            rate_limit_burst: env::var("RATE_LIMIT_BURST")
                .unwrap_or_else(|_| "30".into())
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid RATE_LIMIT_BURST: {e}"))?,
        })
    }

    /// Validates configuration for staging/production deployments.
    ///
    /// # Errors
    ///
    /// Returns an error when required production settings are missing or insecure.
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.is_production() && self.environment != Environment::Staging {
            return Ok(());
        }

        if WEAK_SECRETS
            .iter()
            .any(|weak| self.better_auth_secret.eq_ignore_ascii_case(weak))
            || self.better_auth_secret.len() < 32
        {
            anyhow::bail!(
                "BETTER_AUTH_SECRET must be at least 32 characters and not a default placeholder in {}",
                self.environment
            );
        }

        if self.database_url == DEFAULT_DATABASE_URL {
            anyhow::bail!("DATABASE_URL must be set to a production database in {}", self.environment);
        }

        if self.cors_allowed_origins.is_empty() {
            anyhow::bail!(
                "CORS_ALLOWED_ORIGINS must list at least one origin in {} (comma-separated)",
                self.environment
            );
        }

        if self.better_auth_url.starts_with("http://") && self.is_production() {
            tracing::warn!("BETTER_AUTH_URL uses http:// in production — use https://");
        }

        Ok(())
    }

    /// Returns the server bind address as `host:port`.
    #[must_use]
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }

    #[must_use]
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    #[must_use]
    pub fn use_json_logs(&self) -> bool {
        matches!(
            self.environment,
            Environment::Production | Environment::Staging
        )
    }
}

fn parse_csv_env(key: &str) -> Vec<String> {
    env::var(key)
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_addr_formats_correctly() {
        let config = AppConfig {
            database_url: String::new(),
            redis_url: String::new(),
            qdrant_url: String::new(),
            server_host: "127.0.0.1".into(),
            server_port: 8080,
            environment: Environment::Development,
            run_migrations: true,
            ai_default_provider: "gemini".into(),
            gemini_api_key: None,
            groq_api_key: None,
            openrouter_api_key: None,
            ollama_base_url: "http://localhost:11434".into(),
            jina_api_key: None,
            better_auth_secret: String::new(),
            better_auth_url: String::new(),
            cors_allowed_origins: vec![],
            rate_limit_per_minute: 120,
            rate_limit_burst: 30,
        };
        assert_eq!(config.server_addr(), "127.0.0.1:8080");
    }

    #[test]
    fn environment_parses_aliases() {
        assert_eq!(
            "prod".parse::<Environment>().expect("prod"),
            Environment::Production
        );
        assert_eq!(
            "dev".parse::<Environment>().expect("dev"),
            Environment::Development
        );
    }

    #[test]
    fn production_validation_rejects_weak_secret() {
        let config = AppConfig {
            database_url: "postgres://prod:prod@db:5432/neuralscope".into(),
            redis_url: "redis://redis:6379".into(),
            qdrant_url: "http://qdrant:6333".into(),
            server_host: "0.0.0.0".into(),
            server_port: 8080,
            environment: Environment::Production,
            run_migrations: false,
            ai_default_provider: "gemini".into(),
            gemini_api_key: None,
            groq_api_key: None,
            openrouter_api_key: None,
            ollama_base_url: "http://localhost:11434".into(),
            jina_api_key: None,
            better_auth_secret: "change-me-in-production".into(),
            better_auth_url: "https://app.example.com".into(),
            cors_allowed_origins: vec!["https://app.example.com".into()],
            rate_limit_per_minute: 120,
            rate_limit_burst: 30,
        };

        assert!(config.validate().is_err());
    }
}
