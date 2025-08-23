mod websocket;
mod handlers;
mod database;
mod auth;

use axum::Router;
use tower_http::services::ServeDir;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::mpsc;
use axum::extract::ws::Message;

use websocket::{websocket_handler};
use handlers::{index, status, auth_get, auth_post, logout};
use database::Database;

#[derive(Clone)]
pub struct AppState {
    sockets: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Message>>>>,
    db: Database,
}

impl AppState {
    pub fn new() -> Self {
        let database: Database = Database::new();
        match database.create() {
            Ok(_) => println!("Database schema created successfully."),
            Err(e) => panic!("Error creating database schema: {}", e),
        }
        AppState {
            sockets: Arc::new(Mutex::new(HashMap::new())),
            db: database,
        }
    }

    pub fn get_connected_clients(&self) -> usize {
        let sockets = self.sockets.lock().unwrap();
        sockets.len()
    }

    pub fn db_action(&self) -> Database {
        self.db.clone()
    }
}

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let app = Router::new()
        .route("/", axum::routing::get(index))
        .route("/ws", axum::routing::get(websocket_handler))
        .route("/status", axum::routing::get(status))
        .route("/auth", axum::routing::get(auth_get).post(auth_post))
        .route("/logout", axum::routing::post(logout))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("192.168.1.233:1578").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}