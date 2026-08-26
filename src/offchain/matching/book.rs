// book — in-memory order book with price-time priority matching
//
// Simplicity is loop-free, so the order book can NEVER live on-chain. This
// in-memory engine is the off-chain counterpart to `order_book.simf`: it
// matches orders and emits [`Trade`]s, which the settler then turns into
// on-chain atomic swaps (one contract instance per matched order).
//
// Matching policy: price-time priority (FIFO within equal price), as stated
// in docs/HANDOFF.md. Bids are sorted best (highest price) first; asks are
// sorted best (lowest price) first. A cross occurs when the best bid price
// >= best ask price.

use std::collections::BTreeMap;

use super::order::{Order, OrderId, OrderState, Side};

/// A matched pair: one maker order and the (possibly partial) fill.
#[derive(Debug, Clone)]
pub struct Trade {
    /// The maker order being taken.
    pub maker: Order,
    /// Amount of `maker_asset` the taker takes from the order UTXO.
    pub maker_fill: u64,
    /// Amount of `taker_asset` the taker pays to the maker.
    pub taker_fill: u64,
}

/// The in-memory order book. Cloning is cheap-ish (orders are Arc-light) but
/// the book is intended to live behind a mutex/`Arc` in the server.
#[derive(Debug, Default)]
pub struct OrderBook {
    /// (asset_pair, price) -> queue of order ids at that price, FIFO.
    /// Bids: price is `taker/maker` — higher is better.
    /// Asks: price is `taker/maker` — lower is better.
    bids: BTreeMap<(AssetPair, PriceKey), Vec<OrderId>>,
    asks: BTreeMap<(AssetPair, PriceKey), Vec<OrderId>>,
    orders: std::collections::HashMap<OrderId, Order>,
    sequence: u64,
}

/// Canonical ordering of an asset pair so (A,B) and (B,A) share a bucket.
type AssetPair = ([u8; 32], [u8; 32]);

/// Newtype around the rational price so BTreeMap ordering works.
/// For bids we want highest-first, so we store `u64::MAX - price_taker` as
/// the key (reverse-ordered); for asks lowest-first, store `price_taker`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct PriceKey(u64);

fn pair_key(a: simplex::simplicityhl::elements::AssetId, b: simplex::simplicityhl::elements::AssetId) -> AssetPair {
    let a_bytes = a.into_inner().to_byte_array();
    let b_bytes = b.into_inner().to_byte_array();
    if a_bytes <= b_bytes {
        (a_bytes, b_bytes)
    } else {
        (b_bytes, a_bytes)
    }
}

impl OrderBook {
    /// Create an empty book.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a new order. Returns any trades it immediately crosses against.
    ///
    /// The order is stored regardless; if it crosses, the crossed counter-order
    /// is removed from the book (its `state` set to `Taken`).
    #[must_use]
    pub fn insert(&mut self, order: Order) -> Vec<Trade> {
        let id = order.id.clone();
        let pair = pair_key(order.maker_asset, order.taker_asset);
        let price_taker = order.taker_amount;
        let mut trades = Vec::new();

        // Try to cross against the opposite side. v1: one fill per insert.
        match order.side {
            Side::Buy => {
                // A buy crosses asks whose price (taker/maker) <= the buyer's
                // price. Asks are keyed by PriceKey(taker_amount) ascending,
                // so best ask = first entry with key <= price_taker.
                let best = self
                    .asks
                    .range((pair, PriceKey(0))..=(pair, PriceKey(price_taker)))
                    .next()
                    .and_then(|(_, q)| q.first().cloned());
                if let Some(maker_id) = best {
                    let maker = self.orders.get(&maker_id).cloned();
                    if let Some(maker) = maker {
                        let fill = order.maker_amount.min(maker.maker_amount);
                        let taker_fill = fill * order.taker_amount / order.maker_amount;
                        let maker_taker_amount = maker.taker_amount;
                        trades.push(Trade { maker, maker_fill: fill, taker_fill });
                        remove_from_book(&mut self.asks, pair, PriceKey(maker_taker_amount), &maker_id);
                        if let Some(o) = self.orders.get_mut(&maker_id) {
                            o.state = OrderState::Taken;
                        }
                    }
                }
            }
            Side::Sell => {
                // A sell crosses bids whose price >= the seller's price.
                // Bids are keyed by PriceKey(u64::MAX - taker_amount) so that
                // highest taker_amount (best bid) sorts first. A bid crosses
                // when bid.taker_amount >= sell_price, i.e. its key <=
                // u64::MAX - sell_price.
                let bound = u64::MAX - price_taker;
                let best = self
                    .bids
                    .range((pair, PriceKey(0))..=(pair, PriceKey(bound)))
                    .next()
                    .and_then(|(_, q)| q.first().cloned());
                if let Some(maker_id) = best {
                    let maker = self.orders.get(&maker_id).cloned();
                    if let Some(maker) = maker {
                        let fill = order.maker_amount.min(maker.maker_amount);
                        let taker_fill = fill * maker.taker_amount / maker.maker_amount;
                        let maker_taker_amount = maker.taker_amount;
                        trades.push(Trade { maker, maker_fill: fill, taker_fill });
                        remove_from_book(&mut self.bids, pair, PriceKey(u64::MAX - maker_taker_amount), &maker_id);
                        if let Some(o) = self.orders.get_mut(&maker_id) {
                            o.state = OrderState::Taken;
                        }
                    }
                }
            }
        }

        // Store the new order and index it if it didn't fully cross.
        self.orders.insert(id.clone(), order.clone());
        if trades.is_empty() || order.state != OrderState::Taken {
            let pk = match order.side {
                Side::Buy => PriceKey(u64::MAX - order.taker_amount),
                Side::Sell => PriceKey(order.taker_amount),
            };
            let book = match order.side {
                Side::Buy => &mut self.bids,
                Side::Sell => &mut self.asks,
            };
            book.entry((pair, pk)).or_default().push(id);
        }
        self.sequence += 1;
        trades
    }

    /// Look up an order by id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Order> {
        self.orders.get(id)
    }

    /// Cancel an open order (removes from book, sets state `Cancelled`).
    /// Returns `true` if the order was open and is now cancelled.
    pub fn cancel(&mut self, id: &str) -> bool {
        if let Some(order) = self.orders.get_mut(id) {
            if order.state != OrderState::Open {
                return false;
            }
            order.state = OrderState::Cancelled;
            let pair = pair_key(order.maker_asset, order.taker_asset);
            let pk = match order.side {
                Side::Buy => PriceKey(u64::MAX - order.taker_amount),
                Side::Sell => PriceKey(order.taker_amount),
            };
            let book = match order.side {
                Side::Buy => &mut self.bids,
                Side::Sell => &mut self.asks,
            };
            if let Some(q) = book.get_mut(&(pair, pk)) {
                q.retain(|oid| oid != id);
                if q.is_empty() {
                    book.remove(&(pair, pk));
                }
            }
            self.sequence += 1;
            true
        } else {
            false
        }
    }

    /// Snapshot all open orders.
    #[must_use]
    pub fn open_orders(&self) -> Vec<&Order> {
        self.orders.values().filter(|o| o.state == OrderState::Open).collect()
    }

    /// Best bid price (highest) for a pair, if any.
    #[must_use]
    pub fn best_bid(&self, maker_asset: simplex::simplicityhl::elements::AssetId, taker_asset: simplex::simplicityhl::elements::AssetId) -> Option<(u64, u64)> {
        let pair = pair_key(maker_asset, taker_asset);
        self.bids
            .range((pair, PriceKey(0))..)
            .next()
            .and_then(|(_, q)| q.first())
            .and_then(|id| self.orders.get(id))
            .map(|o| o.price())
    }

    /// Best ask price (lowest) for a pair, if any.
    #[must_use]
    pub fn best_ask(&self, maker_asset: simplex::simplicityhl::elements::AssetId, taker_asset: simplex::simplicityhl::elements::AssetId) -> Option<(u64, u64)> {
        let pair = pair_key(maker_asset, taker_asset);
        self.asks
            .range((pair, PriceKey(0))..)
            .next()
            .and_then(|(_, q)| q.first())
            .and_then(|id| self.orders.get(id))
            .map(|o| o.price())
    }

    /// Monotonic sequence number (for `server/` change notifications).
    #[must_use]
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

/// Remove an order id from a price level in the given book, deleting the
/// price level entirely if it becomes empty.
fn remove_from_book(
    book: &mut BTreeMap<(AssetPair, PriceKey), Vec<OrderId>>,
    pair: AssetPair,
    pk: PriceKey,
    id: &OrderId,
) {
    if let Some(q) = book.get_mut(&(pair, pk)) {
        q.retain(|oid| oid != id);
        if q.is_empty() {
            book.remove(&(pair, pk));
        }
    }
}
