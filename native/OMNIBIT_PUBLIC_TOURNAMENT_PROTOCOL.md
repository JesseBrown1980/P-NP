# Omnibit tournament — source-authenticated public six-vantage routing fixture

This is a **measured, source-rooted public-fixture experiment** connecting the restored dual binary/ternary ladder from [PR #38](BINARY_TERNARY_BRIDGE_PROTOCOL.md), native matrix fabric, and automatic post-GULP retention policy. It is the first bounded three-arm routing comparison, not a measurement of seat-local Acer/Liris/Relic matrices or trained multi-GNN forward quality.

## Original, immutable source

Five files are Git-blob-pinned at [Asolaria Omega Omnibit Tournament](https://github.com/JesseBrown1980/asolaria-omega-omnibit-tournament/tree/dad4647ad19a1b08ee90990c34aab1255f9a6046):

- `contracts/ARCHITECTURE.hbp`: Acer/Liris/Relic, NORMAL/ANTI, 64→256→1024→4096, and original distinction among flashlight A/B, software codec A/B, and sign directions
- `provenance/receipts/tournament/OMNIBIT-TOURNAMENT-RECORD-2026-10-09.hbp`: recorded `scoreboard=NOT_SCORED`, `winner=NONE` on October 9, 2026
- `provenance/receipts/operator-catalog/OPERATOR-CATALOG.hbp`: 40 reversible operators, 8 RNQ + 12 TRI + 20 PI, with frozen original-reference test requirements
- `reference/omega-operator-catalog-v0/omega_operator_catalog.rs`: original reference Rust implementation; compiled and independently tested on the hosted Rust 1.81 toolchain, not replaced or retitled as P-NP's routing algorithms
- `contracts/ACER-GLYPH-CODEC-V1.hbp`: `GC_SAFE_TRANSIENT_STATE_ONLY`, `RESIDUAL_COMPLETES_DISCARDED_DISTINCTIONS`, exact charged standalone decoding requirement

Source manifest: `policy/OMNIBIT_TOURNAMENT_PINS.hbi`. Before each native attestation, the process confirms the exact original Git commit, all five source Git blobs, full file SHA-256 and expected contracts. The accepted P-NP [binary/ternary bridge](BINARY_TERNARY_BRIDGE_PROTOCOL.md) HBP/HBI roots are also included.

## Experimental inputs and routes

**20 frozen public synthetic balanced-ternary fixtures** (five lengths 3, 5, 6, 7, 8, each with all-negative, all-zero, all-positive, and alternating trits), stored explicitly as HBP in `native/fixtures/OMNIBIT_PUBLIC_TRITS.hbp`. No private seat filesystem, model checkpoint inference, hidden source data or training corpus is accessed.

The harness crosses:

- 3 **seat labels**: Acer / Liris / Relic; these identify comparison slots, with `seat_local_execution=0`
- 2 **view labels**: NORMAL / ANTI. For this public fixture only, the test mapping is forward trit order vs reversed trit order. The source's operator-specified NORMAL/ANTI semantics remain `original_view_binding=HOLD`.
- 3 actual **reversible routing arms** on the same fixture: `BINARY` (two-bit trit tags with invalid `11` rejected), `TERNARY` (exact radix-three ordinal, two-byte word, strict maximum), and `MIXED` (accepted 64/256/1024/4096 floors and composed 256+64 eight-trit route, with original Q-PRISM 8↔10-bit stream roundtrip in its path).

That is **3 × 2 × 3 × 20 = 360** source-scoped completed public fixture trials. Every arm consumes the *same* frozen trit input for the same seat/view label and must reproduce all input trits exactly after decoding. The fixture labels do not certify independent private-seat executions.

## Measurement and accounting

The native harness writes all 360 actual encoded frames, including four-byte frame lengths, to `PNP_OMNIBIT_PAYLOAD.bin`, with offsets, per-frame SHA-256, payload lengths and total framed bytes referenced in HBI/HBP. It counts an explicit, deterministic **logical-step proxy** for each arm. These step counts are defined by the measured software experiment; they are **not elapsed wall time, CPU machine instructions or performance rankings**.

Each record binds the original public input SHA, view, arm, source and bridge SHA, exact-replay verdict, payload offset, length, framed storage bytes, logical steps and the number of temporary PRISM glyphs. Published HBI/HBP, anchor and binary payload bytes are independently compared against fresh replay and checked by SHA-256; original source modification, index, journal, anchor and frame tampering must fail.

**No winner is declared.** The original Omnibit scoreboard remains `NOT_SCORED`; the new public fixture does not create a source-seat result. Total binary/ternary/mixed bytes and logical operations permit descriptive accounting, while full standalone decoder/executable charges and comparable timing are still `HOLD`. These are exact routing experiments, not a Hutter Prize score or P-versus-NP result.

## GC distinction, no per-operation human approval

The source Acer codec contract requires retaining every distinction necessary for exact reconstruction. The new `Retention` decision is automatic under a frozen policy:

| Input retention class | Requirements for reclaiming original working bytes | Expected test |
|---|---|---|
| `SUMMARY_ONLY` | Fully verified GULP summary | `RECLAIM` |
| `EXACT_REPLAY_REQUIRED`, no complete payload | A hash/summary alone does not reconstruct the source | `HOLD` |
| `EXACT_REPLAY_REQUIRED`, complete payload and independent decoder replay | All required source bits reconstruct and verified evidence is committed | `RECLAIM` |

For the second and third rows, the current gate tests the **retention classification**; the actual prior `gulp-discard` operation remains scoped to owned summary-only event scratch. Exact-replay-required live deletion stays behind future atomic commit and crash/restart tests. Policy checks are automatic; they do not require a human in the inner loop.

## Full HBI → HBP → SHA → SH → HASH receipt chain

```text
pnp-native omnibit-attest source-omnibit evidence evidence
pnp-native omnibit-verify source-omnibit evidence evidence
```

The archive contains **369 HBI/HBP records**: 5 source records, 360 public trial records, 3 GC retention policy cases, and 1 evidence summary. Artifacts:

- `PNP_OMNIBIT.hbp`: indexed source/trial/policy/summary records with full SHA, SH witness, linked HASH and declared provenance
- `PNP_OMNIBIT.hbi`: byte offsets, lengths and full row hashes
- `PNP_OMNIBIT.anchor`: HBP/HBI and physical payload SHA-256 roots
- `PNP_OMNIBIT_PAYLOAD.bin`: exact framed encoded bytes required for independent reconstruction

The hosted pipeline independently builds the original Omnibit reference catalog's Rust tests, runs the new integer-only/native Rust 1.81 harness, repeats the attestation, exercises planted corruption, publishes all evidence, and checks it in a second isolated job. No Node, Python, JSON, external crates, floating arithmetic or live human approval is introduced into the native runtime.

## Next gate

Replace synthetic view proxies with original source-defined NORMAL/ANTI adapters where the operator sources actually specify them, and request independently comparable Acer/Liris/Relic receipts when those computations are available. Run the full 40-operator catalog across frozen binary floors and trit routes, then evaluate elapsed work, memory peaks, full charged state and GC-safe exact restart. Trained GNN forward parity and original source-seat tournament outcomes stay separate acceptance gates.
