# Stochastic TLS temporal memory — frozen held-out predictive acceptance

Scientific inspiration: Nicolas Gosling et al., [*Probing the memory of a superconducting qubit environment*, PRL accepted October 28, 2026](https://journals.aps.org/prl/accepted/10.1103/j1r2-5ph7), DOI `10.1103/j1r2-5ph7`. The paper experimentally distinguishes non-Poissonian quantum jumps caused by long-lived two-level-system (TLS) defects from memoryless dissipation using real hardware and fitted Solomon equations. The current P-NP work is a **separate synthetic, classical, discrete-event model**, building on [the accepted fixed-point reservoir pilot](TLS_TEMPORAL_MEMORY_PROTOCOL.md). Hardware jump data, physical TLS rate fits, quantum coherence and physically calibrated `g₂` are outside the present acceptance claim.

## Frozen generator: three causal regimes

The [public corpus definition](fixtures/TLS_STOCHASTIC_CORPUS.hbi) fixes the PRNG, independent seeds, event probabilities, sequence lengths and selector policy before the hosted calculation.

| Regime | Number of underlying synthetic hidden reservoir bits | Conditional event probability | Hidden transition probability per tick |
|---|---:|---|---|
| `IID_NULL` | 0 | 4,500/10,000, constant | None |
| `ONE_TLS` | 1 | 800/10,000 (inactive), 8,300/10,000 (active) | 800/10,000 |
| `THREE_TLS` | 3 | (1,000+2,500 × active cells)/10,000 | 200, 500, 900 per 10,000 |

The chosen probabilities and transition rates are **frozen synthetic test settings**, not numbers extracted from the experiment. Hidden bits persist between ticks, and the observed event is generated from their current state using a fixed xorshift32 integer PRNG. Every regime runs **three independent seed specifications**. Each seed creates disjoint pseudorandom streams of 8,000 training, 4,000 validation and 8,000 held-out events; a separately seeded Fisher–Yates permutation provides 8,000 count-preserving, time-shuffled held-out events. All input trace bytes and their SHA-256 digests are retained as certified evidence.

These are independent fixed PRNG runs, not a train/test split of the exact same physical qubit trace. The train-only model receives only event observations, not the source's hidden bits, transition rates or held-out labels.

## Native memory capacity and integer scoring

Five predictors are fitted on training observations only, using integer probability scale `Q10000`:

| State capacity | Frozen model definition |
|---:|---|
| 0 | Constant event rate estimated on training data, the memoryless reference |
| 1 | One EMA hidden scalar; train-fit signed correlation coefficient, limited to the accepted integer range |
| 3 | Table indexed by the ternary count of events in the previous 2 time steps (−/0/+ or 0/1/2) |
| 27 | Table keyed by three successive, overlapping 2-event ternary observations |
| 81 | Table keyed by four successive, overlapping 2-event ternary observations |

For each table, observed training successes and counts are smoothed using a **train-only global prior** and frozen pseudocounts 12/48/96. The next-event prediction reads only the *past*, then updates its state with the outcome; held-out model parameters are never refit. The 3/27/81 counts are real trit-addressed state-table capacities, while 1 represents a real persistent EMA scalar.

The objective is the **sum of squared Brier errors** over 8,000 held-out binary event outcomes, scored with exact signed integers. The winner among capacities 0/1/3/27/81 is selected **solely from validation Brier plus an explicit, fixed complexity penalty of 500,000,000 × model index**. The frozen held-out and shuffled results are examined only after selection. A selector chosen on held-out Brier would invalidate the acceptance.

## Causal and negative controls

The primary criterion is not simply whether a generator-aware oracle can predict its own output. The successful regime must show a validation-selected, observation-trained memory model improving *unseen* held-out events over the constant-rate baseline. The time-shuffled held-out events must have **the same number of events**, but lose the held-out temporal-memory advantage. The IID null must not spuriously select a memory model under the frozen validation penalty.

This experiment also computes each held-out event conditional by five time lags `1,2,4,8,16`: the empirical probability of a later event given that an event did or did not occur at the earlier time step, with raw numerator/denominator counts. The same conditional difference is measured on shuffled events. These are **g₂-like binary event correlations**, not the paper's experimentally measured quantum g₂.

The precommitted acceptance checks are:
- 3 IID-null repetitions selecting baseline capacity **0**.
- 6 coupled repetitions selecting a history-aware model with a lower held-out Brier sum than the baseline.
- All 6 coupled repetitions losing that advantage on shuffled held-out events.
- Positive lag-one event association in all 6 coupled repetitions, with the source-defined fixed synthetic threshold. Every model and lag value is published, including larger capacity models that do not win.

This is a bounded hypothesis test on **three selected synthetic scenarios**; broader distributions, device data, statistical intervals, cross-implementation replication and larger independent seeds remain separate acceptance work. The selection penalty and deterministic precommitted generator are not a general proof of predictive learning capacity.

## HBI → HBP → SHA → SH → HASH and GC retention

The exact native Rust 1.81 commands are:

```text
pnp-native tls-stochastic-attest evidence evidence
pnp-native tls-stochastic-verify evidence evidence
```

The gate first re-verifies the previously accepted **196-row PRL-inspired integer reservoir pilot** and binds the prior HBP SHA-256 root into every new row. It emits **117 HBI/HBP receipts**: 1 source, 9 trace groups, 45 model Brier scores, 45 lag comparisons, 9 validated selector decisions, 3 per-regime aggregates, 4 original GC-policy controls, and 1 final summary. It retains the complete training, validation, held-out and shuffled event byte streams in `PNP_TLS_STOCHASTIC_TRACE.bin`, all offsets and per-stream SHA digests, and a final HBI/HBP/trace chain anchor.

The independent hosted evidence job verifies the exact SHA manifest and row census. Negative tests reject modified HBI/HBP/anchor/raw trace bytes and changes to the accepted prior TLS source receipt; full regenerate-and-compare is deterministic.

No per-step human operator approval is needed. The accepted `SUMMARY_ONLY` GULP policy allows deletion after durable summary commitment; `EXACT_REPLAY_REQUIRED` hidden histories and source data require complete retained state and independent reconstruction. This gate verifies admission only; **it does not delete protected hidden-state or held-out event traces**.

## Research and implementation boundaries

Actual quantum defects may have more complex kinetics than these three discrete hidden Markov models. Their rates were not inferred from the paper. These measurements do not establish superconducting device error correction, qubit coherence recovery, P=NP, a continuum Yang–Mills mass gap, or compression gains. Connecting the source-authenticated 40 Omnibit operators, the four trained GNN families, physical source data, and memory budget accounting requires separate measured gates.
