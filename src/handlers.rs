use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::extract::{State};
use axum::Form;
use axum::Json;
use crate::AppState;
use crate::auth::AuthenticatedUser;
use sha2::{Sha256, Digest};
use uuid::Uuid;
use askama::Template;

#[derive(serde::Serialize)]
pub struct StatusResponse {
    connected_clients: usize,
}

pub async fn index(user: AuthenticatedUser) -> Response {
    let template = crate::template::IndexTemplate {
        username: &user.username,
    };
    match template.render() {
        Ok(body) => Html(body).into_response(),
        Err(_e) => Html("<h1>Template Error</h1>".to_string()).into_response(),
    }
}

pub async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let response = StatusResponse {
        connected_clients: state.get_connected_clients(),
    };
    Json(response)
}

pub async fn auth_get() -> Html<&'static str> {
    Html(std::include_str!("../templates/auth.html"))
}

#[derive(serde::Deserialize)]
pub struct AuthForm {
    username: String,
    password: String,
}

pub async fn auth_post(
    State(state): State<AppState>,
    Form(form): Form<AuthForm>
) -> Response {

    // Extract username and password from the form
    let username = form.username.as_str();
    let password = form.password.as_str();
    // Hash the password
    let hash = format!("{:x}", Sha256::digest(password.as_bytes()));

    {
        match state.db_action().get_user(username) {
            Ok(Some(_)) => {
                // User exists, check password
                if state.db_action().check_password(username, &hash) {
                    return start_session(&state, &username)
                }
            }
            Ok(None) => {
                // User not found, register new user
                state.db_action().add_user(username, &hash).unwrap();
                // Repeat the check to authorize the new user
                if state.db_action().check_password(username, &hash) {
                    return start_session(&state, &username)
                }
            }
            Err(e) => {
                return Html(format!("<p>Error: {}</p>", e)).into_response()
            }
        }
    }
    Html("<p>Authentication logic not implemented yet.</p>".to_string()).into_response()
}

fn generate_session_token() -> String {
    Uuid::new_v4().to_string()
}

fn start_session(state: &AppState, username: &str) -> Response {
    let session_token = generate_session_token();
                let user_id = match state.db_action().get_user(username).ok().and_then(|opt| opt.map(|(id, _)| id)) {
                    Some(id) => id,
                    None => return Html("<p>Invalid credentials</p>".to_string()).into_response(),
                };
                
                if state.db_action().create_session(user_id, &session_token).is_ok() {
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        "Set-Cookie",
                        HeaderValue::from_str(&format!("session_token={}; HttpOnly; Path=/", session_token)).unwrap()
                    );
                    return (headers, Redirect::to("/")).into_response();
                }
                Html("<p>Invalid credentials</p>".to_string()).into_response()
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap
) -> impl IntoResponse {
    // Extract and delete session
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("session_token=") {
                    let token = &cookie[14..];
                    let _ = state.db_action().delete_session(token);
                    break;
                }
            }
        }
    }
    
    let mut headers = HeaderMap::new();
    headers.insert(
        "Set-Cookie",
        HeaderValue::from_str("session_token=; HttpOnly; Path=/; Max-Age=0").unwrap()
    );
    (headers, Redirect::to("/auth"))
}