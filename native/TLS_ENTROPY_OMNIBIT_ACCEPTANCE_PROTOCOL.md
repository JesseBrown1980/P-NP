# Temporal-memory probability coding × Omnibit40: precommitted acceptance gate

Status: **protocol / implementation pending**. The base is merged P-NP PR #44, commit `62682cabfcb5cbc0cf6700b7cabb2469af12a35e`. This protocol freezes the accounting and acceptance rules before examining a new entropy-coded result.

## Purpose and exact population

Take the authenticated `PNP_TLS_STOCHASTIC_TRACE.bin` from accepted PR #43, source-pinned by the accepted PR #44 archive, and its nine disjoint 8,000-event train / 4,000-event validation / 8,000-event held-out scenarios. Each has a count-preserving 8,000-event time shuffle. Train / validation / held-out separation remains unchanged. No TLS generator, hidden reservoir bit, held-out-label fit, hardware data, or oracle conditional may enter a decoder.

There are **18 unique test streams**, 144,000 source event bits, and separately **54 route frames** (three reversible representations of each stream). These are distinct denominators. Never interpret the 54 frames as 54 independent datasets.

The fixed reference points inherited from accepted PR #44 are:

| Reference | Exact bytes | Accounting |
|---|---:|---|
| 18 distinct streams, raw 0/1 byte-per-event | 144,000 | Source dataset |
| 18 distinct streams, unframed one-bit-per-event | 18,000 | Theoretical byte pack |
| All 54 routes, unframed one-bit-per-event | 54,000 | Corresponding 54-frame pack |
| Original 54-frame identity-only Omnibit | 357,338 | Same existing route/frame layout |
| Selected 54-frame original Omnibit40 | 294,615 | Accepted PR #44 payload |
| PR #44 full release executable | 1,449,920 | Historical *reference* decoder size; remeasure changed executable |

The accepted 54-frame Omnibit40 + full binary result is 1,744,535 bytes. Never use its 294,615-byte numerator in a 18-stream comparison. For apples-to-apples comparisons, add complete actual headers and other metadata to the corresponding bit pack and report both framed and unframed totals.

## Prior-trained model state and causality

Use accepted PR #43's five models, indexed 0 / 1 / 3 / 27 / 81. The source model is defined in `native/src/tls_stochastic.rs`: fixed-point Q10,000 event probabilities; a train-only global prior; EMA coefficient and mean; and count/success tables of sizes 3 / 27 / 81 with the fixed 12 / 48 / 96 pseudocounts. Predictions are clamped to 1..=9999.

Serialize the **exact sufficient snapshot** required to decode each model: selection/version and integer parameters, relevant table counts, table successes, prior and EMA parameters. Charge these bytes in every score. Reuse a snapshot across streams/routes **only if a single retained archive physically contains it once**. The decoder may load this embedded snapshot, but never an external trained-model file or original training events. Both streams of each regime/seed pair share the same immutable train-derived model.

Model selection must use **validation data only**. Freeze two labeled arms separately: (A) the historically accepted validation-Brier-plus-penalty selector; (B) minimum actual validation **framed coded bytes including serialized model state**, with deterministic tie breaking to lower model capacity. Freeze the winner and its snapshot before encoding held-out/shuffled streams. Also publish all five per-capacity ablations, with the selection costs charged. No held-out entropy score may choose a predictor model.

At every event the decoder computes the next-event probability from previously reconstructed events only, then updates context with the newly decoded event. Reset context at the start of each independent stream. Context crosses block boundaries within a stream, never crosses held-out/shuffle or scenario boundaries.

## Four reversible paths and the hybrid

Run all the following arms with the same source-event inputs and explicit source SHA anchors:

1. **BITPACK:** exact 0/1 bit packing, with unframed and fully framed references.
2. **OMNI40:** byte-identical reference to the accepted standalone original-operator/RLE encoder, including operator IDs, route IDs and residuals; report identity-only too.
3. **TEMPORAL_RANGE:** deterministic integer-only binary arithmetic/range coding with conditional `p/10000`; encode the events in causal original order. Fix the interval width, renormalization, termination, padding and error handling in code before hosted measurements. Charge block lengths and coder-reset overhead. Decoder must reject malformed/truncated/trailing streams, not silently infer missing bits.
4. **TEMPORAL_RESIDUAL_OMNI40:** predict a hard bit from Q10,000 `p>=5000`, form reversible `residual = event XOR predicted_bit`, pack residuals via the accepted binary/ternary/mixed bridge, then apply all **40** frozen source operators and literal/RLE encoding to each 20-symbol block. Charge the operator ID, bank modes, complete residual, frame and selection. At decode, inverse-transform the full residual block, reconstruct original events sequentially using prior decoded-event context, and advance state.
5. **HYBRID:** for each source-compatible block, compare complete actual bytes of TEMPORAL_RANGE and TEMPORAL_RESIDUAL_OMNI40, including an explicit stored mode selector. Store the cheaper *actual* candidate; tie break deterministically. Model selection remains validation-only; per-block codec selection is permitted only because each choice is present in the archive and its exact byte cost is charged.

Report the direct probability coder, residual-transform coder and hybrid separately. A hybrid selected entirely from one arm is reported as such; do not count the mere presence of Omnibit as demonstrated additive synergy. To attribute combination gain, compare against **both** individual arms on the same route/frame population, including all selection bytes.

Every inverse must be source-independent: all 54 route frames and the 18 unique streams must recover the original authenticated 8,000 0/1 symbols per stream. Raw event histories, trained predictor tables and source PRNG seeds are not external runtime dependencies.

## Byte-accounting and scientific scorecard

For each regime × seed × held-out/shuffled × route report: model capacity, snapshot bytes, selector bytes, mode bytes, residual bytes, entropy bytes, framing/padding bytes, total archive contribution, operator histogram, decode SHA, and coded bits/event. Include a full 18-stream aggregate and a separate 54-frame aggregate. Report the total trained-state charge once according to the actual serialization; count source-event repetitions correctly.

Report three explicit size columns for every arm:
- **Payload only:** the actual durable framed encoding, complete embedded trained-state snapshot, predictors/selector/mode/residual/padding bytes.
- **Payload + executable:** payload plus actual full `pnp-native` release-binary byte size, charged once. A stripped/minimal standalone decoder can be a *separate* arm only if actually built and archived.
- **External evidence:** HBI/HBP/anchor, decoded replay witness, SHA manifest and any protected original retained by policy, reported explicitly and consistently; do not hide them within another arm or silently equate protected source with disposable scratch.

Preserve the absolute benchmarks 18,000 and 54,000 unframed packed bytes and the 294,615-byte accepted Omnibit payload. The current full executable is larger than the packed references; a payload win and a self-contained binary win are **separate findings**. No claim that lower Brier error necessarily means fewer code bits: measure actual integer-coded bytes.

Record exact hosted CPU/wall, peak RSS, encoder and decoder throughput. Separate full upstream source attestations from steady-state coding time. Publish held-out vs identical-count shuffled controls, IID-null controls, all capacities, and original-operator versus identity controls; include negative or null results.

## Native acceptance and third-seat witness

Rust 1.81.0, `--locked --offline`, no external crates, `#![forbid(unsafe_code)]`, strict Clippy, integer-only runtime, no Node, no JavaScript runtime, no Python or JSON runtime. Keep the accepted original 40-operator catalog untouched. Preserve the existing binary–ternary route validation and the full source-scoped Git blob provenance gate.

Implement CLI boundaries (names reserved for the subsequent implementation):

```text
pnp-native tls-entropy-attest source-omnibit evidence evidence
pnp-native tls-entropy-verify source-omnibit evidence evidence
pnp-native tls-entropy-decode evidence/PNP_TLS_ENTROPY_SELECTED.bin restored.bin
pnp-native tls-entropy-gulp-discard source-omnibit evidence evidence
```

Accept only if:
- All original upstream PR #43 and #44 exact archives and SHA roots are authenticated.
- The independent decoder reads only the selected archive and the compiled executable, reconstructs every bit exactly, verifies archive canonicality and rejects corrupted model snapshots, coder state, padding, selectors, HBI/HBP/anchor and prior-root alterations.
- Source-bound 3-seat checks compare **worker encode**, **reflection standalone inverse**, and **fabric source/HBI/HBP witness** and reject coordinated false claims; independent published SHA manifest checks run in a distinct hosted job.
- Deterministic regenerate-and-compare yields byte-identical archives and exact integer statistics. Every observed compression verdict is recorded whether PASS, NO_GAIN or HOLD.
- The hosted report contains framed and unframed bitpack, identity-only, Omnibit40, entropy and hybrid totals with full model and decoder charges, unchanged denominator definitions and elapsed/RSS measurements.

`EXACT_REPLAY_REQUIRED` source is retained. After complete signed HBI→HBP→SHA→SH→HASH acceptance, autonomously create, fsync, read back, verify, then physically delete **owned disposable post-GULP scratch only**; publish GC receipt and confirm source archive is intact. No live operator approval in event, coder-selection or GC loops. Crash-atomic protected-source rotation and concurrent-reader GC remain independently held until their own tests pass.

## Results discipline

This document specifies a **new test**, not a claimed entropy-coding measurement. Native implementation, hosted run, negative controls, and independently replayed output must all be completed before changing the result boxes in `README.md`. The observed outcome may establish payload gain, additional operator–prediction benefit, self-contained gain, or a valid null result. Record every outcome without changing the frozen success definitions after seeing held-out bytes.
