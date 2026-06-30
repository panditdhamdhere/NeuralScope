use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use futures::StreamExt;
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

use crate::api::state::AppState;
use crate::auth::application::{ensure_project_member, AuthService};
use crate::auth::presentation::extractors::{
    API_KEY_HEADER, BETTER_AUTH_SESSION_COOKIE, SESSION_COOKIE,
};
use crate::AppError;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub project_id: Uuid,
    pub token: Option<String>,
}

/// `GET /ws` — WebSocket stream for real-time project events.
pub async fn ws_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let user = authenticate_ws(&headers, query.token.as_deref(), &state).await?;
    ensure_project_member(&state.db, user.id, query.project_id).await?;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, query.project_id)))
}

async fn authenticate_ws(
    headers: &HeaderMap,
    query_token: Option<&str>,
    state: &AppState,
) -> Result<crate::auth::domain::User, AppError> {
    let token = query_token
        .map(str::to_string)
        .or_else(|| extract_bearer_token(headers))
        .or_else(|| extract_session_cookie(headers));

    if let Some(token) = token {
        return AuthService::new(&state.db).validate_session(&token).await;
    }

    if let Some(api_key) = extract_api_key(headers) {
        let user_id = crate::auth::application::ApiKeyService::new(&state.db)
            .validate(&api_key)
            .await?;
        return AuthService::new(&state.db).get_user(user_id).await;
    }

    Err(AppError::Unauthorized(
        "WebSocket authentication required. Provide token query param, session cookie, Bearer token, or X-API-Key."
            .into(),
    ))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, project_id: Uuid) {
    let mut rx = state.events.subscribe();

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(envelope) if envelope.project_id == project_id => {
                        let json = match serde_json::to_string(&envelope) {
                            Ok(json) => json,
                            Err(_) => continue,
                        };

                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(RecvError::Lagged(skipped)) => {
                        let notice = serde_json::json!({
                            "type": "warning",
                            "payload": { "message": format!("Dropped {skipped} events — reconnect or reduce volume") }
                        });

                        if socket
                            .send(Message::Text(notice.to_string().into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(RecvError::Closed) => break,
                }
            }
            incoming = socket.next() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
}

fn extract_session_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;

    for part in cookie_header.split(';') {
        let part = part.trim();
        for name in [SESSION_COOKIE, BETTER_AUTH_SESSION_COOKIE] {
            if let Some(value) = part.strip_prefix(&format!("{name}=")) {
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}

fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get(API_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(str::to_string)
}

/// Maps WebSocket auth failures to HTTP responses before upgrade.
pub struct WsRejection(AppError);

impl From<AppError> for WsRejection {
    fn from(error: AppError) -> Self {
        Self(error)
    }
}

impl IntoResponse for WsRejection {
    fn into_response(self) -> Response {
        match self.0 {
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({
                    "error": { "message": msg, "code": 401 }
                })),
            )
                .into_response(),
            other => other.into_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_session_cookie_from_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "neuralscope.session_token=abc123; other=value"
                .parse()
                .unwrap(),
        );

        assert_eq!(
            extract_session_cookie(&headers).as_deref(),
            Some("abc123")
        );
    }
}
