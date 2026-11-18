# Native ternary SAT engine family and Relic rediscovery specification

**Scope:** Rust 1.81.0, std-only offline locked build, Clippy `-D warnings`, exact integer arithmetic, 0 Node.js, 0 JSON, 0 V8, no floating-point, no `unsafe`. These are new *native* engines; the 54 pinned historical Python/CI artifacts remain `FAIL_NATIVE_ACCEPTANCE` until separately ported and verified.

## Design corrected by the Relic reference

Original source: [THE-RELIC-REDISCOVERY](https://github.com/JesseBrown1980/THE-RELIC-REDISCOVERY/blob/main/README.md), [exact interface specification](https://github.com/JesseBrown1980/THE-RELIC-REDISCOVERY/blob/main/FRAME-AND-INTERFACES.md), [verified source](https://github.com/JesseBrown1980/THE-RELIC-REDISCOVERY/blob/main/frame_interfaces.py). Original tree at `1ea1ad48969380b25d8fc485ba74c1f405106ef2`.

The bounded machine uses balanced trits **minus (-1), free/unassigned (0), plus (+1)**. One invariant free *reference center* `C=1` has coordinate `0`; it is **not** the same as the value `0` of an unassigned Boolean variable. Each address is a vector in `{−1,0,+1}^d` with exactly `3^d` leaves; finite prefixes select lazily computed subtrees.

The Relic **intermediate x3 waves** and **structural landings** are deliberately distinct:

| Landing n | Depth e(n) | Outward A(n) | Intermediate wave values and next landing |
|---|---:|---:|---|
| 0 | 3 | 27 | 81 → 243 |
| 1 | 5 | 243 | 729 → 2,187 → 6,561 |
| 2 | 8 | 6,561 | 19,683 → 59,049 → 177,147 → 531,441 |
| 3 | 12 | 531,441 | continues ×3 five times to 129,140,163 |

Exact recurrence `e(n)=3+n(n+3)/2`, `A(n)=3^e(n)`, `A(n+1)=A(n)*3^(n+2)`, with a single invariant `C=1`. The physical interpretations in the historical repository remain research hypotheses, not SAT proof rules.

Exact frozen 2D representation for each ternary triple `(x0,x1,x2)`: `S=x0+x1+x2`, `d_i=3*x_i-S`, `d2=-d0-d1`, with inverse `x_i=(S+d_i)/3` when division is exact. The Rust tests exhaustively verify injectivity on all 27 trit triples and reject malformed inversions. Intensity-only measurements intentionally fail to reconstruct ordered triples.

## Current native SAT engines

| Label | Dynamic candidate guidance | Routing structure |
|---|---|---|
| `base` | clause activity / original variable tie order | 3 lanes |
| `mtp` | frozen forward and reverse order-two clause trit sign counts | 3 lanes |
| `fischer` | integer source-derived 27-point Fischer sphere addresses | 3 lanes |
| `nest3`, `nest27`, `nest81` | original-CNF matrix fabric observations verified by worker and reflection | 3, 27, 81 actual lanes |
| `hybrid3`, `hybrid27`, `hybrid81` | combined frozen MTP, Fischer, and verified matrix third witness | 3, 27, 81 actual lanes |

All nine use the **same complete ternary-state DPLL correctness core**, with exact unary forced-literal propagation and two nonzero Boolean alternatives per decision. A fanout 3, 27, or 81 is a **routing and fabric width**, not extra truth values for the SAT input. Clause-model validity is independently checked. No search-node limit is needed for a completed small trial; if one is specified and reached, the result is `HOLD_RESOURCE` (never an invented UNSAT verdict).

## Reproduce

Run in `native/`: `cargo +1.81.0 test --all-targets --offline --locked`, `cargo +1.81.0 clippy --all-targets --offline --locked -- -D warnings`, `cargo +1.81.0 build --release --offline --locked`. From repository root:

```text
native/target/release/pnp-native relic
native/target/release/pnp-native sat hybrid27 native/fixtures/sat3.cnf
native/target/release/pnp-native sat hybrid81 native/fixtures/php4_3.cnf
native/target/release/pnp-native sat hybrid81 native/fixtures/php4_3.cnf 0
```

The native solver tests exhaustively compare 2,304 small CNF/strategy outcomes against a separate complete assignment enumerator and replay 9 configurations on SAT, UNSAT, and PHP examples. The original source pin remains enforced; no legacy executable is edited. The native workflow also checks output determinism, source-only guidance, third-seat tamper controls, compiled project integer LLVM IR, and independent uploaded evidence recheck.

## Phase-2 exact third-seat matrix witness (PR following #27)

The frozen 27-cell room is now witnessed at **each of its nine 2D columns** by an exact, reversible 3-layer triple `(S,d0,d1)`. The data cells remain stored: each `S=x0+x1+x2`, `d0=3x0-S`, `d1=3x1-S`, and `d2=-d0-d1`; the independent matrix seat reconstructs `x_i=(S+d_i)/3` with exact divisibility and signed range checks. The 2D shadow is still retained for provenance, but equality of aggregate totals or shadows is not an acceptance criterion. The third seat checks original ordered values at every `(room,column,depth)` address.

The fail-closed hierarchy retains all 42 established single-report faults and additionally exercises **42 cancelling ±1 report errors** (same shadow column, aggregate sum unchanged) plus **14 coordinated worker/reflection report corruptions** across all 3/9/27/81 supervisor levels. Exact reconstructed triples cover **1,080** addressed columns across 120 rooms. Passing reports are emitted by `pnp-native certify` as `PNP_NATIVE_WITNESS` and independently checked in the HBP archive replay. Source-cell and witness mutations, including a balanced two-cell mutation that preserves the old shadow, have direct negative tests.

Each room retains 27 `i64` source words, 9 `i128` shadow words, and 27 `i128` reversible witness words: **99 `i64`-equivalent words per room**, deliberately storing redundant information for verification. This is a verification plane, not a compression claim or a cryptographic source authentication layer. Immutable Git source pins and SHA-256 evidence remain responsible for source provenance; a coordinated alteration of every source and witness surface requires external authenticating evidence to detect.

## Further certification work

**HOLD for historical engine equivalence:** The archived MTP textual trained weights, GNN traversal-generated functions, FNN, Host-8 tournament, OmniDispatcher, OmniSpindle, OmniRouters, OmniHotel/OmegaHotel, and full original HXF structural certificates are not yet native Rust ports. Current MTP counts are trained *locally from input clause trits*, without external trained weights. The Fischer integer branch-prior is an exact generator-based addressing heuristic, not cryptographic inversion.

**HOLD for universal complexity:** Proof-carrying UNSAT replay for arbitrary-size CNF, blinded SAT/Bitcoin benchmarks with matched resource budgets, and a polynomial worst-case complexity theorem are separate acceptance milestones. This release targets a correct, auditable native ternary kernel, not a P=NP claim.