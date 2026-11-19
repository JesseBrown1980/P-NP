# Bitcoin SHA256d five-minute budget calibration

Status: implementation in P-NP draft PR #57. Acceptance remains dependent on the exact
GitHub Actions workflow results at the tested commit.

## The corrected experimental interpretation

The November 17 Bitcoin genesis benchmark imposed 20-second wall-clock limits
on the four research engines, 12-second limits on external SAT solvers, and an
independent 1,024-node search cap inside the historical research adapters.

A HOLD from either cap records an unfinished computation. The five-minute
experiment must distinguish a time limit from a node limit before drawing
any conclusion about the cost of the hard source instances.

## First five-minute reference experiment

The newly added workflow, .github/workflows/pnp-bitcoin-300s-source-reference.yml,
source-pins the original Bitcoin genesis header, full target double SHA256,
and original two difficult CNF workloads:

- nonce32: 32 free nonce bits, 239,112 variables, 877,879 clauses.
- merkle_nonce288: 256 free Merkle-header bits plus 32 nonce bits,
  361,188 variables and 1,325,357 clauses.

MiniSat 2.2.1, CaDiCaL 1.7.4 and PicoSAT 965 each receive exactly
300 seconds per source case, yielding six separately hosted comparisons.

The original historical Python generator, original runner and original
model-to-CNF witness checker remain byte-unchanged. The explicit external
research wrapper only replaces the 12-second budget with 300 seconds.
Source original SHA values, complete model witness SHA256d, solver
exit statuses, stdout/stderr, elapsed wall-time and RSS are archived.
A separate hosted referee rechecks the public Bitcoin header and full
hash, original source SHA, every result receipt, eight altered-header
negatives and budget consistency, followed by a separate archive readback.

Any solver time expiration remains HOLD_TIME. Neither budget expiration
nor a completed SAT instance is a general complexity theorem.

## Following experiment: the actual orchestration

The original research adapters require a separately controlled node budget.
Merely waiting five minutes while the original 1,024-node bound is still
active would not extend their computational search.

The subsequent Rust-native matrix experiment should feed the same
original Bitcoin CNFs through a source-authenticated funnel, generate
bounded 27-cell ternary slices, measure Fischer/Omnibit routing,
preserve freeze/play and source-recoverable A/B/P memories,
run independent worker/reflection/fabric verification, and match final
solutions against the original CNFs and full SHA256d digest.

Record the entire source-to-receipt computation cost, all routing and
storage metadata, search nodes, misses, memory use and completed results.
Compare against direct matching-budget solver paths, with repeats and
negative controls. Report each observed numerical advantage and every
HOLD independently.

The discrete-universe model is a motivating hypothesis for this engineering
architecture, while its physical interpretation and P=NP mathematical
status remain separate research problems.

## Native exact 27-cell funnel: first integration stage

Source: native/src/bin/bitcoin_matrix_funnel.rs
Hosted workflow: .github/workflows/pnp-bitcoin-matrix-funnel.yml

The actual historical nonce32 and merkle_nonce288 full Boolean CNFs are
ingested by the Rust 1.81 binary, authenticated against their pinned SHA256
digests, and independently parsed as exact DIMACS clauses. The program
constructs the published Bitcoin genesis 80-byte reference header solely for
a source-path round-trip experiment. This reference header is excluded from
the search input supplied to the experimental SAT solvers.

Each header byte is represented by six balanced ternary digits (480 data
trits total), padded by six neutral cells into 18 exact 27-cell frames.
Each frame traverses the existing Fischer null-outward addresses and is
inverted using the pinned prime-tower addressing. Every state is preserved
through Brown Ledger freeze/play; every transition is preserved through
exact signed BrownDelta forward/reverse operations. The program reassembles
all 80 original bytes, independently recomputes the full published Bitcoin
double-SHA256 digest, and rejects header, source-CNF, receipt, and frozen
state modifications. It emits exact HBP text, SHA256 evidence roots, and
elapsed-time metering. Hosted matrix workers run both original source
cases and a distinct replay/referee step.

This stage tests a real Bitcoin data funnel, source identity, reversible
transport and exact verification; solver_search=NOT_RUN is an explicit
receipt field. Measured compute cost includes the parsed source CNF and
reversible transport; the separate 300-second external-search reference
provides the first timing baseline.

Following acceptance gate: integrate a genuinely time-bounded,
nontrivially searchable branch/candidate generator with this Rust transport
and compare its outcomes under exactly matched 300-second budgets.
Separate search nodes, state/wave reuse, transport bytes, watchdog checks
and end-to-end wall duration. The historical 1,024-node stop must be
removed or treated as a separately reported bounded HOLD.
