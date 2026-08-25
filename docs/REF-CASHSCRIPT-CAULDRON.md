# Reference: CashScript & Cauldron (fetched, verified)

Both fetched via `curl` on 2026-08-25 and saved under `docs/ref/`.
Source files:
- `docs/ref/cashscript-about.html` — CashScript overview
- `docs/ref/cs-guides-covenants.html` — CashScript Covenants & Introspection guide
- `docs/ref/cs-language-contracts.html` — CashScript contract structure / functions / params
- `docs/ref/cs-basics-about-bch.html` — About Bitcoin Cash
- `docs/ref/cauldron.pdf` + `docs/ref/cauldron.txt` — Cauldron CPMM paper + extracted text

## CashScript model (verified from fetched docs)

- **Syntax**: Solidity-like (`contract Foo { ... }`, `function`, `require(...)`).
- **No mutable state**: "Once a contract is instantiated with certain parameters,
  these values cannot change." Constructor args are baked into the bytecode
  ("conceptually similar to hard-coded values") — directly analogous to
  Simplicity's `param::`.
- **Function args are untrusted, supplied at spend time** — analogous to
  Simplicity's `witness::`.
- **No user-defined reusable functions** in current CashScript (as of the doc);
  loops-not-in-functions restriction noted. No loops at the contract level in
  practice for covenants.
- **Introspection surface** (the covenant primitives — quoted from the fetched
  covenants guide):
  - `this.activeInputIndex` — index of the input under evaluation
    ↔ Simplicity `jet::current_index()`
  - `tx.inputs.length` / `tx.outputs.length` — input/output counts
    ↔ Simplicity `jet::num_inputs()` / `jet::num_outputs()`
  - `tx.inputs[i].value`, `tx.inputs[i].lockingBytecode`,
    `tx.inputs[i].unlockingBytecode`, `tx.inputs[i].outpointTransactionHash`,
    `tx.inputs[i].outpointIndex`, `tx.inputs[i].sequenceNumber`,
    `tx.inputs[i].tokenCategory`, `tx.inputs[i].tokenCapability`,
    `tx.inputs[i].nftCommitment`, `tx.inputs[i].tokenAmount`
    ↔ Simplicity `jet::input_amount(i)`, `jet::input_script_hash(i)`,
    `jet::input_asset(i)`, etc.
  - `tx.outputs[i].value`, `tx.outputs[i].lockingBytecode`,
    `tx.outputs[i].tokenCategory`, `tx.outputs[i].tokenCapability`,
    `tx.outputs[i].nftCommitment`, `tx.outputs[i].tokenAmount`
    ↔ Simplicity `jet::output_amount(i)`, `jet::output_script_hash(i)`,
    `jet::output_asset(i)`, `jet::output_is_fee(i)`, `jet::output_null_datum(i,n)`
- **Covenant definition** (quoted): "a covenant is a constraint on how money can
  be spent." Identical concept in both CashScript (BCH) and Simplicity (Liquid).

So: **CashScript and SimplicityHL expose a structurally identical covenant
capability set** — read the input/output being spent, read any output's
value/script/token, enforce equality with baked constants. The only differences
are surface syntax (Solidity-like vs Rust-like) and that Simplicity additionally
handles Liquid's confidentiality (`Option`/`Either` wrappers) and has the
`load`/`store` Taproot state-commitment pattern (`third-time.simf`) for carried
state, which BCH achieves differently (script-bytecode self-replication via
`OP_INPUTINDEX OP_UTXOBYTECODE OP_OUTPUTBYTECODE OP_EQUALVERIFY`).

## Cauldron CPMM (verified from fetched PDF text)

From `docs/ref/cauldron.txt` (extracted text):

- **Model**: CPMM on BCH UTXOs. Each micro-pool is one UTXO holding both BCH and
  a fungible token. The UTXO itself *is* the pool state (reserves x, y).
- **Constant product**: `K = x * y`. Trade must keep the output UTXO's
  `output_value * output_token_amount >= K - fee`, with fee ~0.3%.
- **Self-replication (the covenant)**: trade path asserts the contract UTXO is
  recreated at the output with the same index (`OP_INPUTINDEX
  OP_OUTPUTBYTECODE OP_INPUTINDEX OP_UTXOBYTECODE OP_EQUALVERIFY`). This is the
  BCH equivalent of Simplicity's `ensure_input_and_output_script_hash_eq(i)`
  (see `options.simf`).
- **Two paths** (verbatim from the appendix):
  1. Withdrawal: if an input pubkey+sig is present matching `withdraw_pkh`,
     owner withdraws everything (bypasses CPMM). → analogous to a Simplicity
     `withdraw_path` with `bip_0340_verify`.
  2. Trade: verify token category matches, verify contract recreated, compute
     `K = utxo_value * utxo_token_amount`, compute fee = `3 * abs(out-in) /
     1000`, compute effective output K = `(output_value - fee) *
     output_token_amount`, assert `effective_K >= target_K`. → maps directly to
     Simplicity arithmetic jets: `jet::multiply_64`, `jet::subtract_64`,
     `jet::abs` (negate if carry), `jet::divide_64`, `jet::le_64`.
- **Micro-pools / aggregation**: multiple pool UTXOs can be inputs to one trade;
  each is recreated in its corresponding output. No global state. This is the
  key scalability insight: **state is per-UTXO, not global**, so it sidesteps the
  no-loops constraint entirely — each pool is an independent stateless covenant
  instance, exactly like our order-book "each order is a standalone UTXO".

## Cauldron BCH Script appendix (verbatim, for translation reference)

```
OP_DEPTH,
OP_IF,
  # Withdrawal: pubkey + sig matching withdraw_pkh
  OP_DUP, OP_HASH160, withdraw_pkh, OP_EQUALVERIFY, OP_CHECKSIG,
OP_ELSE,
  # Trade:
  OP_INPUTINDEX, OP_OUTPUTTOKENCATEGORY, OP_INPUTINDEX, OP_UTXOTOKENCATEGORY,  # category match
  OP_TXVERSION, 2, OP_EQUALVERIFY,                                              # version 2
  OP_INPUTINDEX, OP_OUTPUTBYTECODE, OP_INPUTINDEX, OP_UTXOBYTECODE, OP_EQUALVERIFY,  # self-replicate
  OP_INPUTINDEX, OP_UTXOVALUE, OP_INPUTINDEX, OP_UTXOTOKENAMOUNT, OP_MUL,        # target K = x*y
  OP_INPUTINDEX, OP_UTXOVALUE, OP_INPUTINDEX, OP_OUTPUTVALUE, OP_SUB, OP_ABS, 3, OP_MUL, 1000, OP_DIV,  # fee
  OP_INPUTINDEX, OP_OUTPUTVALUE, OP_SWAP, OP_SUB,                                # effective bch
  OP_INPUTINDEX, OP_OUTPUTTOKENAMOUNT, OP_MUL,                                  # effective K
  OP_SWAP, OP_GREATERTHANOREQUAL,                                                # effective_K >= target_K
OP_ENDIF
```
