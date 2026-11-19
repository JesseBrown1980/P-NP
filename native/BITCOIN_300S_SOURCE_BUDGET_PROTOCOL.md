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

## Extended historical research-arm search: 300s and one million nodes

The newly submitted research-only hosted contest is:
.github/workflows/pnp-bitcoin-300s-research-extended.yml

It runs all four originally recorded research arms (HXF-1, Yang-Mills
Fischer order, modern RIME/Fischer, and byte-original historical Fischer)
on the same nonce32 and merkle_nonce288 full-SHA256d original CNFs.
Each of the eight individual hosted jobs gets a true 300-second wall cap
and a separately declared 1,000,000-node bound, compared with the old
20-second / 1,024-node research restrictions.

The original historical arm, watcher, bitblast generator, Fischer/RIME
source implementations and exact solver-model/header checks remain
byte-identical. A research-only external wrapper changes just the node
budget at the actual historical function call boundaries. This enables
an apples-to-apples longer search experiment without claiming those
historical non-Rust programs have acquired native production acceptance.

Every result must be explicitly labeled SAT, HOLD_TIME, HOLD_BUDGET,
or HOLD_STACK if the historical recursive interpreter reaches its
call-stack limit. The separate referee rechecks original input SHA,
case dimensions, time and node budgets, actual Bitcoin full-SHA256d
candidate headers when SAT, eight corrupted-header controls, all eight
receipts and the archive manifest.

Stage boundaries remain explicit: this test measures longer original
research search; the Rust matrix-funnel test independently measures
source-addressed reversible transport; a later combined solver must
feed its matrix-derived transformations directly into measured search
and outperform direct controls on the same source under full cost
accounting.

This work is committed on the PR branch. Hosted acceptance is always
reported by the exact-commit Actions run results rather than inferred
from the presence of executable code.

## Combined matrix plus raised-node genuine-search gate

The next combined program is now committed separately from the reference
and original research-arm experiments:

- `research/bitcoin_300s_matrix_search.py` orchestrates the original
  source-CNF SHA check, exact Rust `bitcoin_matrix_funnel` attestation and
  full inverse replay, then invokes historical Fischer/RIME ordering and
  the original watched-DPLL algorithm in a second timed process.
- The original 1,024-node stop is replaced in this experimental adapter
  with a 1,000,000,000,000-node ceiling. GNU timeout gives each worker
  exactly 300 seconds for **CNF parsing, route scoring and search together**.
- Six independent jobs cover Yang-Mills Fischer ordering, modern RIME,
  and historical RIME/Fischer, each on nonce32 and merkle_nonce288.
  The companion historical research-arm experiment additionally tests
  HXF-1 with a separate one-million-node ceiling.
- `research/bitcoin_300s_matrix_verify.py` independently checks the
  source hashes and dimensions, rebuilds the native matrix proof, checks
  solver exit and timeout evidence, and verifies the entire signed
  Boolean assignment against every source clause if a SAT outcome occurs.
  It independently reconstructs the 80-byte Bitcoin header and the
  full target double-SHA256 before accepting any SAT result.
- Every result records matrix transport time, timed solver work,
  overall time, CNF digest, matrix root and whether a complete SAT
  model was supplied. Timed-out or interpreter-stack-limited search nodes are honestly
  reported as `UNOBSERVED_ON_HOLD` because the historical watcher does
  not export a partial visited-node counter. A recursive interpreter
  limit is classified separately as `HOLD_STACK` with its own exit code.

Hosted workflow: `.github/workflows/pnp-bitcoin-300s-matrix-search.yml`.

**Evidence boundary:** this combined gate performs exact native matrix
transport and measured heuristic-guided SAT search on the same authenticated
Bitcoin source. The currently transported ternary frames encode a known
public Bitcoin reference header. The SAT search operates on the original
CNF clauses and is never initialized with that known reference assignment.
A later controlled experiment must route the *live search state and
learned constraint transforms* through the matrix before asserting that
matrix computations themselves accelerated cryptographic preimage search.

The standalone MiniSat/CaDiCaL/PicoSAT 300-second baseline finished
with six source-verified `HOLD_TIME` outcomes and full independent
evidence PASS in hosted run #44619920959. A budget HOLD is a measured
unfinished computation. The new combined workflow, distinct
historical raised-node workflow and current-head native policy must
all pass their own hosted acceptance before their new results are
published as accepted.
