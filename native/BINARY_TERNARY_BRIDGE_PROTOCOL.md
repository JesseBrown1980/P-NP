# Native Asolaria binary-floor ↔ balanced-ternary bridge

This is the integration gate between the previously accepted **binary representation floors** and **native balanced ternary** computation. It restores the user-established intent of a single Asolaria matrix fabric using two coordinated arithmetic ladders. The exact five-trit-to-byte ordinal rule below is a **newly frozen bridge convention**, not a claim that every historical repository used this particular ordinal ordering.

## Source-authenticated supporting systems

- [Original Q-PRISM Host-8](https://github.com/JesseBrown1980/Q-PRISM-human-organoid-neural-stream-as-a-high-dimensional-control/blob/b4a21a25381d6920b7aab92c94c3c3e1420fc9e4/host8/qprism_cube_host8.rs) (256→1024 8-bit/10-bit stream, exact inverse; pinned and independently replayed by PR #35).
- [Asolaria GAC 60D and neural fabric laws](FABRIC_GAC_GC_PROTOCOL.md) (10 pinned source artifacts and 16 verified receipts).
- Native [ternary.rs](src/ternary.rs) and [27-cell matrix.rs](src/matrix.rs), with existing third-seat frozen witness tests.

Both source evidence bundles are independently reverified by `pnp-native bridge-attest` before the new bridge HBI/HBP is generated. The Q-PRISM physical bitstream and balanced ternary ordinal are separate, typed domains.

## Exact ordinal law

A balanced trit is one of `−1, 0, +1`. The bridge's canonical **data code** for `n` trits in most-significant-first order is:

```text
digit(−1)=0, digit(0)=1, digit(+1)=2
code = Σ digit(trit[i]) × 3^(n−1−i)
```

This is a bijection between `3^n` valid trit strings and **the data subset** `0..3^n−1` of a binary floor. It is an injection into the whole binary floor because the floor has additional bit patterns. The decoder rejects all remaining patterns as data. It does not silently interpret the reserved codes as controls, witnesses, or operator instructions; those meanings require a separately tagged, validated control record.

Examples at the 256-value byte floor: `----- → 0`, `00000 → 121`, `+++++ → 242`, `+−0+0 → 178`, and codes **243..255 → RESERVED / REJECT_AS_DATA**. The matrix center coordinate `(1,1,1)` has 27-cell index `13`; it is distinct from the meaning of SAT's unassigned `Trit::Zero`. The encoding is a representation rule, not a promotion or runtime execution policy.

## Binary floors and accepted ternary payload capacity

| Binary floor | Binary width | Valid trits per floor | Ternary data states | Reserved binary codes |
|---|---:|---:|---:|---:|
| 64 | 6 bits | 3 | 27 | 37 |
| 256 | 8 bits | 5 | 243 | 13 |
| 1,024 | 10 bits | 6 | 729 | 295 |
| 4,096 | 12 bits | 7 | 2,187 | 1,909 |

Smaller ternary sizes 3, 9 and 81 are handled by the smallest applicable certified floor, with **canonical zero-trit padding** where necessary. Padding is verified on decode, avoiding additional encodings for the same payload.

The next ternary landing, **6,561 = 3⁸**, is encoded exactly as **five trits in a 256 floor + three trits in a 64 floor**, with independently enforced floor identities. That is a valid mixed-floor composition, not a single 4,096-symbol value. The route and original trit length are required metadata for decoding. No assumption of sub-entropy representation or compression is made.

## Host acceptance and independent HBI/HBP receipts

All possible values on each certified floor are exhaustively checked:

- 3,186 valid per-floor ordinal-to-trit inverses across 64, 256, 1,024 and 4,096.
- 9,840 ternary sequence routes across sizes 3, 9, 27, 81, 243, 729, 2,187 and 6,561.
- Every reserved code on all four binary floors, including the **13 byte codes** 243–255, rejected as ternary data.
- All 27 three-coordinate matrix positions, including the exact center address 13, checked against existing native `matrix::address`/`coordinates`.
- A 243-byte corpus consisting of all valid 5-trit data codes is passed through the original source-verified PRISM 256↔1024 representation translator and back, with no byte differences.

`bridge-attest` / `bridge-verify` bind the existing Git-pinned GAC and PRISM HBP roots to **26 new indexed records**: four binary floors, eight ternary landings, 13 explicitly rejected reserved byte codes and one 27-cell matrix witness. Each record contains an upstream root, ordinal/scope metadata, a full SHA-256 row digest, an SH prefix, HBI offset and length, and a linked HASH successor. The final anchor commits to both HBI/HBP roots. The archive is independently regenerated and compared byte-for-byte, including deliberate HBI/HBP/anchor corruption tests.

```text
pnp-native bridge-attest source-prism source-gac source-neural source-omni evidence evidence
pnp-native bridge-verify source-prism source-gac source-neural source-omni evidence evidence
```

The accepted runtime remains Rust 1.81, no third-party crates, integer/trit arithmetic only, and no Python, Node, JSON or floating-point execution in the hot path. All ordinary state routing is automatic under pinned policy; there is no human approval on each routing step.

## Research boundary and follow-on performance test

The first gate proves exact reversible **address and representation semantics** and source-linked verification. A separate Omnibit tournament gate must measure actual work and memory: binary-only, ternary-only and mixed routing under identical frozen inputs, with full selector, witness, stored-state, and decoder charges. The model's four trained neural forward functions, 60D original-axis glyph decoding, and further GC replay classes remain independently tracked as HOLD until their own acceptance jobs are complete.
