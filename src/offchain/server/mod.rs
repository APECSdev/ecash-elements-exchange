// server — REST + WebSocket front for the matching engine
//
// Exposes the in-memory [`OrderBook`] over HTTP (axum) for order
// submission/cancel/snapshot and a WebSocket stream of book changes
// (per docs/HANDOFF.md REST/WebSocket front requirement).

pub mod rest;
pub mod ws;

pub use rest::{AppState, ServerHandle, start_server};
