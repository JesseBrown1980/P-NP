# Gate D — multi-GNN source-identical control and model restoration ledger

This tranche restores the original **multi-component ensemble**, with independent acceptance for each original function instead of treating all GNNs as one trained checkpoint.

## Source-confirmed inventory

The original [trained model repository](https://github.com/JesseBrown1980/Asolaria-fnns-trained-and-reverse-gnns-many) and [stage specification](https://github.com/JesseBrown1980/Shannon-and-the-gnns-stage/blob/32f9670a648dbbd02d64524e91150efeec8e4d2c/TRAINED-MODELS.md) describe **four trainable families** (EdgeLevelGNN, PrototypeGNN, ContrastiveGNN, GSLGNN), **nine published .pt checkpoints**, and **eight scoring signals** (L0, L4, G1, G2, G3, G4, OmniShannon, SHA fallback). L1 and L2 are additional trained architectures rather than extra live scoreboard slots. `native/fixtures/MULTI_GNN_FAMILIES.hbi` captures ten separately named families, including L1 and L2, with origin Git blob references and exact model files.

| Family/plane | Source state | Native state |
|---|---|---|
| L0 EdgeLevelGNN | exact origin source blob and baseline checkpoint pin | `HOLD_FORWARD` |
| L1 PrototypeGNN | three published checkpoints | `HOLD_FORWARD` |
| L2 ContrastiveGNN | three published checkpoints | `HOLD_FORWARD` |
| L4 GSLGNN | v1/v2 checkpoints with manifest SHA-256 | `HOLD_FORWARD` |
| G1 edge-mining | published source, authority and JL fingerprint computations | `HOLD_JL_PARITY` |
| G2 forward-genius | published source, Hamming similarity and path scoring | `HOLD_JL_PARITY` |
| G3 reverse-gain | external PII-bearing implementation withheld | `HOLD_SOURCE` |
| G4 GLSM | published deterministic 5-state transition mechanism | native integer Q10,000 **source-rule test** |
| OmniShannon | published stage source | `HOLD_ENTROPY_PARITY` |
| SHA baseline | published stage source | already certified in PR #31 |

The original healthcare GNN lineage is recorded with four exact model-code blobs in [the model origin document](https://github.com/JesseBrown1980/Asolaria-fnns-trained-and-reverse-gnns-many/blob/c4b2458c88358c46155d3b308c594f9991720847/docs/PRE-ASOLARIA-HEALTHCARE-GNN-PROVENANCE-AND-PATH2-2026-07-11.md). Its reported training metrics are historical. The newer GSLGNN corpus manifest contains **315,209 suspicious and 40 benign samples**. Any balanced evaluation must report per-class performance, not rely on class-imbalanced accuracy alone.

## Native G4 GLSM exact-integer transition port

The original G4 source is `src/planes/glsm.mjs` at Git blob `9006f5cca08fab56c906bb67237e17974090cfe8` from the pinned original repo. The implementation uses integer Q10,000 for exact comparisons with `tau_path=tau_gain=0.5`.

- `DESCRIBED` + edge weight strictly greater than zero → `EDGE_MINED`.
- `EDGE_MINED` + path confidence at least 0.5 → `PATH_FOUND`.
- `PATH_FOUND` + negative reverse gain → `MISTAKE_FLAGGED`, which **hard blocks Fischer Tier-0 promotion**.
- `PATH_FOUND` + reverse gain at least 0.5 → `CONVERGED`.
- `MISTAKE_FLAGGED` + path confidence at least 0.5 → `PATH_FOUND`.
- `CONVERGED` is absorbing.

The G4 emitter also requires **all three G1/G2/G3 antecedent row identifiers** and real inputs; missing inputs cause no transition. A no-change transition does not emit a new record. The engine keeps the operator's final authority separately at `operator_held=true`. The source's original five-state walk and eight adversarial/boundary cases are frozen in `native/fixtures/G4_GLSM_REFERENCE.hbi`. Exact integer comparison is appropriate here because finite source tests all use values representable at four decimal places.

## HBI → HBP → SHA → SH → HASH

`pnp-native gnn-gate-attest SOURCE_ARCHIVE OUTPUT` validates the previously certified 23-file source bundle, checks the 10-family inventory against its original checkpoint Git blob identities, validates each G4 reference case, and constructs `PNP_GNN_GATE.hbi`, `PNP_GNN_GATE.hbp`, and `PNP_GNN_GATE.anchor`.

The chain records **23 receipts** (10 family identities and 13 G4 cases), the full SHA-256 of each source record, 16-hex-character SH routing prefix, sequential HBI offsets/lengths, full source-archive SHA root and linked HASH chain. The anchor is an unkeyed checksum bound to separately Git-pinned sources and hosted evidence; it is not a third-party signature. The native verifier independently regenerates the record sequence, checks exact byte equality, and hosted CI checks file digests, rerun determinism and planted HBI/HBP/anchor mutations.

Usage:

```text
native/target/release/pnp-native source-attest source-train source-stage evidence
native/target/release/pnp-native gnn-gate-attest evidence evidence
native/target/release/pnp-native gnn-gate-verify evidence evidence
```

All executable acceptance code runs under Rust 1.81, `--offline --locked`, with Clippy `-D warnings`, no `unsafe`, no float arithmetic, no native Node/JS/JSON/V8, and independent archive rechecks.

## Remaining Gate D work

The next implementable neural milestone is a **source-matched, bounded export of all nine actual checkpoint tensors**, a documented quantization rule, independently reproduced original-model inference outputs for each of the four trainable architectures on balanced and held-out graphs, and exact comparisons against native integer inference. The G1/G2 JL calculations, G3 private-source boundary, and full OmniShannon entropy parity are tracked independently. A deterministic G4 PASS cannot substitute for trained checkpoint forward equivalence.
