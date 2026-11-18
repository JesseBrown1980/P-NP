# PRL temporal memory: actual causal prediction through the binary–ternary matrix fabric

Scientific origin: Nicolas Gosling et al., *Probing the memory of a superconducting qubit environment*, Physical Review Letters, accepted October 28, 2026, DOI https://doi.org/10.1103/j1r2-5ph7.

This experiment directly tests whether train-only temporal memory can predict future synthetic events better than a constant-rate baseline after passing through real Omnibit40 binary/ternary HTTP-0 matrix rooms. The paired count-preserving time shuffle tests whether the advantage depends on event order, as the PRL-inspired memory hypothesis predicts.

## Exact original data

The experiment takes the original accepted PNP_TLS_STOCHASTIC_TRACE.bin and its source hashes. There are nine precommitted cases: three repetitions each of IID_NULL, ONE_TLS and THREE_TLS. Every case retains 8,000 training, 4,000 validation, 8,000 unseen held-out, and 8,000 count-preserving shuffled held-out Bernoulli events. The PRL research inspires the separate **classical synthetic** reservoir simulation; hardware quantum jump observations and physically fitted TLS parameters are not available as inputs.

For every case, the original Rust integer predictor is fit from training events only, and its 0/1/3/27/81 memory capacity is selected from validation Brier plus the frozen original complexity penalty. The same frozen fitted predictor is used on both held-out orders, with no held-out fitting or selection. The independent constant Q10000 prior is the memoryless baseline.

## Native data path

Full 8,000-event streams are split into 297 consecutive 27-trit frames; remaining cells in the last frame use canonical neutral ternary padding. Every original source event crosses four binary floors, 64/256/1024/4096, then original RNQ8/TRI12/PI20 Omnibit40 invertible operators rotated across all 40 IDs. Backend-only HTTP-0 dispatch uses original Fischer room addresses at capacity 3, 27 and 81. The remote separate native process independently decodes all binary/trit representations and calculates the original source-trained Brier model using context updated **only after prediction**. Each frame also independently computes the 27-cell exact 243-step integer recurrence. The first-party controller reconstructs all input trits and all 27 computed output values and compares every per-event forecast against source-fitted native recomputation.

Expected runtime: 9 cases × 2 stream orders × 4 floors × 297 frames × 3 capacities = **64,152 native HTTP-0 calls**; the full comparison includes **1,728,000 forecasted events**, each with baseline and source-trained memory score; and **108 matched chronological/shuffle paired score rows**. Every arity processes 72 complete 8,000-event streams and all 40 original operators.

The nine original trained PyTorch GNN checkpoints and the authentic G4 GLSM gate remain protected source dependencies. Native trained-GNN tensor forward parity is still HOLD; the actual trained component in this experiment is the already accepted train-observations-only classical integer TLS-memory predictor. No synthetic GNN/HRM score is substituted for a learned graph neural result.

## Falsifiable gates

1. Nine fixed validation selections, with 3 IID choices of zero memory and 6 coupled choices selected on validation alone.
2. Across all 3 room capacities and 4 floor widths, all 36 IID pair rows show no false memory improvement.
3. All 72 coupled chronological pair rows show positive held-out improvement over the identical memoryless prior; their matched 72 time-shuffled rows lose that improvement.
4. For each chronological/shuffled source pair, baseline Brier errors must match exactly (equal positive-event counts).
5. Each of 1,728,000 probabilities must match the source-trained independent controller prediction; 64,152 frames must reconstruct every trit, exact finite matrix output, and sequential causal state.
6. Wrong Fischer address, invalid Omnibit ID, out-of-order frame and source/model corruption reject; G4 holds untrained neural input.
7. Source hashes of original PRL DOI-bearing TLS pilot and stochastic heldout archives, all40 full-matrix certificate, original GNN checkpoint ledger and full-matrix witness must be bound into each new deterministic HBI/HBP evidence chain.
8. Regenerate 120-indexed-row archive and frozen canonical arity-3 stream forecast file byte-identically. Reject tamper. Independent hosted SHA-manifest job passes.

Native commands:

    pnp-native prl-temporal-fabric-attest evidence evidence
    pnp-native prl-temporal-fabric-verify evidence evidence
    pnp-native prl-temporal-fabric-inspect evidence/PNP_PRL_MEMORY_FORECAST.bin

Generated files: PNP_PRL_MEMORY_FABRIC.hbp, PNP_PRL_MEMORY_FABRIC.hbi,
PNP_PRL_MEMORY_FABRIC.anchor, PNP_PRL_MEMORY_FORECAST.bin,
PNP_PRL_CAUSAL_DIGESTS.sha256.

Interpretation: a positive result establishes an exact finite classical, synthetic, causal prediction gain transmitted through this real native multi-plane implementation, with a count-preserving negative temporal-order control. It does not measure physical qubit data, trained-GNN inference, P=NP, or parallel scaling. Those remain source-scoped research gates.

**Hosted acceptance:** PASS on [run #44397648431](https://github.com/JesseBrown1980/P-NP/actions/runs/44397648431), both Rust 1.81 source-certification and independent published-artifact SHA jobs successful. 109 native tests PASS. The additional acceptance result is committed only after a passing final run of these documentation changes.

## Actual measured Brier evidence

The original source has **three independent seeded synthetic repetitions per regime**, nine case groups in total. The four floor representations and three room widths replicate the **same nine** source cases, rather than producing 108 statistically independent datasets. The table below uses one canonical 3-room/64-symbol floor view per original source group, and aggregates three separate source repetitions per regime.

All source outcomes are 8,000 held-out events per repetition. Brier scores are mean squared forecast error for binary outcomes, calculated from the exact Q10000 integer sums. The percentage gain is (memoryless error − fitted-memory error) / memoryless error.

| Original synthetic regime | Source repetitions | Validation-selected capacities | Mean memoryless Brier | Mean learned-memory Brier | Held-out Brier gain | Shuffled order gain |
|---|---:|---|---:|---:|---:|---:|
| IID_NULL | 3 | 0, 0, 0 | 0.24785035 | 0.24785035 | **0.000%** | **0.000%** |
| ONE_TLS | 3 | 27, 27, 27 | 0.248735675 | 0.180554214 | **+27.411%** | **−15.946%** |
| THREE_TLS | 3 | 27, 1, 1 | 0.249471095 | 0.234687417 | **+5.926%** | **−2.647%** |

At the individual held-out source-repetition level, ONE_TLS shows gains 28.202%, 25.080%, 28.960%; THREE_TLS shows gains 6.298%, 5.965%, 5.515%. All six distinct coupled source repetitions reverse sign with the original count-preserving shuffle. The single-run native HTTP-0 wall times for 21,384 requests / 576,000 scored events at each room capacity were 1.608 s (3), 1.664 s (27), 1.664 s (81). These times include native transport, matrix computation, prediction and independent response checking, and are not a trained-GNN speed comparison.

The full accepted evidence contains:
- 64,152 native HTTP-0 frame deliveries (all source trits and 27 matrix values exact).
- 1,728,000 Bernoulli event predictions with two Q10000 probabilities per event, matched independently against the original fitted integer model.
- 108 paired chronological/shuffled Brier receipts: 72 coupled positive-and-reversed rows, 36 IID null rows. Each original source pair is replicated across four floor formats and three room widths.
- 120 source/score/third-seat rows in HBI/HBP/SHA/SH/HASH; 72 canonical forecast stream vectors stored in the binary archive with source digest 24c8c55b95b1272025146023e4cdd87064aa8b0c778360e6008782a4d9f3c894.
- Complete regenerate-and-compare, byte tamper/reordered-route negative controls and separate published-artifact SHA acceptance.

Scientific interpretation: ordered synthetic reservoir event history contains statistically usable causal information for the fixed training-only predictor under these precommitted settings. The native four-floor/Omnibit40/Fischer/HTTP-0 matrix path carries that information without changing the integer forecasts. The result is a finite, source-authenticated **classical computational observation** using the PRL-inspired test scenarios; the real qubit jump dataset and trained GNN forward model remain separate research gates.
