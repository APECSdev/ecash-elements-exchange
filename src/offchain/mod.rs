// offchain — off-chain coordination layer
//
// Two engines, one shared core:
//   matching/       — order book: in-memory price-time book + atomic-swap settler
//   amm_aggregator/ — AMM pool router (v2; stubbed here, see docs/HANDOFF.md)
// Both sit on top of core/, which wraps the smplx-sdk provider+signer client
// and the Elements PSET/asset helpers.
//
// The REST/WebSocket front (server/) exposes the matching engine over HTTP/WS.

pub mod core;
pub mod matching;
pub mod server;
pub mod test_support;
