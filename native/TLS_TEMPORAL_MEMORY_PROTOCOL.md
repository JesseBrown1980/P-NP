# PRL qubit-environment memory: classical, fixed-point latent-reservoir gate

Scientific reference: Nicolas Gosling and colleagues, *Probing the Memory of a Superconducting Qubit Environment*, accepted by Physical Review Letters 2026-10-28, DOI [10.1103/j1r2-5ph7](https://doi.org/10.1103/j1r2-5ph7). This PRL work reports correlated quantum jumps and hidden two-level-system (TLS) reservoir effects. The source identifier and scope are frozen in `native/fixtures/PRL_TLS_SOURCE.hbi`, and its SHA-256 is carried through the native HBI/HBP evidence.

**Scientific scope:** This gate implements a controlled, **classical**, discrete-time Solomon-population-inspired model. It is neither an experimental qubit result nor a fit to the authors' recorded quantum jumps. The fixed-point parameters are synthetic and were selected to create falsifiable null/coupled cases. It does not measure g₂ directly or establish information-bearing quantum coherence.

## Fixed-point model and independent calculation

The classic one-TLS population equations are

```text
dp_q/dt = -Gamma_q(p_q-p_th) + Gamma_qt(p_t-p_q)
dp_t/dt =  Gamma_qt(p_q-p_t) - Gamma_t(p_t-p_th)
```

The native solver uses `scale = 10,000`, `p_th=2,000`, `Gamma_q=900`, `Gamma_t=150`, a coupled total `Gamma_qt=1,200` or a memoryless coupling of zero. It advances via an explicit fixed-point Euler tick, checks `0<=p<=10000` after every tick, and allocates the total coupling across **0, 1, 3, 27, or 81 latent TLS cells** with an integer remainder-preserving split. `0` cells forces total coupling zero. Each case retains both hidden trajectories for 32 ticks.

The verifier independently recomputes the visible one-step population from the **net-flux form**, summing individual signed reservoir exchanges before integer rounding. This is a separate formula from the simulator's weighted-output implementation, and the paired histories must match exactly. It also independently validates a rounded alternative decomposition. Arithmetic uses Rust 1.81 integers without external crates.

## Causal test: identical visible starting state, different hidden history

Each of the five cell counts is tested under coupling ON/OFF, with three frozen visible starts `5000, 3000, 7000`, two distinct hidden initial populations (`500` and `9500` for each resident cell), and 32 future ticks.

- **10 model configurations:** five cell counts × enabled/disabled coupling.
- **30 paired histories** (60 explicit trajectories): 18 memoryless negative-control pairs (including the zero-cell cases) and 12 genuinely coupled pairs.
- **150 fixed-lag observations** across lags `1, 2, 4, 8, 16`.
- Every memoryless pair produces **exactly zero** conditioned difference; each coupled pair has a **positive** difference after one tick. The frozen one-tick separation is `1080` Q10,000 units at the same observed `q`, with high vs low hidden excitation.
- A **history-label shuffle** explicitly swaps the hidden TLS inputs while preserving the visible qubit population, verifying that the correct next-step prediction swaps and a visible-state-only baseline cannot distinguish them.

The source-free memoryless baseline is a population update conditioned only on visible `q`. For coupled trajectories it accumulates nonzero sum of squared deterministic prediction differences; the independently computed full TLS model has zero difference from its own generator. This is **generator-aware deterministic reconstruction**, not trained held-out prediction, a claim of generalization, or physical quantum hardware data. Splits labeled `DEVELOPMENT` and `HELDOUT` are fixed initial-condition partitions under the same known generator.

## Source-linked HBI → HBP → SHA → SH → HASH acceptance

```text
pnp-native tls-memory-attest evidence
pnp-native tls-memory-verify evidence
```

The simulator publishes **196 indexed HBI/HBP receipts**: one DOI/scope, 10 model configuration rows, 30 paired-history rows, 150 lag observations, 4 automatic GC eligibility cases, and one summary. Evidence includes complete two-history fixed-point state trajectories in `PNP_TLS_TRACE.bin`, per-history SHA-256 and byte offsets, a deterministic HBI offset/row length index, SH prefixes, linked HASH row chain, and `PNP_TLS.anchor` over all final roots.

A second hosted job checks the signed summary controls and the full independent artifact digest list. CI re-runs every accepted test under **Rust 1.81, strict Clippy**, rejects deliberately modified HBI, HBP, anchor, trace, and detects modified source manifest bytes.

## Automatic GC retention policy

The same original Asolaria [GULP/GC policy](GC_ARCHIVE_RESTART_PROTOCOL.md) applies, without live human approval:

| Data class | Post-GULP committed? | Complete future state + independently replayable? | GC eligible? |
|---|---|---|---|
| Summary-only scratch | Yes | Not needed for summary | Yes |
| Exact-replay temporal memory | Yes | No | HOLD |
| Exact-replay temporal memory | Yes | Yes | Yes |
| Uncommitted summary | No | Irrelevant | HOLD |

This gate **only verifies eligibility decisions**. Physical deletion of owned summary-only event files is independently implemented in the earlier accepted GC gate. Deletion of exact-replay-required temporal state remains held until atomic crash recovery and reader-concurrency acceptance.

## Next evidence necessary

1. Validate discretization convergence and the link to source-defined physical rates with an independently implemented numerical model.
2. Generate distinct seeded, stochastic photon-jump histories and measure conditional event probabilities and lag correlations, including shuffled-order controls. Compare against a properly held-out baseline with uncertainty intervals.
3. Evaluate latent capacity 0/1/3/27/81 by predictive quality **and** exact memory/decoder costs, separating overfitting from useful hidden memory.
4. Connect the resulting frozen history traces to actual Omnibit 40-operator coding and existing source-authenticated multi-GNN forward gates.

The published PRL study establishes measured quantum-environment memory. This Rust experiment tests a narrow **classical computational analogue**, with explicit status boundaries.
