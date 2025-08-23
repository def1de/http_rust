use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use uuid::Uuid;
use futures_util::{stream::StreamExt, sink::SinkExt};
use tokio::sync::mpsc;
use crate::AppState;
use crate::auth::AuthenticatedUser;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<AppState>,
    user: AuthenticatedUser
) -> impl IntoResponse {
    let username = user.username;
    ws.on_upgrade(move |socket| handle_socket(socket, state, username))
}

async fn handle_socket(socket: WebSocket, state: AppState, username: String) {
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
        println!("Disconnecting a client with id: {}", socket_id_clone);
        // Remove the socket from a HashMap when the connection is closed
        let mut sockets = state_clone.sockets.lock().unwrap();
        sockets.remove(&socket_id_clone);
    });

    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                state.db_action().insert_message(&text, &username).unwrap();

                let broadcast_message = format!("{}: {}", username, text);

                let sockets = state.sockets.lock().unwrap();
                for (id, sender) in sockets.iter() {
                    if id != &socket_id {
                        let _ = sender.send(Message::Text(broadcast_message.clone()));
                    }
                }
            }
            Message::Close(_) => {
                println!("Client {} disconnected", socket_id);
                break;
            }
            _ => {}
        }
    }

    // Cleanup
    let mut sockets = state.sockets.lock().unwrap();
    sockets.remove(&socket_id);
}