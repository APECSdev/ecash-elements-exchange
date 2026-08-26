// ws — WebSocket book-change stream
//
// Each connected client subscribes to the shared `broadcast::Sender<BookEvent>`
// held in [`AppState`] and receives a JSON-encoded event on every insert/cancel.
// Per docs/HANDOFF.md the WS stream is the real-time book-change notification
// channel; the REST endpoints handle the mutating commands.

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use serde::Serialize;

use super::rest::{AppState, BookEvent};

/// `GET /ws` upgrade handler.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| client_loop(socket, state))
}

/// Drive a single WS client: send an initial snapshot then forward events
/// until the client disconnects or the broadcast channel closes.
async fn client_loop(mut socket: WebSocket, state: Arc<AppState>) {
    // Initial snapshot so late joiners see the current sequence.
    let snapshot = {
        let book = state.book.lock().await;
        BookEvent::Snapshot {
            sequence: book.sequence(),
            open_count: book.open_orders().len(),
        }
    };
    if send_json(&mut socket, &snapshot).await.is_err() {
        return;
    }

    let mut rx = state.events.subscribe();
    while let Ok(event) = rx.recv().await {
        if send_json(&mut socket, &event).await.is_err() {
            return;
        }
    }
}

async fn send_json<T: Serialize>(socket: &mut WebSocket, event: &T) -> Result<(), axum::Error> {
    let json = serde_json::to_string(event).unwrap_or_else(|_| "{}".into());
    socket.send(Message::Text(json.into())).await
}
