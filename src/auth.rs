use axum::{
    async_trait,
    extract::{FromRequestParts},
    http::{request::Parts},
    response::{Redirect},
};
use crate::AppState;

pub struct AuthenticatedUser {
    pub user_id: i64,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = Redirect;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract session token from cookies
        if let Some(cookie_header) = parts.headers.get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie in cookie_str.split(';') {
                    let cookie = cookie.trim();
                    if cookie.starts_with("session_token=") {
                        let token = &cookie[14..]; // Remove "session_token="
                        
                        if let Ok(Some(user_id)) = state.db_action().validate_session(token) {
                            return Ok(AuthenticatedUser { user_id });
                        }
                    }
                }
            }
        }
        
        Err(Redirect::to("/auth"))
    }
}