// rest — REST API front for the matching engine (axum)
//
// Exposes the in-memory [`OrderBook`] over HTTP for order submission/cancel/
// snapshot. Designed to run behind an `Arc<AppState>` shared with the
// WebSocket layer. Per docs/HANDOFF.md the matching engine + REST/WS front
// together form the off-chain coordinator; settlement (on-chain atomic
// swap) is triggered separately via `matching::settle_take`.
//
// Routes:
//   POST /order            — submit a new order, returns {id, trades}
//   DELETE /order/:id      — cancel an order
//   GET  /order/:id        — fetch a single order
//   GET  /orders           — list all open orders
//   GET  /book/:maker/:taker — best bid + best ask for a pair
//   WS   /ws               — book change stream (see ws.rs)

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::sync::broadcast;

use simplex::simplicityhl::elements::AssetId;

use crate::offchain::matching::{Order, OrderBook, OrderId, OrderState, Side};

/// Shared state held behind `Arc` across HTTP + WS handlers.
pub struct AppState {
    pub book: Mutex<OrderBook>,
    /// Broadcasts a `BookEvent` on every insert/cancel so WS subscribers
    /// receive the new book sequence.
    pub events: broadcast::Sender<BookEvent>,
}

impl AppState {
    /// Create fresh state with an empty book and an event channel.
    #[must_use]
    pub fn new() -> Self {
        let (events, _) = broadcast::channel(256);
        Self {
            book: Mutex::new(OrderBook::new()),
            events,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// A handle to the running server (graceful shutdown via `Drop`/`stop()`).
pub struct ServerHandle {
    shutdown: tokio::sync::oneshot::Sender<()>,
    join: tokio::task::JoinHandle<()>,
}

impl ServerHandle {
    /// Tell the server to stop and await termination.
    pub async fn stop(self) {
        let _ = self.shutdown.send(());
        let _ = self.join.await;
    }
}

/// Start the REST + WS server on the given bind address.
///
/// # Errors
/// Returns [`crate::offchain::core::Error::Server`] if the server fails to
/// bind or the runtime errors.
pub async fn start_server(addr: &str) -> crate::offchain::core::Result<ServerHandle> {
    let state = Arc::new(AppState::new());
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| crate::offchain::core::Error::Server(e.to_string()))?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let join = tokio::spawn(async move {
        let _ = axum::serve(listener, app).with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        }).await;
    });
    Ok(ServerHandle { shutdown: shutdown_tx, join })
}

/// Build the router with the shared state. Public so tests can mount it.
#[must_use]
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/order", post(create_order).get(list_orders))
        .route("/order/{id}", get(get_order).delete(cancel_order))
        .route("/book/{maker}/{taker}", get(best_quote))
        .route("/ws", get(crate::offchain::server::ws::ws_handler))
        .with_state(state)
}

// ---- request/response DTOs ----

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub id: OrderId,
    pub side: Side,
    pub maker_asset: String,
    pub maker_amount: u64,
    pub taker_asset: String,
    pub taker_amount: u64,
    pub maker_script_hash: [u8; 32],
    pub maker_pubkey: [u8; 32],
    pub expiry: u32,
}

#[derive(Debug, Serialize)]
pub struct CreateOrderResponse {
    pub id: OrderId,
    pub trades: Vec<TradeDto>,
    pub state: OrderState,
}

#[derive(Debug, Serialize)]
pub struct TradeDto {
    pub maker_id: OrderId,
    pub maker_fill: u64,
    pub taker_fill: u64,
}

#[derive(Debug, Serialize)]
pub struct OrderDto {
    pub id: OrderId,
    pub side: Side,
    pub maker_asset: String,
    pub maker_amount: u64,
    pub taker_asset: String,
    pub taker_amount: u64,
    pub expiry: u32,
    pub state: OrderState,
}

#[derive(Debug, Serialize)]
pub struct BestQuoteDto {
    pub best_bid: Option<(u64, u64)>,
    pub best_ask: Option<(u64, u64)>,
}

/// Book change event broadcast to WS subscribers.
#[derive(Debug, Clone, Serialize)]
pub enum BookEvent {
    Insert { sequence: u64, id: OrderId },
    Cancel { sequence: u64, id: OrderId },
    Snapshot { sequence: u64, open_count: usize },
}

// ---- handlers ----

async fn create_order(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateOrderRequest>,
) -> Response {
    let maker_asset = match crate::offchain::core::asset::parse_asset_id(&req.maker_asset) {
        Ok(a) => a,
        Err(e) => return (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };
    let taker_asset = match crate::offchain::core::asset::parse_asset_id(&req.taker_asset) {
        Ok(a) => a,
        Err(e) => return (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };
    let order = Order {
        id: req.id.clone(),
        side: req.side,
        maker_asset,
        maker_amount: req.maker_amount,
        taker_asset,
        taker_amount: req.taker_amount,
        maker_script_hash: req.maker_script_hash,
        maker_pubkey: req.maker_pubkey,
        expiry: req.expiry,
        utxo: None,
        state: OrderState::Open,
    };
    let mut book = state.book.lock().await;
    let trades = book.insert(order.clone());
    let sequence = book.sequence();
    drop(book);
    let _ = state.events.send(BookEvent::Insert { sequence, id: req.id.clone() });
    let trade_dtos = trades.iter().map(|t| TradeDto {
        maker_id: t.maker.id.clone(),
        maker_fill: t.maker_fill,
        taker_fill: t.taker_fill,
    }).collect();
    let state_field = state.book.lock().await.get(&req.id).map(|o| o.state).unwrap_or(OrderState::Open);
    Json(CreateOrderResponse { id: req.id, trades: trade_dtos, state: state_field }).into_response()
}

async fn cancel_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<OrderId>,
) -> Response {
    let mut book = state.book.lock().await;
    let ok = book.cancel(&id);
    let sequence = book.sequence();
    drop(book);
    if ok {
        let _ = state.events.send(BookEvent::Cancel { sequence, id });
        (axum::http::StatusCode::OK, "cancelled").into_response()
    } else {
        (axum::http::StatusCode::NOT_FOUND, "order not open").into_response()
    }
}

async fn get_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<OrderId>,
) -> Response {
    let book = state.book.lock().await;
    match book.get(&id) {
        Some(o) => Json(order_to_dto(o)).into_response(),
        None => (axum::http::StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

async fn list_orders(State(state): State<Arc<AppState>>) -> Json<Vec<OrderDto>> {
    let book = state.book.lock().await;
    let dtos = book.open_orders().into_iter().map(order_to_dto).collect();
    Json(dtos)
}

async fn best_quote(
    State(state): State<Arc<AppState>>,
    Path((maker, taker)): Path<(String, String)>,
) -> Response {
    let maker_asset = match crate::offchain::core::asset::parse_asset_id(&maker) {
        Ok(a) => a,
        Err(e) => return (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };
    let taker_asset = match crate::offchain::core::asset::parse_asset_id(&taker) {
        Ok(a) => a,
        Err(e) => return (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };
    let book = state.book.lock().await;
    Json(BestQuoteDto {
        best_bid: book.best_bid(maker_asset, taker_asset),
        best_ask: book.best_ask(maker_asset, taker_asset),
    }).into_response()
}

fn order_to_dto(o: &Order) -> OrderDto {
    OrderDto {
        id: o.id.clone(),
        side: o.side,
        maker_asset: o.maker_asset_hex(),
        maker_amount: o.maker_amount,
        taker_asset: o.taker_asset_hex(),
        taker_amount: o.taker_amount,
        expiry: o.expiry,
        state: o.state,
    }
}

// Re-export AssetId so the unused-import lint stays clean if we add typed
// endpoints later that take raw `AssetId` in path params.
#[allow(unused_imports)]
use AssetId as _AssetId;
