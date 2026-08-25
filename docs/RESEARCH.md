# Research Findings: Simplicity Covenant Jets & Open Questions

Resolved from source by inspecting the installed toolchain and reference repos
cloned into `~/Workspace/simplicity-tooling/`. All citations below are direct
file paths.

## Source of truth for the jet catalog

- `~/Workspace/simplicity-tooling/SimplicityHL/src/jet/elements.rs` — the
  **Elements (Liquid)** jet environment, the authoritative list of jets
  available when compiling for Liquid/Elements with `simc`.
- `~/Workspace/simplicity-tooling/SimplicityHL/src/jet/core.rs` — the
  **Core (Bitcoin)** jet environment.
- `~/Workspace/simplicity-tooling/simplicity-contracts/crates/contracts/simf/option_offer.simf`
  — a working reference **covenant swap** contract (closest analogue to an
  order-book "take" path).
- `~/Workspace/simplicity-tooling/simplicity-contracts/crates/contracts/simf/options.simf`
  — working reference for multi-asset covenant settlement + issuance
  introspection.
- `~/Workspace/simplicity-tooling/simplicity-codespace/exercises/03-third-time/third-time.simf`
  — canonical covenant / state-commitment pattern.

## Q1 — Covenant jets for output introspection: RESOLVED (YES, available)

The Elements jet environment exposes, with `.simf` call syntax confirmed in
real contracts:

| Jet (snake_case in .simf) | Returns | Source citation |
|---|---|---|
| `jet::output_amount(idx: u32)` | `Option<(Asset1, Amount1)>` — asset **and** amount of output `idx` | elements.rs target class `OutputAmount`; option_offer.simf `get_output_explicit_asset_amount` |
| `jet::output_asset(idx: u32)` | `Option<Asset1>` — asset ID of output `idx` | elements.rs `OutputAsset`; options.simf `ensure_output_asset_eq` |
| `jet::output_script_hash(idx: u32)` | `Option<u256>` — script-hash (destination) of output `idx` | elements.rs `OutputScriptHash`; option_offer.simf `get_output_script_hash` |
| `jet::output_is_fee(idx: u32)` | `Option<bool>` — is output a fee output | elements.rs `OutputIsFee`; third-time.simf update() |
| `jet::output_null_datum(idx, n)` | OP_RETURN/null-datum of output | options.simf `ensure_output_is_op_return` |
| `jet::num_outputs()` | `u32` | elements.rs `NumOutputs`; third-time.simf |
| `jet::current_index()` | `u32` (index of the input being spent) | elements.rs `CurrentIndex`; options.simf |
| `jet::input_amount(idx)` / `jet::input_asset(idx)` / `jet::input_script_hash(idx)` | same shapes as output counterparts | elements.rs; options.simf |

`Asset1` and `Amount1` are Elements "confidential-or-explicit" option types
(`Either<Point, u256>` for asset, `Either<Point, u64>` for amount). For
explicit (unblinded) outputs the value is the `Right` variant, extracted via
`unwrap_right::<(u1, u256)>(asset)` / `unwrap_right::<(u1, u256)>(amount)` —
exactly as `option_offer.simf::get_output_explicit_asset_amount` does.

## Q2 — Can a contract verify a specific output sends a specific amount of a
specific asset to a specific pubkey? RESOLVED (YES, partially)

- **Asset + amount of a given output**: YES, directly. Pattern
  `ensure_output_asset_with_amount_eq(idx, asset_id, amount)` in
  option_offer.simf compares `jet::output_amount(idx)`'s asset/amount against
  `param::`-baked expected values via `jet::eq_256` / `jet::eq_64`.
- **Destination to a specific pubkey**: ONLY via **script-hash**, not a raw
  pubkey. `jet::output_script_hash(idx)` returns the SegWit script-hash of the
  output; the contract compares it (`jet::eq_256`) against a `param::`-baked
  expected script-hash. There is no jet that returns a destination *pubkey*
  directly; destinations are enforced as precomputed script-hashes. For a
  maker receiving payment, the maker's receiving script-hash is baked as a
  `param::` at compile time.
- For the taker receiving the maker asset: see Q4 (destination need not be
  baked — signature authorizes the whole tx).

## Q3 — Programmatic asset issuance via Simplicity? RESOLVED (NO — read-only)

The issuance jets in elements.rs are all **introspection/calculation**, not
creation: `Issuance`, `IssuanceAsset`, `IssuanceToken`, `IssuanceEntropy`,
`CalculateIssuanceEntropy`, `CalculateAsset`, `CalculateExplicitToken`,
`CalculateConfidentialToken`, `CurrentIssuanceAssetAmount`,
`CurrentIssuanceTokenAmount`, `IssuanceAssetAmount`, `IssuanceTokenAmount`.

No jet *creates* an issuance. Asset issuance is a transaction-level Elements
feature (the `issueasset` RPC / PSET issuance fields). A Simplicity contract
can only *verify* issuance present on its own input
(`CurrentIssuance*`) or on input `idx` (`Issuance*`).
Conclusion: issue test assets via Elements RPC **before** contract funding.
This matches the HANDOFF fallback note and the options.simf funding path
which verifies pre-issued reissuance tokens.

## Q4 — How is the taker destination address specified? RESOLVED

It is **NOT** baked into the contract. Two valid patterns observed:

1. **Free destination (signature-authorized)** — `option_offer.simf`
   `withdraw_path` and `exercise_path` constrain only
   `ensure_output_asset_with_amount_eq(idx, asset, amount)` for the
   counterparty/taker output and do NOT check its script-hash. The comment in
   option_offer.simf explicitly says "Settlement asset → user (any address)".
   Authorization comes from `jet::bip_0340_verify((param::USER_PUBKEY,
   jet::sig_all_hash()), sig)` — the signer authorizes the *entire*
   transaction (including their own destination) by signing the sighash.
2. **Baked destination (script-hash)** — if stricter enforcement is wanted,
   the taker's destination script-hash can be a `param::` and checked with
   `ensure_output_script_hash_eq(idx, expected)`.

Recommended for our order book: the **maker's** receiving script-hash and the
**maker asset** / **taker asset** / amounts are `param::`-baked; the
**taker's** destination is left free and authorized by the taker's BIP-340
signature over `sig_all_hash()` (pattern 1). This avoids recompiling per
taker.

## Q5 — Jets enabled on Liquid vs Bitcoin testnet: RESOLVED

- The Elements (Liquid) jet environment (`elements.rs`) is a **superset** of
  the Core (Bitcoin) environment (`core.rs`).
- The Core (Bitcoin) environment in this `simc` 0.7.1 build exposes **no**
  transaction-introspection jets among the searched categories
  (`grep -cE "Output|Input|Num|Current|TxHash|SigAllHash"` on `Core::` variants
  in core.rs = 0). Core exposes only signature verification
  (`CheckSigVerify`, `Bip0340Verify`) and `ParseLock`/`ParseSequence` plus
  crypto/arithmetic primitives.
- All covenant tx-introspection jets (`OutputAsset`, `OutputAmount`,
  `OutputScriptHash`, `OutputIsFee`, `NumOutputs`, `CurrentIndex`,
  `InputAmount`, `InputAsset`, `SigAllHash`, issuance, confidentiality,
  `LbtcAsset`, `GenesisBlockHash`) are **Elements-only**.
- Conclusion: on-chain output/asset/amount enforcement is available **only on
  Liquid/Elements**, not on Bitcoin. Our project targets Liquid, so the full
  covenant design is viable. This is why the HANDOFF focuses on
  Elements/Liquid.

## Signature / sighash primitives (confirmed)

- `jet::sig_all_hash()` → `u256` (the BIP-341-ish sighash of the whole tx).
- `jet::bip_0340_verify((pubkey: Pubkey, msg: u256), sig: Signature)` —
  Schnorr/BIP-340 verification. `CheckSigVerify` is **disabled**
  (`is_disabled` returns true for it in elements.rs); use `Bip0340Verify`.
- `jet::check_lock_time(t: Time)` and `jet::check_lock_height(h: Height)` —
  time-lock assertions (used for the cancel-after-expiry path).

## Params & witness (confirmed syntax from option_offer.simf)

- Compile-time params: `param::NAME` (e.g. `param::COLLATERAL_ASSET_ID`,
  `param::USER_PUBKEY`). Each distinct order requires a separate compilation.
- Spend-time witness: `witness::NAME`, typed in `main()` via `match
  witness::PATH { Left(...), Right(...) }`. Witness files are JSON
  (`{"NAME": {"value": "0x...", "type": "Signature"}}`).

## Recommended reference contract to model `order.simf` on

`simplicity-contracts/crates/contracts/simf/option_offer.simf` — it is a
covenant swap (maker deposits collateral+premium, counterparty swaps
settlement asset), with explicit helper functions
(`ensure_output_asset_with_amount_eq`, `ensure_output_script_hash_eq`,
`ensure_correct_change_at_index`, `check_user_signature`). This is the
closest factual template for an order-book "take" path.
