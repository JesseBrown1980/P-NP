# Gate B / deterministic scoring seam — ternary waves and HBI-HBP-SHA-SH-HASH

**P-NP native Rust 1.81; no external runtime or uncontrolled model execution.**

## Computed ternary wave

For each native matrix-enabled SAT arm, the router reads the frozen 27-cell **physical source rooms**, with the third seat reconstructing every cell from exact integer `(sum, d0, d1)` witnesses. At each requested 3-way supervisor, it calculates three signed subtree totals, freezes them as balanced center/difference coordinates, reconstructs them exactly, chooses one balanced address trit `−,0,+`, then repeats until it reaches the addressed leaf.

There is one **invariant Relic reference center** with coordinate 0 and value 1. The SAT symbol `0` remains *unassigned*, not the same center. Widths 3/9/27/81 have depths 1/2/3/4, with no need to instantiate the entire tree of possible address values. Every read must round-trip its exact three-way address to the selected original room; a corrupted address, value or child witness is rejected.

An independently certified full sweep evaluates **120 room addresses**, performs **426 path decisions**, and checks **1,278 signed child totals**. These measurements describe *actual data-derived ternary operations*, not an increase in Boolean truth values or an unmeasured 81-fold search speedup.

Matrix-enabled native SAT variable ranking now consumes the room-total observation from the selected wave path, in addition to its existing exact Fischer, MTP and cell guidance. The common DPLL SAT/UNSAT correctness core and independent original-CNF verification remain unchanged; search still checks both nonzero truth assignments.

## Source-matched original score functions

The original SHA baseline and reverse-gain primitives in `JesseBrown1980/Shannon-and-the-gnns-stage/pipeline/asolaria-score.mjs`, exact Git blob `b101c0989de3af0ca41db0eac1293c758d7e5296` at commit `32f9670a648dbbd02d64524e91150efeec8e4d2c`, have now been ported to exact Rust integer Q10,000:

- `SHA256(UTF8(pid + "|" + content))` produces a 64-hex-character digest; the first four hexadecimal characters interpreted as an unsigned integer define `baseline_q10000 = (value mod 1000) × 10`. This is numerically identical to the original baseline's 3-decimal result expressed with four fixed decimal digits.
- `reverse_risk_q10000 = 10000 − baseline_q10000`.
- The original `reverseGain` gate marks `GENIUS` when `score_q10000 ≥ 7200` and `reverse_risk_q10000 ≤ 2800`; otherwise `MISTAKE`.

This port deliberately applies reverse gain to the **baseline score**, not an invented output from the unavailable trained L0/L4 neural models. Both original standalone helper functions are tested using **nine independently calculated SHA-256 fixtures** in `native/fixtures/SOURCE_SCORE_VECTORS.hbi`. The fixtures include empty strings, signed trits, Unicode, whitespace, and newline-containing inputs. Source-supplied `pid|content` serialization can be ambiguous; receipts also bind separate PID and content SHA-256 values to prevent ambiguity in *provenance*.

## Evidence chain — HBI → HBP → SHA → SH → HASH

The source-attestation archive from Gate A is required before scoring. The scorer verifies all 23 original file pins and the original scoring-stage Git blob identity, then produces:

| Name | Purpose |
|---|---|
| `PNP_SCORE.hbi` | ordered input/score indexes, offsets, byte lengths, per-row SHA digests |
| `PNP_SCORE.hbp` | SHA baseline and reverse-gain outputs, separate request/content/PID digests, source HBP root, source blob pin and predecessor |
| `PNP_SCORE.anchor` | hash of both complete output files, original input SHA and source-archive HBP SHA, terminal linked HASH |

`SH` is the 16-character source digest prefix for routing only; the full SHA-256 is the integrity check. Linked `HASH` combines the previous hash and the receipt digest in strict order. Source SHA pins, independently published hosted archives, and a separately checked full input/root are necessary for provenance. The unkeyed anchor is an integrity checksum, not a third-party digital signature.

## Commands

```text
native/target/release/pnp-native certify
native/target/release/pnp-native source-attest source-train source-stage evidence
native/target/release/pnp-native score-attest native/fixtures/SOURCE_SCORE_VECTORS.hbi evidence evidence
native/target/release/pnp-native score-verify native/fixtures/SOURCE_SCORE_VECTORS.hbi evidence evidence
```

Native tests and hosted CI require Rust 1.81 `--offline --locked`, `-D warnings`, integer-only compiler IR, frozen original sources, deterministic HBI/HBP/SHA/SH/HASH generation, independent archive rechecks, and planted HBI, HBP, anchor and golden-vector tamper rejection.

## Pending Gate D

The actual **trained .pt GNN/FNN checkpoint forward functions**, model-specific integer/ternary weight quantization, original model behavior on balanced out-of-sample examples, trained routing control, and Host-8/Omni execution remain `HOLD`. This release certifies source-matched deterministic scoring helpers and the genuine ternary routing computation. It does not claim trained neural-equivalence or a mathematical result on P versus NP.
