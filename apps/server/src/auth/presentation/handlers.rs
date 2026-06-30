use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::api::state::AppState;
use crate::auth::application::{ApiKeyService, AuthService, ProjectService};
use crate::auth::domain::{ApiKey, Project, User};
use crate::auth::presentation::extractors::{AuthUser, SESSION_COOKIE};
use crate::AppError;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub user: User,
    pub session: SessionInfo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyCreatedResponse {
    #[serde(flatten)]
    pub api_key: ApiKey,
    pub key: String,
}

#[derive(Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub meta: ListMeta,
}

#[derive(Serialize)]
pub struct ListMeta {
    pub total: usize,
}

/// `POST /api/v1/auth/register`
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Response, AppError> {
    let auth = AuthService::new(&state.db);
    let (user, session) = auth
        .register(&body.email, &body.password, body.name.as_deref())
        .await?;

    Ok(session_response(
        &state,
        AuthResponse {
            user,
            session: SessionInfo {
                expires_at: session.expires_at,
            },
        },
        &session.token,
        session.expires_at,
        StatusCode::CREATED,
    ))
}

/// `POST /api/v1/auth/login`
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<Response, AppError> {
    let ip = client_ip(&headers);
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok());

    let auth = AuthService::new(&state.db);
    let (user, session) = auth
        .login(&body.email, &body.password, ip.as_deref(), user_agent)
        .await?;

    Ok(session_response(
        &state,
        AuthResponse {
            user,
            session: SessionInfo {
                expires_at: session.expires_at,
            },
        },
        &session.token,
        session.expires_at,
        StatusCode::OK,
    ))
}

/// `POST /api/v1/auth/logout`
pub async fn logout(auth: AuthUser, State(state): State<AppState>) -> Result<StatusCode, AppError> {
    if let Some(token) = auth.session_token() {
        AuthService::new(&state.db).logout(token).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/v1/auth/me`
pub async fn me(auth: AuthUser) -> Json<User> {
    Json(auth.user)
}

/// `GET /api/v1/projects`
pub async fn list_projects(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<ListResponse<Project>>, AppError> {
    let projects = ProjectService::new(&state.db)
        .list_for_user(auth.user.id)
        .await?;
    let total = projects.len();

    Ok(Json(ListResponse {
        data: projects,
        meta: ListMeta { total },
    }))
}

/// `POST /api/v1/projects`
pub async fn create_project(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), AppError> {
    let project = ProjectService::new(&state.db)
        .create(auth.user.id, &body.name)
        .await?;

    Ok((StatusCode::CREATED, Json(project)))
}

/// `GET /api/v1/projects/:slug`
pub async fn get_project(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> Result<Json<Project>, AppError> {
    let project = ProjectService::new(&state.db)
        .get_by_slug(auth.user.id, &slug)
        .await?;

    Ok(Json(project))
}

/// `GET /api/v1/api-keys`
pub async fn list_api_keys(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<ListResponse<ApiKey>>, AppError> {
    let keys = ApiKeyService::new(&state.db).list(auth.user.id).await?;
    let total = keys.len();

    Ok(Json(ListResponse {
        data: keys,
        meta: ListMeta { total },
    }))
}

/// `POST /api/v1/api-keys`
pub async fn create_api_key(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<ApiKeyCreatedResponse>), AppError> {
    let (api_key, raw_key) = ApiKeyService::new(&state.db)
        .create(auth.user.id, &body.name)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiKeyCreatedResponse {
            api_key,
            key: raw_key,
        }),
    ))
}

/// `DELETE /api/v1/api-keys/:id`
pub async fn revoke_api_key(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Path(key_id): axum::extract::Path<uuid::Uuid>,
) -> Result<StatusCode, AppError> {
    ApiKeyService::new(&state.db)
        .revoke(auth.user.id, key_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

fn session_response<T: Serialize>(
    state: &AppState,
    body: T,
    token: &str,
    expires_at: DateTime<Utc>,
    status: StatusCode,
) -> Response {
    let max_age = (expires_at - Utc::now()).num_seconds().max(0);
    let secure = if state.config.is_production() {
        "; Secure"
    } else {
        ""
    };

    let cookie = format!(
        "{SESSION_COOKIE}={token}; HttpOnly; Path=/; SameSite=Lax; Max-Age={max_age}{secure}"
    );

    let mut response = (status, Json(body)).into_response();
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
    response
}

fn client_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .map(str::to_string)
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        })
}
