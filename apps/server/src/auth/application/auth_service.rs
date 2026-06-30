use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::application::crypto::{generate_session_token, hash_password, verify_password};
use crate::auth::domain::{Session, User};
use crate::AppError;

const SESSION_TTL_DAYS: i64 = 7;
const CREDENTIAL_PROVIDER: &str = "credential";

/// Handles user registration, login, logout, and session validation.
pub struct AuthService<'a> {
    pool: &'a PgPool,
}

impl<'a> AuthService<'a> {
    #[must_use]
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Registers a new user with email and password credentials.
    pub async fn register(
        &self,
        email: &str,
        password: &str,
        name: Option<&str>,
    ) -> Result<(User, Session), AppError> {
        if password.len() < 8 {
            return Err(AppError::Validation(
                "Password must be at least 8 characters".into(),
            ));
        }

        let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(self.pool)
            .await?;

        if existing > 0 {
            return Err(AppError::Validation("Email already registered".into()));
        }

        let user_id = Uuid::new_v4();
        let account_id = Uuid::new_v4().to_string();
        let password_hash = hash_password(password)?;
        let now = Utc::now();

        let mut tx = self.pool.begin().await?;

        let user = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (id, email, name, email_verified, created_at, updated_at)
            VALUES ($1, $2, $3, false, $4, $4)
            RETURNING id, email, name, email_verified, image, created_at
            "#,
        )
        .bind(user_id)
        .bind(email)
        .bind(name)
        .bind(now)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO accounts (id, account_id, provider_id, user_id, password, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $6)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(account_id)
        .bind(CREDENTIAL_PROVIDER)
        .bind(user_id)
        .bind(password_hash)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        let session = self
            .create_session_in_tx(&mut tx, user_id, None, None)
            .await?;

        tx.commit().await?;

        Ok((user.into(), session))
    }

    /// Authenticates a user and creates a new session.
    pub async fn login(
        &self,
        email: &str,
        password: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(User, Session), AppError> {
        let row = sqlx::query_as::<_, CredentialRow>(
            r#"
            SELECT u.id, u.email, u.name, u.email_verified, u.image, u.created_at, a.password
            FROM users u
            INNER JOIN accounts a ON a.user_id = u.id
            WHERE u.email = $1 AND a.provider_id = $2
            "#,
        )
        .bind(email)
        .bind(CREDENTIAL_PROVIDER)
        .fetch_optional(self.pool)
        .await?;

        let Some(row) = row else {
            return Err(AppError::Unauthorized("Invalid email or password".into()));
        };

        if !verify_password(password, &row.password)? {
            return Err(AppError::Unauthorized("Invalid email or password".into()));
        }

        let session = self.create_session(row.id, ip_address, user_agent).await?;

        Ok((row.into_user(), session))
    }

    /// Validates a session token and returns the associated user.
    pub async fn validate_session(&self, token: &str) -> Result<User, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT u.id, u.email, u.name, u.email_verified, u.image, u.created_at
            FROM sessions s
            INNER JOIN users u ON u.id = s.user_id
            WHERE s.token = $1 AND s.expires_at > NOW()
            "#,
        )
        .bind(token)
        .fetch_optional(self.pool)
        .await?;

        row.map(Into::into)
            .ok_or_else(|| AppError::Unauthorized("Invalid or expired session".into()))
    }

    /// Revokes a session token.
    pub async fn logout(&self, token: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM sessions WHERE token = $1")
            .bind(token)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    /// Fetches a user by ID.
    pub async fn get_user(&self, user_id: Uuid) -> Result<User, AppError> {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, email, name, email_verified, image, created_at FROM users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(self.pool)
        .await?
        .map(Into::into)
        .ok_or_else(|| AppError::NotFound("User not found".into()))
    }

    async fn create_session(
        &self,
        user_id: Uuid,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<Session, AppError> {
        let mut tx = self.pool.begin().await?;
        let session = self
            .create_session_in_tx(&mut tx, user_id, ip_address, user_agent)
            .await?;
        tx.commit().await?;
        Ok(session)
    }

    async fn create_session_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<Session, AppError> {
        let session_id = Uuid::new_v4().to_string();
        let token = generate_session_token();
        let now = Utc::now();
        let expires_at = now + Duration::days(SESSION_TTL_DAYS);

        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, token, expires_at, ip_address, user_agent, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
            "#,
        )
        .bind(&session_id)
        .bind(user_id)
        .bind(&token)
        .bind(expires_at)
        .bind(ip_address)
        .bind(user_agent)
        .bind(now)
        .execute(&mut **tx)
        .await?;

        Ok(Session {
            id: session_id,
            user_id,
            token,
            expires_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    name: Option<String>,
    email_verified: bool,
    image: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            name: row.name,
            email_verified: row.email_verified,
            image: row.image,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CredentialRow {
    id: Uuid,
    email: String,
    name: Option<String>,
    email_verified: bool,
    image: Option<String>,
    created_at: DateTime<Utc>,
    password: String,
}

impl CredentialRow {
    fn into_user(self) -> User {
        User {
            id: self.id,
            email: self.email,
            name: self.name,
            email_verified: self.email_verified,
            image: self.image,
            created_at: self.created_at,
        }
    }
}
