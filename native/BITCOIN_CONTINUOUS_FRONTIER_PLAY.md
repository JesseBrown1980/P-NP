# Bitcoin three-shadow SAT: complete-frontier continuous play and equal-work control

**Stage:** source-pinned research, on a branch stacked on the accepted
one-vs-three original Bitcoin SAT [PR #60](https://github.com/JesseBrown1980/P-NP/pull/60).
The original two-case, six-mode-node pilot already passed independent
[GitHub Actions #44742147572](https://github.com/JesseBrown1980/P-NP/actions/runs/44742147572).
The new 8→32→128 node frontier experiment requires its own exact-commit
Rust 1.81 acceptance and independent source referee.

## Why an assignment-only GGUF checkpoint is insufficient

The original Bitcoin RimeSphere streams and Mythos-style GGUF checkpoints
represent the complete original CNF and a ternary source state. Replacing
the all-unknown state with the *last* assignment would lose which Boolean
branch polarities have already been explored, how far propagation has
progressed, and which independent decision branches remain.

The new `native/src/continuous_frontier.rs` runs real original-CNF
**iterative exact DPLL**, with each complete node visit using exact
Boolean unit propagation and a source-frozen Fischer candidate order.
The selector is one orthogonal X signed shadow, three independent X/Y/Z
shadows with an exact eight-trit inversion check, or a direct no-shadow
control. Both Boolean ± branches remain available; 0 denotes unknown.
All original clauses and exact root GGUF bytes are unchanged.

Every state transition is represented by:

- A live signed ternary assignment reconstructed from the original GGUF
  root plus an ordered variable/polarity assignment **trail**
- An explicit stack of original variable numbers, exact trail
  rollback marks, and whether the **minus branch** has started
- Complete visit, propagation, literal-check and branch-decision counters
- The original source SHA256, Fischer route SHA256 and GGUF ternary seed
  SHA256, plus deterministic per-node and per-branch rolling SHA256 roots
- Source-signed shadow projection/reconstruction counters and actual
  snapshot storage sizes

Unlike a recursive call-stack snapshot, this is an explicit, parseable,
versioned source-bound binary: `PNPFRT01`. Its trailing SHA-256 checks
the entire record. The independent parser verifies all trail entries
are unique/assigned, every frame refers to the precise Boolean
polarity at its rollback mark, frame marks increase monotonically,
and the original source/mode/root bindings never change.

Each `advance(quota)` commits a *chosen amount of new node work* and
stops **only between nodes**. The active branch stack and trail are then
frozen. A subsequent process checks the original source, original GGUF
and the previous checkpoint SHA, rebuilds the exact live assignment,
and resumes **at the next unvisited node**.

**No algorithmic timer or fixed cumulative node ceiling exists in
this implementation.** The work quota is a session boundary, and each
runner naturally remains subject to its platform's resource limits.
To keep playing indefinitely, a controller must schedule additional
sessions while preserving their finalized artifacts. An infinite
runtime on a single hosted GitHub runner is neither assumed nor
required.

## Exact acceptance plan: more equal work on real Bitcoin

The hosted workflow `.github/workflows/pnp-bitcoin-frontier-continue.yml`
tests the original, full SHA256d `nonce32` and
`merkle_nonce288` CNFs with the *same original GGUF* previously
certified in [three-shadow run #44725165515](https://github.com/JesseBrown1980/P-NP/actions/runs/44725165515).

Six comparable groups are tested:

| Original Bitcoin CNF | Direct Fischer DPLL | One X shadow | Three X/Y/Z shadows |
| --- | --- | --- | --- |
| `nonce32` | Source control | Live X-pixel scores | Live X/Y/Z scores and exact inverse |
| `merkle_nonce288` | Source control | Live X-pixel scores | Live X/Y/Z scores and exact inverse |

Each group performs **four executable runs**, with genuine runner-session
separation and source/GGUF readback:

1. Begin at the accepted all-unknown GGUF; visit up to 8 nodes;
   freeze `FRONTIER.frt`.
2. Download and verify the checkpoint on a *different hosted runner*;
   visit 24 new nodes, reaching 32 cumulative nodes; freeze again.
3. Download that checkpoint to a third hosted runner; visit 96
   new nodes, reaching 128 cumulative nodes; freeze again.
4. Separately start from the original GGUF and visit 128 nodes
   **uninterrupted**, making a reproducible reference.

The independent referee `research/bitcoin_frontier_referee.py`
decodes each frozen record without importing native Rust, confirms
original Bitcoin CNF and GGUF SHA256, exact chains between the three
sessions, *every* trail/frame and polarity, and the raw work receipts.
It then checks split-versus-uninterrupted **exact** parity for terminal
state, source visits, propagations, literal checks, choices, ternary
trail, pending frames, first Fischer choice and full progress SHA root.
For any claimed SAT result it also checks every original CNF clause
and the real Bitcoin SHA256d target. For unfinished searches, the
state remains `RUNNING`, with its real frontier available to a future
session. It is not promoted to UNSAT or SAT based on node limits.

The new worker's native receipt `BTC_FRONTIER_SESSION` charges
source parsing, original GGUF readback, source routing,
frontier deserialization, exact SAT work, actual shadow encoding and
the atomic file/write/sync/reopen checkpoint overhead.

### Scientific interpretation

The accepted 2-node precursor established that three 2D shadows
selected different *first* original Bitcoin variables:

| Case | Source-Fischer candidate | One shadow | Three shadows |
| --- | ---: | ---: | ---: |
| `nonce32` | 124,075 | 124,075 | 11,030 |
| `merkle_nonce288` | 244,087 | 114,107 | 10,142 |

The larger 128-node result will measure whether those decisions
meaningfully affect exact source work and overhead. Even exact
split-run parity alone is a **checkpoint correctness result**,
not a claim of cryptographic preimage acceleration or a P=NP proof.

The source and code remain pinned to Rust 1.81, no external runtime
dependencies, `#![forbid(unsafe_code)]`, and strict Clippy acceptance.
Historical Mythos/RIME source, PRs #57–#60 and `main` remain unchanged.
