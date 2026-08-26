// order — order domain types
//
// An order is the off-chain record of an intent to trade. Its on-chain
// representation is a UTXO carrying an `order_book.simf` instance baked
// with the order's `param::` set; this struct is the off-chain mirror.
//
// Per docs/RESEARCH.md finding #4 (Pattern A, open order book): no
// `TAKER_PUBKEY` is baked — anyone may fill. The maker's receiving
// script-hash (`maker_script_hash`) IS baked so the take path can
// covenant-enforce the maker's payout.

use std::fmt;

use simplex::simplicityhl::elements::AssetId;

use crate::offchain::core::asset::asset_id_hex;

/// Opaque order id (uuid-style hex string).
pub type OrderId = String;

/// Which side of the book the order sits on.
///
/// `Buy` = the maker wants to *buy* `maker_asset` paying `taker_asset`;
/// `Sell` = the maker wants to *sell* `maker_asset` receiving `taker_asset`.
/// In both cases the order UTXO funds `maker_asset`; the side only affects
/// how the off-chain book sorts and matches against the order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "buy"),
            Side::Sell => write!(f, "sell"),
        }
    }
}

/// Lifecycle of an order. `Open` orders are matchable; `Taken`/`Cancelled`
/// are terminal and dropped from the book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OrderState {
    Open,
    Taken,
    Cancelled,
}

/// A single order in the book.
///
/// `maker_asset`/`maker_amount` is the asset/amount locked in the order
/// UTXO (the input the take path spends). `taker_asset`/`taker_amount` is
/// the asset/amount the taker must deliver to the maker's baked script-hash.
///
/// `maker_script_hash` is the 32-byte hash baked into `param::MAKER_SCRIPT_HASH`
/// — the output the take path covenant-enforces as the maker's payout
/// destination. `maker_pubkey` (x-only) is baked into `param::MAKER_PUBKEY`
/// for the cancel path signature.
#[derive(Debug, Clone)]
pub struct Order {
    pub id: OrderId,
    pub side: Side,
    pub maker_asset: AssetId,
    pub maker_amount: u64,
    pub taker_asset: AssetId,
    pub taker_amount: u64,
    /// 32-byte script-hash baked into `param::MAKER_SCRIPT_HASH`.
    pub maker_script_hash: [u8; 32],
    /// 32-byte x-only pubkey baked into `param::MAKER_PUBKEY`.
    pub maker_pubkey: [u8; 32],
    /// CLTV expiry (Elements block height) baked into `param::EXPIRY`.
    pub expiry: u32,
    /// Txid+vout of the funded order UTXO carrying the compiled contract.
    /// `None` until the order is funded on-chain (callers set this after
    /// broadcasting the funding tx).
    pub utxo: Option<(String, u32)>,
    pub state: OrderState,
}

impl Order {
    /// Effective price as `taker_amount / maker_amount` (rational, not f64).
    /// Used only for sorting/matching; settlement uses integer amounts.
    #[must_use]
    pub fn price(&self) -> (u64, u64) {
        (self.taker_amount, self.maker_amount)
    }

    /// Hex of the maker asset (for `param::MAKER_ASSET_ID` baking display).
    #[must_use]
    pub fn maker_asset_hex(&self) -> String {
        asset_id_hex(self.maker_asset)
    }

    /// Hex of the taker asset (for `param::TAKER_ASSET_ID` baking display).
    #[must_use]
    pub fn taker_asset_hex(&self) -> String {
        asset_id_hex(self.taker_asset)
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Order({}, {} {} @ {}/{})",
            self.id,
            self.side,
            self.maker_amount,
            self.taker_amount,
            self.maker_amount
        )
    }
}
