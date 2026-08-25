# Project Handoff: ecash-elements-exchange — Simplicity Exchange on Elements/Liquid

## Objective

Build on-chain exchange contracts for asset trading on the Liquid (Elements) sidechain using Blockstream's Simplicity smart contract language. The system supports trustless, peer-to-peer atomic settlement of trades between Liquid assets with formal verification of contract behavior. The repo comprises two contract designs — an off-chain-matched order book with on-chain atomic settlement, and an on-chain constant-product AMM (CPMM micro-pool) — sharing common covenant helpers (`simf/lib.simf`).

## Background

Simplicity is a typed, combinator-based smart contract language for Bitcoin-like blockchains, developed by Blockstream Research. It is deployed on Liquid, which is a Bitcoin sidechain that supports multiple issued assets and confidential transactions. Simplicity contracts are written in SimplicityHL (a high-level language with Rust-like syntax) and compiled down to Simplicity bytecode that full nodes execute.

Key properties of Simplicity that make it suitable for this project:

- No loops or unbounded recursion; statically bounded execution cost
- Formally specified semantics suitable for machine-checked proofs
- Multi-asset support via Liquid (each asset has a unique asset ID)
- Confidential transactions (amounts and asset types hidden by default)
- Instant finality (~1 minute Liquid block interval)

## Tech Stack and Toolchain

### Core Tools (Verified Installed)

1. **`simc`** — SimplicityHL compiler. Takes `.simf` source files and produces compiled Simplicity programs. Optionally accepts `.wit` witness files for spend-time data. Supports `--json` output.
   - Repo: https://github.com/BlockstreamResearch/SimplicityHL
   - Binary: ~/.cargo/bin/simc
   - Usage: simc <program.simf> [-w <witness.wit>] [--json]
   - MSRV: Rust 1.79.0+
   - Verified installed: Yes (binary name confirmed via --help)

2. **`hal-simplicity`** — HAL CLI for inspecting compiled programs, building transactions, signing, finalizing, and broadcasting.
   - Repo: https://github.com/BlockstreamResearch/hal-simplicity
   - Binary: ~/.cargo/bin/hal-simplicity
   - Install: cargo install --locked hal-simplicity
   - Verified installed: Yes (v0.2.0)
   - Subcommands:
     - hal-simplicity simplicity info <base64-program> — inspect a compiled program
     - hal-simplicity simplicity sighash <tx-hex> <input-index> <cmr> <control-block> -i <input-utxo> [-s <secret-key>] — compute sighash for signing
     - hal-simplicity address create <program> — create Simplicity addresses
     - hal-simplicity address inspect — inspect/derive unconfidential addresses
     - hal-simplicity tx create <tx-info-json> — create raw Simplicity transaction from JSON
     - hal-simplicity tx decode <tx-hex> — decode raw transaction to JSON
     - hal-simplicity block create <block-info-json> — create raw block from JSON
     - hal-simplicity block decode <block-hex> — decode block

3. **`lwk_cli`** — Liquid Wallet Kit CLI for wallet management, balance queries, UTXO enumeration, and transaction utilities.
   - Repo: https://github.com/Blockstream/lwk
   - Binary: ~/.cargo/bin/lwk_cli
   - Install: cargo install lwk_cli --locked (uses system Rust 1.97.1, not the repo's 1.85.0 pin)
   - Verified installed: Yes (v0.19.0)

4. **`simplex`** (smplx) — Development framework for project scaffolding, building, and local regtest integration testing.
   - Repo: https://github.com/BlockstreamResearch/smplx
   - Binary: ~/.simplex/bin/simplex
   - Install: curl -L https://smplx.simplicity-lang.org | bash, then simplexup
   - Verified installed: Yes (v0.0.9)
   - Subcommands:
     - simplex init <name> — initialize project (creates Simplex.toml, simf/, src/, tests/, Cargo.toml)
     - simplex build — compile .simf contracts to artifacts
     - simplex regtest — spin up local Electrs + Elements nodes
     - simplex test — run unit and integration tests
     - simplex clean — clean generated artifacts
   - Also installs elementsd and electrs to ~/.simplex/bin/

5. **`simplicityhl-lsp`** — Language server for IDE integration (diagnostics, completions, hover docs, go-to-definition).
   - Repo: https://github.com/BlockstreamResearch/simplicityhl-lsp
   - Binary: ~/.cargo/bin/simplicityhl-lsp
   - Install: git clone https://github.com/BlockstreamResearch/simplicityhl-lsp && cd simplicityhl-lsp && cargo install --path .
   - Verified installed: Yes (v0.1.3)

6. **`elementsd`** — Elements daemon for local Liquid node.
   - Repo: https://github.com/ElementsProject/elements
   - Binary: ~/.simplex/bin/elementsd (installed by simplexup)
   - Verified installed: Yes (v23.3.1)

7. **`elements-cli`** — Elements CLI client.
   - Repo: https://github.com/ElementsProject/elements
   - Binary: /usr/local/bin/elements-cli (from CMake build)
   - Build from source: git clone https://github.com/ElementsProject/elements.git && cd elements && cmake -B build -DCMAKE_BUILD_TYPE=Release && cmake --build build -j$(nproc) — binaries at build/bin/
   - Verified installed: Yes (v28.99.0)

### Prerequisites

- Rust toolchain (1.79.0+): https://rust-lang.org/tools/install
  - Verified: rustc 1.97.1
- Elements daemon and CLI (see above)
- Nix (recommended for building Simplicity core from source; not required for this project)

### Repositories

| Repository | URL | Purpose |
|---|---|---|
| Simplicity (core) | https://github.com/BlockstreamResearch/simplicity | C + Haskell + Coq implementation |
| SimplicityHL | https://github.com/BlockstreamResearch/SimplicityHL | High-level language |
| smplx | https://github.com/BlockstreamResearch/smplx | Dev framework |
| simplicity-contracts | https://github.com/BlockstreamResearch/simplicity-contracts | Example contracts |
| simplicity-demo | https://github.com/BlockstreamResearch/simplicity-demo | Rust quickstart |
| simplicity-codespace | https://github.com/Blockstream/simplicity-codespace | Browser workshop |
| simplicityhl-std | https://github.com/BlockstreamResearch/simplicityhl-std | Standard library |
| simplicityhl-lsp | https://github.com/BlockstreamResearch/simplicityhl-lsp | Language server |
| hal-simplicity | https://github.com/BlockstreamResearch/hal-simplicity | HAL CLI |
| lwk | https://github.com/Blockstream/lwk | Liquid Wallet Kit |
| ELIPs | https://github.com/ElementsProject/ELIPs | Improvement proposals |
| elements | https://github.com/ElementsProject/elements | Elements daemon |

### Documentation

- Simplicity docs: https://docs.simplicity-lang.org/
- Quickstart: https://docs.simplicity-lang.org/getting-started/quickstart/
- Execution model: https://docs.simplicity-lang.org/documentation/execution-model/
- Roadmap: https://docs.simplicity-lang.org/resources/roadmap/
- Road to Ecosystem: https://docs.simplicity-lang.org/documentation/road-to-ecosystem/
- Whitepaper (PDF): https://github.com/BlockstreamResearch/simplicity/blob/master/Simplicity-TR.pdf
- Liquid Testnet faucet: https://liquidtestnet.com/faucet
- Liquid Testnet explorer: https://blockstream.info/liquidtestnet/
- Simplicity community (Telegram): https://t.me/simplicity_community

## Architecture Decision: Off-Chain Order Book + On-Chain Settlement

The recommended architecture separates the order matching (off-chain) from the trade settlement (on-chain via Simplicity contracts). This provides the UX of a centralized exchange with the trustless settlement of a DEX.

    [Off-chain order book]     [Matching engine]     [Settlement layer]
         REST/WebSocket  -->  price-time priority  -->  Simplicity contract
         (create/cancel)     (match orders)           (atomic settlement on Liquid)

### Why This Approach

1. **On-chain full order matching is impractical on Liquid**: Simplicity has no loops or unbounded recursion. Iterating an order book would require encoding each order as an explicit branch, which scales poorly.
2. **Atomic settlement is the valuable part**: Each trade settles atomically — either both sides deliver or the transaction is invalid. This eliminates counterparty risk during settlement.
3. **Off-chain matching gives flexibility**: Price-time priority, batch matching, and complex order types (limit, market, stop-loss) are easy to implement in a traditional matching engine.
4. **The Simplicity contract enforces settlement terms**: Even if the matching engine is compromised, it cannot forge a valid settlement without the correct signatures.
5. **Off-chain PSET construction**: The matching engine constructs the full settlement transaction and both parties sign it via PSET.

## Contract Design

### Parameters (compile-time)

| Parameter | Type | Description |
|---|---|---|
| MAKER_PUBKEY | Pubkey | Maker public key (for cancel/refund) |
| MAKER_ASSET | u256 | Asset ID being sold by the maker |
| MAKER_AMOUNT | u6 | Amount of maker asset offered |
| TAKER_ASSET | u256 | Asset ID the maker wants receive |
| TAKER_AMOUNT | u6 | Amount of taker required |
| EXPIRY | Height | Block height after which maker can reclaim |
| TAKER_PUBKEY | Pubkey | Taker public key authorizing settlement |

### Witness values (spend-time)

| Witness | Type | Description |
|---|---|---|
| TAKE_OR_CANCEL | Either< | Branch selector: take path or cancel |
| TAKER_SIG | Signature ( | Taker signature authorizing |
| MAKER_SIG | Signature (cancel | Maker signature authorizing |

### Spend Paths

1. **Take (settlement)**: The taker constructs a transaction that:
   - Spends the order UTXO (consuming MAKER_AMOUNT of MAKER_ASSET)
   - Sends TAKER_AMOUNT of TAKER_ASSET to the maker address
   - Sends MAKER_AMOUNT of MAKER_ASSET to taker address

2. **Cancel (refund)**: The maker reclaims after expiry:
   - Verifies block height >= EXPIRY
   - Verifies maker signature

### Skeleton Contract (SimplicityHL)

    fn checksig(pk: Pubkey, sig: Signature) {
        let msg: u256 = jet::sig_all_hash();
        jet::bip_0340_verify((pk, msg), sig);
    }

    fn take_order(taker_sig: Signature) {
        // Verify taker signature
        let taker_pk: Pubkey = param::TAKER_PUBKEY;
        checksig(taker_pk, taker_sig);

        // Verify transaction outputs via covenant jets:
        // - Output 0: TAKER_AMOUNT of TAKER_ASSET -> MAKER_PUBKEY
        // - Output 1: MAKER_AMOUNT of MAKER_ASSET -> taker-specified destination
        //
        // TODO: Implement output introspection using available covenant jets.
        // This requires investigating which jets are available for:
        //   - Reading output asset IDs
        //   - Reading output amounts
        //   - Reading output script/type
        //   - Verifying output destination
        //
        // The exact jet names need to be confirmed from the SimplicityHL
        // jet catalog and the Elements/Simplicity integration.
    }

    fn cancel_order(maker_sig: Signature) {
        let timeout: Height = param::EXPIRY;
        jet::check_lock_height(timeout);
        let maker_pk: Pubkey = param::MAKER_PUBKEY;
        checksig(maker_pk, maker_sig);
    }

    fn main() {
        match witness::TAKE_OR_CANCEL {
            Left(taker_sig: Signature) => take_order(taker_sig),
            Right(maker_sig: Signature) => cancel_order(maker_sig),
        }
    }

### Critical Implementation Note on Covenants

The take_order function requires **covenant jets** — built-in Simplicity primitives that allow a contract to introspect the transaction it is being spent within (outputs, amounts, asset IDs). The exact set of available covenant jets needs to be confirmed by:

1. Checking the SimplicityHL jet catalog in the SimplicityHL repo (look in src/ for jet definitions)
2. Running simc with available jets and inspecting error messages for the full list
3. Reviewing the simplicity-contracts repo for existing covenant examples
4. Reviewing the "Third Time's the Charm" exercise in the codespace repo (exercise 03) which demonstrates state commitment and covenant patterns
5. Checking the Elements/Simplicity integration for which covenant jets are enabled on Liquid

If output introspection jets are not yet available or insufficient, a fallback design is:
- The taker signs the transaction (proving they authorize it)
- The contract verifies only the taker signature and the maker's expected terms via parameters
- The matching engine (off-chain) constructs the full settlement transaction and both parties sign it via PSET
- This reduces on-chain enforcement but still provides atomic settlement (the transaction either completes or doesn't)

## Development Plan

### Phase 1: Environment Setup and Learning (Complete)

1. Install Rust toolchain (1.79.0+) — DONE (1.97.1)
2. Clone and build SimplicityHL — DONE
3. Clone and run simplicity-demo quickstart — TODO
4. Install smplx — DONE (v0.0.9)
5. Initialize project — DONE
6. Install LSP — DONE (v0.1.3)
7. Study example contracts — TODO

### Phase 2: Core Contract Prototyping

1. Write the single-order contract skeleton as simf/order.simf
2. Compile and validate: simc simf/order.simf --json
3. Determine available covenant jets by:
   - Examining SimplicityHL source for jet definitions
   - Testing compilation with output introspection patterns
   - Reviewing the state commitment exercise (exercise 03 in codespace)
4. If covenants are available: implement full output verification in take_order
5. If covenants are not available: implement the fallback design (signature-only with off-chain PSET construction)
6. Set up local regtest: simplex regtest
7. Write integration tests using smplx-std: cargo add --dev smplx-std
8. Test the full lifecycle on regtest:
   - Compile the contract with maker parameters
   - Fund the contract address with maker asset
   - Construct a take transaction (taker pays maker)
   - Verify settlement/cancel

### Phase 3: Multi-Asset and Edge Cases

1. Issue test assets on regtest: elements-cli issueasset <amount> <asset_name>
2. Test multi-asset settlement
3. Test edge cases:
   - Expiry cancel
   - Invalid take (wrong asset, wrong amount, wrong destination)
4. Test partial fills (if supported by architecture)

### Phase 4: Off-Chain Matching Engine (Weeks 3-4)

1. Build a matching engine in Rust that:
   - Maintains an in-memory order book (bids and asks sorted by price-time priority)
   - Accepts new orders via REST/WebSocket API
   - Matches orders when prices cross
   - Constructs Simplicity settlement transactions for each match
2. Transaction construction flow for each matched trade:
   a. Compile the order contract with the matched parameters (or reuse pre-compiled template)
   b. Get the order UTXO from the order book state
   c. Construct a PSET that:
      - Inputs: the order UTXO
      - Outputs: taker pays maker (correct asset, amount, address), maker asset to taker
      - Fees: appropriate Liquid fee
   d. Compute sighash using hal-simplicity
   e. Taker signs the transaction
   f. Inject witness into the Simplicity program
   g. Recompile with witness: simc order.simf -w order.wit
   h. Finalize PSET: hal-simplicity simplicity pset finalize
   i. Broadcast to Liquid
3. Order lifecycle management:
   - Order creation: compile contract, derive address, fund on Liquid
   - Order cancellation: construct cancel transaction after expiry
   - Order matching: construct take transaction when matched

### Phase 5: Testnet Deployment (Week 5)

1. Deploy on Liquid testnet:
   - Use the faucet at https://liquidtestnet.com/faucet for tLBTC
   - Issue test assets on Liquid testnet
2. End-to-end testing:
   - Create orders via the API
   - Match orders
   - Settle on-chain
   - Verify transactions on the explore

## Project Configuration

The simplex init command auto-generate Simplex. The manual simplex should be:

    [build]
    src_dir = "./simf"
    simf_files = ["*.simf"]
    out_dir = "./src/artifacts"

    [regtest]
    mnemonic = "exist carry drive collect lend cereal occur much tiger just involve mean"
    bitcoins = 10_000_000
    rpc_port = 18443
    esplora_port = 3000
    rpc_user = "user"
    rpc_password = "password"

    [test]
    mnemonic = "exist carry drive collect lend cereal occur much tiger just involve mean"
    bitcoins = 10_000_000

## Key Reference: Demo Script Flow

From the Simplicity Codespace, each demo follows this lifecycle. The matching engine must replicate this programmatically:

    1. Compile         simc <program.simf>
    2. Get address     hal-simplicity address create <base64>
    3. Fund            faucet -> contract address
    4. Build PSET      hal-simplicity tx create <json>
    5. Sign            hal-simplicity sighash <tx-hex> <index> <cmr> <control> -i <utxo> [-s <secret>]
    6. Inject witness  update .wit JSON file
    7. Recompile       simc <program.simf> -w <witness.wit>
    8. Finalize        hal-simplicity pset finalize
    9. Broadcast       curl -> Liquid Testnet API

Witness files are JSON:

    {
        "MY_SIGNATURE": {
            "value": "0x...",
            "type": "Signature"
        }
    }

All contracts use a BIP-341 NUMS internal key (provably disabled key-path spend):

    50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0

Demo scripts use secp256k1 generator multiples (1, 2, 3) as private keys. These are well-known test vectors — never use with real funds.

## Open Questions to Resolve During Phase 1-2

1. What covenant jets are available in the current SimplicityHL release for output introspection (asset ID, amount, script, destination of transaction outputs)?
2. Can a contract verify that a specific output sends a specific amount of a specific asset to a specific pubkey?
3. Is there support for issuing assets programmatically via Simplicity, or must asset issuance be done via Elements RPC before contract deployment?
4. How does the taker destination address get specified? Is it baked into the contract at compile time, or can the taker specify it at spend time?
5. What is the exact set of jets enabled on Liquid vs Bitcoin testnet?

## Starting Point for the Engineering Agent

1. Start with Phase 1 above. Get the environment working and complete the quickstart.
2. Then read every file in the simplicity-contracts repo and the codespace exercises.
3. Then write and test the skeleton contract from the "Skeleton Contract" section above.
4. Report back on the open questions, especially question 1 (covenant jets) and question 4 (taker destination address).
5. Then proceed to Phase 2 and implement the full contract on regtest.
