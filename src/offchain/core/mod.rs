// core — shared coordination core (not contract-specific)
//
// Wraps the smplx-sdk provider+signer client and the Elements asset/PSET
// helpers used by both the matching and amm_aggregator engines. Kept
// contract-agnostic: contract-specific types (e.g. OrderBookArguments) live
// in their engine modules and depend on core/, never the reverse.

pub mod asset;
pub mod client;
pub mod error;

pub use client::ExchangeClient;
pub use error::{Error, Result};
