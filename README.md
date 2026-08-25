# ecash-elements-exchange

On-chain exchange contracts for the **Liquid (Elements)** sidechain, written in Blockstream's **Simplicity** smart contract language ([SimplicityHL](https://github.com/BlockstreamResearch/SimplicityHL)).

The repo contains **two** on-chain contract designs plus the off-chain coordination layer, built to compare and contrast the two dominant DEX architectures under Simplicity's no-loops covenant model.

## Contracts

### 1. Order Book settlement — `simf/order_book.simf`

Off-chain order matching + on-chain atomic swap settlement. Each order is a standalone UTXO with its own contract instance; maker/taker asset IDs, amounts, and the maker's receiving script-hash are baked as `param::` at compile time; the taker's destination is authorized by a BIP-340 signature over `jet::sig_all_hash()`.

Spend paths:
- **Take** (settlement): taker signature + covenant checks that outputs deliver `param::TAKER_AMOUNT` of `param::TAKER_ASSET` to the maker's baked script-hash, and `param::MAKER_AMOUNT` of `param::MAKER_ASSET` to the taker's chosen destination.
- **Cancel** (refund): `jet::check_lock_height(param::EXPIRY)` + maker signature.

Reference template: [`simplicity-contracts/.../option_offer.simf`](https://github.com/BlockstreamResearch/simplicity-contracts/blob/main/crates/contracts/simf/option_offer.simf).

### 2. AMM / CPMM micro-pool — `simf/amm.simf`

On-chain constant-product market maker (`K = x * y`). Reserves live in the UTXO itself (the input/output amounts), so no separate Taproot state commitment is required — each micro-pool is a stateless covenant instance, sidestepping the no-loops constraint. Ported from the [Cauldron](https://cauldron.quest) CPMM design on Bitcoin Cash; see `docs/REF-CASHSCRIPT-CAULDRON.md`.

Spend paths:
- **Trade**: verify the pool UTXO is recreated at the output with the same index (`input_script_hash(i) == output_script_hash(i)`), verify asset category match, and assert `(out_amount - fee) * out_token >= in_amount * in_token` after a configurable `param::FEE_NUM`/`param::FEE_DEN` fee.
- **Withdraw** (owner drain): `jet::bip_0340_verify((param::OWNER_PUBKEY, jet::sig_all_hash()), sig)` bypasses the CPMM constraint.

Arithmetic jets used: `jet::multiply_64`, `jet::subtract_64`, `jet::divide_64`, `jet::le_64`.

## Architecture

```
[Off-chain coordination]              [On-chain settlement]
  matching/  (order matching)    -->   order_book.simf  (atomic swap)
  amm_aggregator/ (pool batching) -->  amm.simf          (CPMM micro-pool)
```

Both contracts use the **same shared covenant helpers** (`simf/lib.simf`): explicit-output asset/amount extraction, output script-hash equality, BIP-340 signature checks.

## Why Liquid, not Bitcoin

The covenant jets — `jet::output_amount`, `jet::output_asset`, `jet::output_script_hash`, `jet::num_outputs`, `jet::current_index`, `jet::sig_all_hash`, issuance introspection — exist **only** in the Elements (Liquid) jet environment (`SimplicityHL/src/jet/elements.rs`). The Core (Bitcoin) environment exposes none of them in `simc` 0.7.1. On-chain output/asset/amount enforcement is therefore a Liquid-only capability. Full findings: `docs/RESEARCH.md`.

## Toolchain (verified installed)

| Tool | Version | Path |
|---|---|---|
| `simc` (SimplicityHL) | 0.7.1 | `~/.cargo/bin/simc` |
| `hal-simplicity` | 0.2.0 | `~/.cargo/bin/hal-simplicity` |
| `lwk_cli` | 0.19.0 | `~/.cargo/bin/lwk_cli` |
| `simplex` (smplx) | 0.0.9 | `~/.simplex/bin/simplex` |
| `elementsd` | v23.3.1 | `~/.simplex/bin/elementsd` |
| `elements-cli` | v28.99.0 | `/usr/local/bin/elements-cli` |
| `simplicityhl-lsp` | 0.1.3 | `~/.cargo/bin/simplicityhl-lsp` |
| `rustc` | 1.97.1 | — |

## Development lifecycle

```bash
simplex build      # compile .simf -> src/artifacts
simplex regtest    # spin up local Electrs + Elements regtest
simplex test       # run integration tests
simplex clean      # remove artifacts
```

Manual compile/inspect:
```bash
simc simf/order_book.simf --json
hal-simplicity simplicity info <base64-program>
```

## Status

- [x] Toolchain verified, reference repos cloned into `~/Workspace/simplicity-tooling/`
- [x] Covenant jet research complete — `docs/RESEARCH.md`
- [x] CashScript/Cauldron references fetched — `docs/REF-CASHSCRIPT-CAULDRON.md`, `docs/ref/`
- [x] Project scaffolded (`Simplex.toml`, `Cargo.toml`, `simf/`, `src/`, `tests/`)
- [ ] `simf/lib.simf` shared helpers
- [ ] `simf/order_book.simf` — order book settlement contract
- [ ] `simf/amm.simf` — CPMM micro-pool contract
- [ ] Regtest integration tests
- [ ] Off-chain matching engine + AMM aggregator (Rust)

See `docs/HANDOFF.md` for the full handoff, `docs/RESEARCH.md` for the resolved covenant-jet findings, and `AGENTS.md` for engineering rules.

> SimplicityHL is a work in progress. Not ready for production use.
