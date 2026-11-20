# Live Bitcoin SAT branch selection from exact 27-cell matrix states

Status: **draft experimental implementation**. Hosted Rust 1.81, independent
source/referee and archived evidence acceptance must PASS at one exact commit.

## Purpose

The Bitcoin source benchmark now compares three computational paths on the
same exact Boolean CNF and the same native DPLL algorithm:

| Mode | SAT search | Matrix role |
| --- | --- | --- |
| direct | Original Fischer source-only variable order | No matrix operations |
| live | Identical direct search decisions | Source-bound 27-cell state freeze/play and watcher at each changed decision/propagation room |
| guided | Original exact DPLL with candidate choice recalculated from active constraints | All live watcher frames plus 27-cell room replay and source-indexed candidate ranking |

The native solver independently verifies any completed SAT model against the
unmodified original clauses. The external research referee independently
reconstructs the complete 80-byte Bitcoin header and checks both SHA256
rounds against the source-pinned 256-bit genesis target.

## Actual state entering the funnel

`native/src/live_sat_matrix.rs` carries the **real** Boolean search
assignment: `-1` false, `0` unassigned, `+1` true. 27 adjacent Boolean
variables form a Brown Ledger room. Each source SHA, variable address,
state digest and frozen byte representation is bound to an exact playback
check. Search state evolves as DPLL branches and propagates; the published
historical reference Bitcoin nonce is never used as search guidance.

`native/src/matrix_selector.rs` builds a source-authenticated occurrence
index from the original CNF clauses. At each open DPLL decision, it:

1. Takes up to 27 currently unassigned candidates in the existing Fischer
   source-only priority order; candidate count and index bytes are charged.
2. Freezes each distinct candidate room, replays it from the exact matrix,
   verifies byte-for-byte equality to the actual live solver assignment.
3. Evaluates each candidate's source clauses, using replayed ternary states
   for any indexed matrix rooms, and the original search assignment for
   variables outside the replayed candidate rooms.
4. Ranks currently unsatisfied clauses by the remaining number of unassigned
   variables, using exact integer weights 81/27/9/3/1 (for 1/2/3/4/5+
   open literals). Ties retain the original source priority.
5. Chooses the highest-scored eligible variable. The exact SAT solver
   subsequently searches both Boolean polarities as required.
6. Hashes the state/frame commitments and selected choice into a rolling
   source-bound receipt.

Unit propagation, SAT truth semantics, original clauses and independent
final model checking remain unchanged. Only candidate priority is variable.

## Matching and metering

The first hosted control uses the authentic nonce32 and merkle_nonce288
genesis full-SHA256d CNFs, **128 search nodes** and a **300-second maximum
wall time** for each of six separate trials (three modes x two sources).
The 128-node bound is deliberately held fixed to isolate selection overhead.
It is a pilot, alongside the existing external-solvers' accepted full
300-second calibration (six budget HOLDs), and ahead of the raised-node
orchestration/search study.

Both original-CNF source dimensions and SHA256 digests are immutable.
Every completed receipt distinguishes `SAT`, `HOLD_NODE`, or
`HOLD_TIME` (with unobserved partial counters when timed out).
Search nodes, propagations and literal checks are separately recorded from
matrix frame count, watcher reads, paid frozen bytes, metadata bytes,
matrix time, selector candidates, original-clause reads, index bytes and
selection time. The guided mode can change the node count and tree; the
direct and observation-only controls must preserve identical search work.

The standard-library `research/bitcoin_matrix_guided_referee.py` uses
the separately maintained `bitcoin_live_sat_referee.py` original source
checker. It audits the six original-CNF results, source identities, complete
Bitcoin headers for SAT witnesses, receipt meters, selector proof roots,
eight deliberately altered headers, and full artifact manifests.
Archived readback follows the independent referee.

Workflow: `.github/workflows/pnp-bitcoin-matrix-guided-selector.yml`.
Rust driver: `native/src/bin/live_sat_benchmark.rs`.
Exact solver integration: `native/src/solver.rs`.

## Subsequent experiments

After same-head native acceptance, increase the search-node ceiling while
keeping the full 300-second source-to-receipt budget; perform matched direct,
passive-live and guided trials with independent source held-outs. Compare
total elapsed time, node visits, actual search completion, frozen bytes,
watcher cost and matrix-derived choice counts. Include a source-randomized
route and an identical-cost clause-scoring control to separate the benefit
of active-constraint selection from the cost of matrix representation.

An observed shorter search is a finite benchmark result. General P versus NP
claims require separate universal mathematics and independently checked proof.
