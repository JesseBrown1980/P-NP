# Original Bitcoin GGUF → Bobby Fischer → resumed SAT

Status: Draft-source implementation. Hosted GitHub Actions acceptance pending
at one exact head commit; PASS must not be claimed until the independent
referee and archive-readback checks finish.

## The precise computational question

Does **actually loading the full original Bitcoin double-SHA256 SAT source
from a GGUF**, thawing its live ternary root assignment, and playing
Bobby Fischer room addresses provide a useful basis for exact DPLL search?

This stage provides a measured, fully charged reference implementation,
rather than counting an in-memory simulation as GGUF playback.

## Three controlled search modes

| Mode | Root assignment | Search decisions | Additional paid work |
| --- | --- | --- | --- |
| direct | All-unknown source-only assignment | Source-frozen Fischer DPLL | None |
| gguf | All-unknown assignment restored from disk-backed GGUF | Identical source-Fischer DPLL | GGUF freeze, write, read, thaw, exact Fischer-address play, live Brown Ledger 27-cell frames |
| guided | Same authentic disk-backed GGUF restore | Matrix-informed candidate ranking, exact DPLL | All GGUF work plus an original-CNF occurrence index and exact matrix room/constraint scoring |

**The resulting root must be identical in all three trials.** There is no
shortcut to the public known genesis nonce. All three run the original CNF
clauses, not a reduced/preconditioned circuit. A completed SAT witness is
checked against the exact CNF and full Bitcoin SHA256d target. HOLD outcomes
are archived and never interpreted as SAT.

The experiment uses **128 node visits per mode and case** as the initial
controlled cost pilot. The real wall limit is **300 seconds per search worker**,
including its source parsing, GGUF file operations, Fischer play, routing,
and SAT search. A timed-out process produces HOLD_TIME with unobserved
partial counters. This node cap must later be raised to test whether a
genuine five-minute exploration improves the outcome.

The independent source referee records its own **per-trial wall time** and
is run after the workers, so its cost is separately reported rather than
unfairly included only in one solver mode's budget.

## Source authentication

- `nonce32`: original source SHA256
  `6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3`,
  239,112 SAT variables, 877,879 original clauses.
- `merkle_nonce288`: original source SHA256
  `601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b`,
  361,188 SAT variables, 1,325,357 original clauses.

The source-to-GGUF byte reader `native/src/bitcoin_gguf.rs`
is modeled on the verified Mythos Blue standard GGUF v3 parser and the
original existing native `fischer_address`/prime-tower inverse.
The unchanged full original DIMACS and a ternary snapshot are stored as
two aligned standard I8 tensors, not inferred from the public genesis
header. `-1 = false`, `0 = unknown`, `+1 = true` in the solver;
the on-disk I8 representation is `255/0/1` byte-for-byte.

## Cost accounting and acceptance

The host-frozen research worker
`native/src/bin/bitcoin_gguf_sat_resume.rs` measures:

- exact source CNF read/parse; GGUF freeze and byte size;
- file write and reread; GGUF strict parse, state SHA and byte identity;
- independently inverted Fischer 27-cell address replay and trace root;
- source-derived Fischer route, actual DPLL nodes, literal checks,
  propagation updates, total wall time, and valid SAT/HOLD state;
- all live Brown Ledger freezes, watcher reads, frozen/metadata bytes,
  event roots and elapsed time;
- guided-mode candidate counts, changed branch choices, occurrence-index
  bytes, matrix replay bytes, metadata, and selection elapsed time.

`research/bitcoin_gguf_sat_resume_referee.py` checks the resulting
artifacts using the separate, pre-existing research-only Python standard
GGUF v3 parser and original Bitcoin source/model SHA256d referees.
It independently checks authentic CNF content, GGUF metadata and tensors,
original all-zero ternary seed, Fischer addresses, time/node counters,
matching original DPLL work between direct and passive GGUF replay, every
claimed SAT model, eight altered-header controls, and separate
per-trial referee overhead measurements.

The workflow `.github/workflows/pnp-bitcoin-gguf-sat-resume.yml`
runs two immutable full-CNF Bitcoin cases × three modes, separately
archives each worker's file manifest, runs the independent referee,
then performs independent acceptance artifact readback.

## Following the actual playback into search

`native/src/solver.rs` now accepts any properly sized partial ternary
assignment as its search root in three variants. This is the input to an
actual recursive DPLL solver, not a replay-only visualization. The
new Rust unit checks freeze a smaller exact SAT source as GGUF, play
its trits back, compare direct against live DPLL tree work, and test
guided exact SAT/UNSAT behavior.

For further rounds, checkpointing a **complete resumable DPLL search**
requires the branch/backtrack stack, unresolved choices, and restart
semantics—not merely the last assignment trits. A future checkpoint
format must preserve that state, and prove replay equivalence before
claiming continued work rather than a restarted constrained branch.
Likewise, any search speedup must be measured end-to-end against matched
external SAT solvers at the entire five-minute limit.

Original Asolaria/Mythos files and historical Bitcoin search sources
remain untouched. PR #57 stays draft pending acceptance.

## Raised-node five-minute original Bitcoin contest

The next control is implemented in
`.github/workflows/pnp-bitcoin-gguf-raised-300s.yml`.
It reuses the *same* original-CNF GGUF binary and independent referee, but
sets `PNP_BTC_NODE_CAP=1000000000000`, so an old 128-node pilot ceiling no
longer ends the experiment early. It compares all six source/solver pairs:
Bitcoin nonce32 and merkle_nonce288, each in direct, GGUF-played and
GGUF-played + matrix-guided modes.

**Wall protocol:** each independent SAT worker has a 300-second external
maximum for original CNF read, GGUF freeze/write/read/thaw, Bobby Fischer
address play, source-only route generation, exact SAT recursion, verification
and output receipt. An internal **285-second absolute deadline** allows
counter/receipt publication with 15 seconds reserved for slow final
serialization. This guard applies in all three raised-node modes; the
pre-existing 128-node workflows remain unchanged. If the externally managed
300-second process kill wins, evidence is `HOLD_TIME` with **unobserved**
partial counters, rather than invented node totals.

Exact DPLL uses a 1,024-level recursion cap only in the raised budget path,
so a branch can honestly report `HOLD_STACK` without process-aborting
stack overflow. `HOLD_NODE` identifies the full one-trillion-node ceiling,
and `HOLD_TIME` identifies an internally timed-out but counted search.
These are distinct from successful source-authenticated `SAT`.

The independent standard-library referee accepts the same provenance fields
for pilot and raised modes. Only fully completed, nontimed direct/GGUF
runs must have identical source-tree counters; time-capped trials can
legitimately visit different numbers of nodes because GGUF playback adds
real overhead. All three paths keep the original clause data and test
any SAT witness against the complete original Bitcoin genesis target.
The referee separately times its own per-trial verification work.

Status remains **queued / acceptance pending** until the exact-commit
Rust 1.81 and independent hosted runs publish final PASS/HOLD records.
A node cap of one trillion is a **safety limit**, not a claim that one
trillion nodes can fit into 300 seconds. No cryptographic preimage speedup
or P-vs-NP proof is inferred from bounded budget results.
