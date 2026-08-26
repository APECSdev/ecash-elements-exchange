// test_support — helpers for #[simplex::test] integration tests
//
// Provides builders that construct [`Order`] fixtures and an
// [`ExchangeClient`] from a [`TestContext`] so tests can drive the
// matching engine + settler against a live Elements regtest node without
// repeating boilerplate.

use simplex::TestContext;

use crate::offchain::core::client::ExchangeClient;
use crate::offchain::matching::order::{OrderId, OrderState};
use crate::offchain::matching::{Order, OrderBook, Side};

/// Build an [`ExchangeClient`] from a test context using a fresh random
/// signer (owned, so it can be moved into the client).
#[must_use]
pub fn client_from_context(context: &TestContext) -> ExchangeClient {
    ExchangeClient::from_signer(context.random_signer())
}

/// Construct a maker [`Order`] fixture with sensible defaults.
#[must_use]
pub fn make_order(
    id: OrderId,
    side: Side,
    maker_asset: simplex::simplicityhl::elements::AssetId,
    maker_amount: u64,
    taker_asset: simplex::simplicityhl::elements::AssetId,
    taker_amount: u64,
    maker_script_hash: [u8; 32],
    maker_pubkey: [u8; 32],
    expiry: u32,
) -> Order {
    Order {
        id,
        side,
        maker_asset,
        maker_amount,
        taker_asset,
        taker_amount,
        maker_script_hash,
        maker_pubkey,
        expiry,
        utxo: None,
        state: OrderState::Open,
    }
}

/// A fresh empty [`OrderBook`] for unit tests.
#[must_use]
pub fn fresh_book() -> OrderBook {
    OrderBook::new()
}
