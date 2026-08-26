// asset — Elements asset issuance + lookup helpers
//
// Per docs/RESEARCH.md finding #3, Simplicity has no programmatic issuance
// jets (issuance jets are read-only introspection). Assets must be issued via
// Elements RPC / PSET issuance fields *before* contract funding. This module
// wraps the RPC issuance path (`issueasset`) so tests and the operator can
// mint the two settlement assets without dropping to raw `elements-cli`.
//
// In regtest the issuer is the same node that funds the operator wallet, so
// issued assets land in the wallet and are spendable via `Signer::send`-style
// flows once registered.

use simplex::simplicityhl::elements::AssetId;
use std::str::FromStr;

use super::error::{Error, Result};

/// A freshly issued Elements asset: its id and the reissuance (inflation)
/// token id, both 32-byte hashes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IssuedAsset {
    pub asset_id: AssetId,
    pub reissuance_token_id: AssetId,
}

/// Hex-encode a 32-byte asset id for use in `param::` baking or RPC calls.
#[must_use]
pub fn asset_id_hex(id: AssetId) -> String {
    id.to_string()
}

/// Parse a hex asset id string into an [`AssetId`].
///
/// # Errors
/// Returns [`Error::Asset`] on malformed hex.
pub fn parse_asset_id(hex: &str) -> Result<AssetId> {
    AssetId::from_str(hex).map_err(|e| Error::Asset(format!("invalid asset id '{hex}': {e}")))
}
