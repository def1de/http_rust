use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::{Html, IntoResponse};
use axum::Router;
use tower_http::services::ServeDir;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;
use futures_util::{stream::StreamExt, sink::SinkExt};

#[derive(Clone)]
struct AppState {
    sockets: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Message>>>>,
    usernames: Arc<Mutex<HashMap<String, String>>>,
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let socket_id = Uuid::new_v4().to_string(); // Unique ID for each socket
    let (tx, mut rx) = mpsc::unbounded_channel();
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Store the socket in the shared state
    {
        let mut sockets = state.sockets.lock().unwrap();
        sockets.insert(socket_id.clone(), tx);
    }
    println!("New client connected with id: {}", socket_id);

    // Spawn a task to handle outgoing messages
    let socket_id_clone = socket_id.clone();
    let state_clone = state.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
        // Remove the socket from a HashMap when the connection is closed
        let mut sockets = state_clone.sockets.lock().unwrap();
        sockets.remove(&socket_id_clone);
        let mut usernames = state_clone.usernames.lock().unwrap();
        usernames.remove(&socket_id_clone);
    });

    let mut username = String::new();

    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                if username.is_empty() {
                    // let mut usernames = state.usernames.lock().unwrap();
                    // usernames.insert(socket_id.clone(), text.clone());
                    username = text.clone();

                    // Send a welcome message
                    let welcome_message = format!("System: Welcome to the chat, {}!", text);
                    let sockets = state.sockets.lock().unwrap();
                    if let Some(sender) = sockets.get(&socket_id) {
                        let _ = sender.send(Message::Text(welcome_message));
                    }
                    continue;
                }

                let broadcast_message = format!("{}: {}", username, text);

                let sockets = state.sockets.lock().unwrap();
                for (id, sender) in sockets.iter() {
                    if id != &socket_id {
                        let _ = sender.send(Message::Text(broadcast_message.clone()));
                    }
                }
            }
            Message::Close(_) => {
                break;
            }
            _ => {}
        }
    }

    // Cleanup
    let mut sockets = state.sockets.lock().unwrap();
    sockets.remove(&socket_id);
}

async fn index() -> Html<&'static str> {
    Html(std::include_str!("../templates/index.html"))
}

#[tokio::main]
async fn main() {
    let state = AppState {
        sockets: Arc::new(Mutex::new(HashMap::new())),
        usernames: Arc::new(Mutex::new(HashMap::new())),
    };

    // Websockets
    let app = Router::new()
        .route("/", axum::routing::get(index))
        .route("/ws", axum::routing::get(websocket_handler))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("192.168.1.233:1578").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}