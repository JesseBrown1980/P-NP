# Omnibit 40-operator mixed-floor candidate gate

This is the next native gate after [the 360-run public Omnibit tournament](OMNIBIT_PUBLIC_TOURNAMENT_PROTOCOL.md) and [the exact 243↔256 binary/ternary bridge](BINARY_TERNARY_BRIDGE_PROTOCOL.md).

## Original operator provenance

The original [Omega operator catalog Rust source](https://github.com/JesseBrown1980/asolaria-omega-omnibit-tournament/blob/dad4647ad19a1b08ee90990c34aab1255f9a6046/reference/omega-operator-catalog-v0/omega_operator_catalog.rs) is copied **byte-for-byte** into `native/src/omega_operator_catalog.rs` from frozen commit `dad4647ad19a1b08ee90990c34aab1255f9a6046`, Git blob `1535e158b21b1c2b9ece4dce84e0e9779498ec19`. Every hosted acceptance checks `cmp` of the original and native bytes and their exact Git blob. Both the independent original source reference and the native module's original test suite run on Rust 1.81.

The 40 exact computational operators are:

| IDs | Family | Original transformation | Native inverse |
|---|---|---|---|
| 0–7 | 8 RNQ operators | The inherited C₂³ identity/reverse/half-swap/per-half-bit-reverse algebra | Same operator ID (involution) |
| 8–19 | 12 TRI operators | Rotate each up-to-12-symbol chunk left by direction 0–11 | Rotate the chunk right by the same direction |
| 20–39 | 20 PI operators | Gather symbols in each up-to-20-symbol chunk in canonical icosahedron face graph BFS-distance then face-index order | Explicit scatter using the exact same face order |

All four accepted binary symbol widths—6, 8, 10, 12 bits—use the **original `apply_symbols` and `invert_symbols` functions**, not approximate rewrites or independently invented operators.

## Native connection to balanced ternary and the matrix

Twenty frozen public synthetic ternary fixtures, three seat labels and two view labels are reused unchanged. Every fixture is represented at **all four** binary floor widths 64, 256, 1024 and 4096. A 20-lane bank for each floor is generated deterministically from cyclic balanced-ternary samples. Every lane is first validated as a canonical source `FloorWord` with its own radix-three capacity. Original operators may temporarily transform a lane into a reserved data code; the source inverse is applied before attempting to recover any ternary data, ensuring lossless reconstruction.

For the four **eight-trit** fixture patterns, an additional actual mixed route runs the accepted **256+64 composed floor**. Both a 20-lane 8-bit bank and a 20-lane 6-bit bank are transformed by the same original operator ID and independently inverted before the original 8-trit route is reassembled.

This arrangement exercises the nontrivial TRI and PI permutations against complete 20-symbol chunks, while retaining the exact original ternary roots. It avoids the false-positive case where a 20-way operator applied to only one or two floor words behaves as the identity.

## Execution, accounting and deterministic selection

For each of the six typed seat/view **comparison labels** (Acer/Liris/Relic × NORMAL/ANTI), the harness runs all 40 source operators over:

- 20 fixtures × 4 fixed floors × 6 vantage labels × 40 operator IDs = **19,200 fixed-floor candidate trials**
- 4 eight-trit fixtures × 6 vantage labels × 40 IDs = **960 composed 256+64 candidate trials**

Total: **20,160 exact source-operator inversions**, with full encoded selector/payload frames retained. Each record charges a 1-byte operator selector, explicitly typed floor bits, each bank's symbol count, a 4-byte outer frame length, the transformed symbols as little-endian `u16` words, and a deterministic logical-work proxy. A separate selection pass evaluates 40 actual candidates for each of **504** comparable contexts by `MIN_BYTES_THEN_LOGICAL_WORK_THEN_ID`; all 504 select operator 0 on these fixed-length public frames. This reflects equal-length permutation outputs and the lowest defined work proxy for identity, **not proof of a globally optimal operator or compression gain**.

Family candidate counts are **4,032 RNQ**, **6,048 TRI**, and **10,080 PI**. Strict tests reject invalid IDs, invalid symbol ranges, altered selectors, incorrect floor labels, missing frame bytes, and mutated transformed symbols. The PI/icosahedron and TRI source reference tests ensure the nontrivial operators change the input representation while preserving exact inverse recovery.

The source tournament still records `scoreboard=NOT_SCORED` and `winner=NONE`. The six labels are public proxy comparisons with `seat_local_execution=0`, not observations of private Acer/Liris/Relic matrices. NORMAL/ANTI behavior here is forward/reversed synthetic input; source-defined operator-level NORMAL/ANTI semantics, real timing and peak memory, and fully charged standalone encoder/decoder size are further measurements.

## HBI → HBP → SHA → SH → HASH archive

Before running, the native gate re-verifies the **369-record preceding tournament** and all five original Git-pinned source files, then matches the byte-identical original 40-operator Rust source to the embedded native module.

It emits **20,710 linked HBI/HBP receipts**: 1 source, 40 operator specifications, 4 binary floors, 20,160 candidate trials, 504 selector decisions, and 1 summary. The companion binary archive persists every actual transformed lane and its explicit selector. HBI includes byte offsets, row lengths, full SHA-256 hashes, 16-hex-character SH prefixes and chained HASH values; the root anchor commits to both text indexes and the complete binary payload.

```text
pnp-native omni40-attest source-omnibit evidence evidence
pnp-native omni40-verify source-omnibit evidence evidence
```

Archives: `PNP_OMNI40.hbp`, `PNP_OMNI40.hbi`, `PNP_OMNI40.anchor`, `PNP_OMNI40_PAYLOAD.bin`.

The hosted native acceptance repeats deterministic generation, executes both the original and imported source tests, checks all 20,710 records, rejects original-source and HBI/HBP/anchor/payload corruption, and runs independent artifact integrity verification in a separate job. The runtime remains Rust 1.81, integer-only, no external crates, no Python/Node service, no per-event human approval.

## Next acceptance: selection that changes a real objective

All source operator choices and inverses are now executable on the native binary/ternary matrix. The next credible comparison should score actual transformed *data* under an explicitly charged, source-consistent coder, over larger independent fixture corpora, with elapsed runtime and peak memory. The current per-frame fixed-length symbol permutation cannot reduce its own raw byte count. A full compression claim requires a source-defined coding objective, metadata/selector costs, complete decoder state and independent replay.

Live exact-GC reclamation, full GNN forward parity, and real three-seat NORMAL/ANTI tournament receipts remain separate research gates.
