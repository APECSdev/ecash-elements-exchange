// matching — off-chain order book: matching + atomic-swap settlement
//
// Per docs/HANDOFF.md §Phase 4 and the architecture decision: maintain an
// in-memory book of bids/asks, match on price-time priority, and for each
// matched pair construct + broadcast the on-chain atomic swap by compiling
// `order_book.simf` with the matched order's `param::` set.
//
// The on-chain/off-chain boundary lives in `settle.rs`: everything before
// `settle.rs::take()` is pure Rust with loops; everything after is a single
// Simplicity program instance per order UTXO (Simplicity is loop-free, so
// the book can never live on-chain).

pub mod book;
pub mod order;
pub mod settle;

pub use book::OrderBook;
pub use order::{Order, OrderId, OrderState, Side};
pub use settle::{settle_take, SettleReceipt};
