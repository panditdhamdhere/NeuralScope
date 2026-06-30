use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use tower_cookies::Cookies;

use crate::api::state::AppState;
use crate::auth::application::{ApiKeyService, AuthService};
use crate::auth::domain::User;
use crate::AppError;

pub const SESSION_COOKIE: &str = "neuralscope.session_token";
pub const BETTER_AUTH_SESSION_COOKIE: &str = "better-auth.session_token";
pub const API_KEY_HEADER: &str = "x-api-key";

/// How the current request was authenticated.
#[derive(Debug, Clone)]
pub enum AuthMethod {
    Session { token: String },
    ApiKey,
}

/// Authenticated user extracted from session cookie, bearer token, or API key.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user: User,
    pub method: AuthMethod,
}

impl AuthUser {
    #[must_use]
    pub fn session_token(&self) -> Option<&str> {
        match &self.method {
            AuthMethod::Session { token } => Some(token.as_str()),
            AuthMethod::ApiKey => None,
        }
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(token) = extract_session_token(parts) {
            let auth = AuthService::new(&state.db);
            let user = auth.validate_session(&token).await?;
            return Ok(Self {
                user,
                method: AuthMethod::Session { token },
            });
        }

        if let Some(api_key) = extract_api_key(parts) {
            let service = ApiKeyService::new(&state.db);
            let user_id = service.validate(&api_key).await?;
            let auth = AuthService::new(&state.db);
            let user = auth.get_user(user_id).await?;
            return Ok(Self {
                user,
                method: AuthMethod::ApiKey,
            });
        }

        Err(AppError::Unauthorized(
            "Authentication required. Provide a session cookie, Bearer token, or X-API-Key header."
                .into(),
        ))
    }
}

fn extract_session_token(parts: &Parts) -> Option<String> {
    if let Some(token) = extract_bearer_token(parts) {
        return Some(token);
    }

    if let Some(cookies) = parts.extensions.get::<Cookies>() {
        for name in [SESSION_COOKIE, BETTER_AUTH_SESSION_COOKIE] {
            if let Some(cookie) = cookies.get(name) {
                let value = cookie.value().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }

    None
}

fn extract_bearer_token(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
}

fn extract_api_key(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get(API_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(str::to_string)
}

/// Maps auth rejections to HTTP responses with correct status codes.
pub struct AuthRejection(AppError);

impl From<AppError> for AuthRejection {
    fn from(error: AppError) -> Self {
        Self(error)
    }
}

impl axum::response::IntoResponse for AuthRejection {
    fn into_response(self) -> axum::response::Response {
        use axum::{http::StatusCode, Json};

        match self.0 {
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": { "message": msg, "code": 401 }
                })),
            )
                .into_response(),
            other => other.into_response(),
        }
    }
}
