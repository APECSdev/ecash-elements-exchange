// client — thin wrapper over the smplx-sdk provider+signer pair
//
// The matching and amm engines need a single handle to (a) fetch UTXOs by
// script-hash, (b) broadcast signed transactions, and (c) sign Simplicity
// program inputs. This struct bundles an `Arc<Signer>` (which in turn owns
// a `ProviderTrait` impl) behind a stable API so engine code doesn't reach
// into SDK internals. `Arc` lets the server share one client across async
// tasks.

use std::sync::Arc;

use simplex::provider::ProviderTrait;
use simplex::signer::Signer;
use simplex::simplicityhl::elements::{AssetId, Script};
use simplex::transaction::{FinalTransaction, TxReceipt, UTXO};

use super::error::Result;

/// The exchange operator's handle to the Liquid/Elements backend.
///
/// Owns an `Arc<Signer>` (which in turn owns a `ProviderTrait` impl). All
/// engine modules receive a shared `&ExchangeClient` (or clone the `Arc`
/// across the server's async tasks).
pub struct ExchangeClient {
    signer: Arc<Signer>,
}

impl ExchangeClient {
    /// Construct from a pre-built provider + mnemonic. The provider is built
    /// by the caller (production: `SimplexProvider`; tests:
    /// `TestContext::get_default_provider`).
    ///
    /// # Panics
    /// Panics if the mnemonic fails to parse (mirrors `Signer::new`).
    #[must_use]
    pub fn new(mnemonic: &str, provider: Box<dyn ProviderTrait>) -> Self {
        Self {
            signer: Arc::new(Signer::new(mnemonic, provider)),
        }
    }

    /// Construct directly from an owned [`Signer`] — the path used by
    /// `#[simplex::test]` tests that can move the signer out of the context.
    #[must_use]
    pub fn from_signer(signer: Signer) -> Self {
        Self {
            signer: Arc::new(signer),
        }
    }

    /// Wrap a shared [`Signer`] — for tests where `TestContext::get_default_signer`
    /// returns a borrowed signer; the caller clones into an `Arc` first.
    #[must_use]
    pub fn from_shared_signer(signer: Arc<Signer>) -> Self {
        Self { signer }
    }

    /// Borrow the underlying [`Signer`] (for program signing + send/broadcast).
    #[must_use]
    pub fn signer(&self) -> &Signer {
        &self.signer
    }

    /// Borrow the underlying [`ProviderTrait`] (for UTXO/block fetches).
    #[must_use]
    pub fn provider(&self) -> &dyn ProviderTrait {
        self.signer.get_provider()
    }

    /// Policy asset id for this network (L-BTC on Liquid).
    #[must_use]
    pub fn policy_asset(&self) -> AssetId {
        self.provider().get_network().policy_asset()
    }

    /// Fetch all UTXOs spending to the given contract script.
    ///
    /// # Errors
    /// Propagates provider errors as [`Error::Provider`](super::error::Error::Provider).
    pub fn fetch_script_utxos(&self, script: &Script) -> Result<Vec<UTXO>> {
        Ok(self.provider().fetch_scripthash_utxos(script)?)
    }

    /// Fund a contract script with `amount` of the policy asset and broadcast.
    ///
    /// # Errors
    /// Propagates signer errors as [`Error::Signer`](super::error::Error::Signer).
    pub fn fund(&self, script: Script, amount: u64) -> Result<TxReceipt<'_>> {
        Ok(self.signer.send(script, amount)?)
    }

    /// Finalize + broadcast a fully-assembled [`FinalTransaction`].
    ///
    /// # Errors
    /// Propagates signer errors as [`Error::Signer`](super::error::Error::Signer).
    pub fn broadcast(&self, tx: &FinalTransaction) -> Result<TxReceipt<'_>> {
        Ok(self.signer.broadcast(tx)?)
    }
}
