// Unit tests for the in-memory order book matching engine (no chain needed).

use ecash_elements_exchange::offchain::matching::{OrderBook, Side};
use ecash_elements_exchange::offchain::matching::order::OrderState;
use ecash_elements_exchange::offchain::test_support::make_order;

fn asset(byte: u8) -> simplex::simplicityhl::elements::AssetId {
    simplex::simplicityhl::elements::AssetId::from_byte_array([byte; 32])
}

#[test]
fn insert_two_non_crossing_orders_both_open() {
    let mut book = OrderBook::new();
    let a = asset(1);
    let b = asset(2);
    let o1 = make_order("o1".into(), Side::Buy, a, 100, b, 50, [0; 32], [1; 32], 1_000_000);
    let o2 = make_order("o2".into(), Side::Sell, a, 100, b, 60, [0; 32], [2; 32], 1_000_000);
    let t1 = book.insert(o1);
    let t2 = book.insert(o2);
    assert!(t1.is_empty(), "first insert should not cross");
    assert!(t2.is_empty(), "non-crossing insert should not produce trades");
    assert_eq!(book.open_orders().len(), 2);
    assert_eq!(book.best_bid(a, b), Some((50, 100)));
    assert_eq!(book.best_ask(a, b), Some((60, 100)));
}

#[test]
fn crossing_buy_consumes_best_ask() {
    let mut book = OrderBook::new();
    let a = asset(1);
    let b = asset(2);
    // Ask at 60; a buy at 70 crosses (buy price >= ask price).
    let ask = make_order("ask".into(), Side::Sell, a, 100, b, 60, [0; 32], [2; 32], 1_000_000);
    let _ = book.insert(ask);
    let buy = make_order("buy".into(), Side::Buy, a, 100, b, 70, [0; 32], [1; 32], 1_000_000);
    let trades = book.insert(buy);
    assert_eq!(trades.len(), 1, "buy should cross the ask");
    assert_eq!(trades[0].maker.id, "ask");
    assert_eq!(trades[0].maker_fill, 100);
    // The crossed ask is taken; the buy may remain open if it did not fully fill.
    let ask_state = book.get("ask").map(|o| o.state);
    assert_eq!(ask_state, Some(OrderState::Taken));
}

#[test]
fn cancel_removes_open_order() {
    let mut book = OrderBook::new();
    let a = asset(1);
    let b = asset(2);
    let o = make_order("o".into(), Side::Buy, a, 100, b, 50, [0; 32], [1; 32], 1_000_000);
    let _ = book.insert(o);
    assert!(book.cancel("o"));
    assert_eq!(book.get("o").map(|o| o.state), Some(OrderState::Cancelled));
    assert!(book.best_bid(a, b).is_none());
    // Cancelling again fails (already terminal).
    assert!(!book.cancel("o"));
}

#[test]
fn sequence_increments_on_mutation() {
    let mut book = OrderBook::new();
    let s0 = book.sequence();
    let o = make_order("o".into(), Side::Buy, asset(1), 100, asset(2), 50, [0; 32], [1; 32], 1_000_000);
    let _ = book.insert(o);
    assert!(book.sequence() > s0);
    let s1 = book.sequence();
    let _ = book.cancel("o");
    assert!(book.sequence() > s1);
}
