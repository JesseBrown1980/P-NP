# Whole matrix neural-fabric integration: first executable gate

## Original architecture and actual connected data path

PRL DOI 10.1103/j1r2-5ph7 -> source-scoped classical Solomon/TLS memory experiment -> synthetic held-out/shuffled stochastic temporal events -> PR43 accepted trace -> PR47 same 54 source-authenticated 27-trit jobs ->
four authentic binary floor encoders -> every original Omnibit40 reversible
operator -> native Fischer address -> backend-only HTTP-0 -> separate Rust
worker process -> original operator inverse -> separately implemented radix-3
decoder -> independent 243-step 27-cell integer computation -> return 27 trits
and 27 exact field values -> original-source verification -> frozen reversible
27-cell matrix worker/reflection/fabric triad -> GAC 60D frozen tuple metadata
-> real G4 GLSM missing-GNN-signal gate -> source-anchored HBI/HBP receipts.

This joins actual previously accepted code paths, rather than simulating
their isolated results as a combined system.

64 symbols / 6 bits carries 3 trits; 256 symbols / 8 bits carries 5;
1024 symbols / 10 bits carries 6; 4096 symbols / 12 bits carries 7.
The final incomplete word has canonical zero-trit padding. The worker
rejects reserved 256 codes 243..255, invalid ordinal codes on all other
floors, invalid padding, unknown operator IDs and invalid wire lengths.
The 40 catalog operators comprise RNQ8, TRI12 and PI20, and the
worker invokes their original compiled invert_symbols functions.

54 source jobs × 4 binary floors × 40 original operators =
**8,640 matched crossings per arm**. The same workload runs at physical
room capacities 3, 27 and 81. Each capacity has a no-cache and a
source-keyed memory arm: **51,840 complete HTTP-0 cross-system calls**.
No-cache performs 8,640 real integer recurrences per capacity;
cache must perform 54 and return 8,586 verified hits. Every request
carries transformed floor symbols, not source events or the hoped-for
result. The parent verifies all 27 reconstructed trits and every one
of the 27 computed cells for each request.

The original accepted microkernel-compute test supplies the exact
54-task corpus and reference recurrence, without a second generator.
All 54 task outputs are frozen into 18 native reversible triads,
checked independently by worker/reflection/fabric code. Cached
result identity is tied to authenticated source-task ID, seed, all 27
trits, arity and Fischer room address. The original 8-byte per-room
state is preserved. Direct same-workload calculation and direct
memoization are measured as separate controls.

## PRL memory research is the source of the original workload

Nicolas Gosling et al., *Probing the memory of a superconducting
qubit environment*, Physical Review Letters, accepted October 28, 2026,
DOI 10.1103/j1r2-5ph7:
https://journals.aps.org/prl/accepted/10.1103/j1r2-5ph7

The paper reports experimental superconducting qubit jump dynamics
influenced by persistent two-level-system reservoirs; fitting
Solomon equations and examining non-Poissonian jump timing exposes
the temporal-memory structure.

P-NP already used that finding to motivate TWO explicitly classical
and synthetic steps. First, its fixed-point Solomon-inspired
two-level-reservoir model tested memoryless versus retained-state
dynamics with 0/1/3/27/81 latent cells (native/TLS_TEMPORAL_MEMORY_PROTOCOL.md).
Second, its stochastic train/validation/heldout experiment
generated independent synthetic IID_NULL, ONE_TLS, THREE_TLS
event streams, matched count-preserving temporal shuffles,
tested Brier-score predictive advantage and lagged correlations,
and froze the accepted source bytes
(native/TLS_STOCHASTIC_HELDOUT_PROTOCOL.md).

The temporal source archive is the actual data source for the later
Omnibit40, fully framed entropy, independent HTTP-0 fabric and
four-floor matrix tests. Each full-matrix acceptance now binds
the DOI-containing PNP_TLS.hbp and PNP_TLS_STOCHASTIC.hbp receipts
as **two explicit SHA-256 provenance roots**, in addition to
the original source trace bytes. The DOI identifies the external
scientific inspiration; measured hardware jump data are not
present and no quantum physical fit is claimed for our
synthetic held-out source.

## Genuine neural and matrix source dependencies

Eight existing accepted artifacts become mandatory provenance
roots: PNP_TLS.hbp, PNP_TLS_STOCHASTIC.hbp, PNP_BRIDGE.hbp,
PNP_OMNI40.hbp, PNP_GNN_GATE.hbp, PNP_CHECKPOINT.hbp,
PNP_FABRIC_GC.hbp, and the exact PNP_MICROKERNEL_RESULTS.bin.
The first two archives must contain DOI 10.1103/j1r2-5ph7.
The new HBI/HBP archive records all eight actual prior SHA-256 roots
as individually authenticated receipt rows. This preserves the original four trainable
GNN families, all nine source-authenticated Q20 model checkpoint
imports, the G1/G2/G3/G4/OmniShannon/SHA family map, and GAC 60D
source law. Each of 54 source tasks also receives an original native
60-coordinate FabricTuple bound to the authentic source SHA, with
roundtrip and access-cell checks.

The authentic G4 state machine is actually evaluated on all 54
tasks with missing trained GNN antecedents, and must stay HELD.
Model tensor name/shape/stride, graph operators and activations
still require accepted source-matched forward parity before
trained model outputs can steer the runtime. The GNN source roots
and G4 HOLD are real checks in this gate; this does not claim a
trained GNN forward pass or a live 60D physical-axis map.

## Independent third seat and evidence

The producer uses the accepted binary_ternary_bridge implementation
and original 40 operator source. The independent OS process uses its
own ordinal radix-3 expansion with canonical padding checks, the
original operator inverse, and the previously separately written
integer stencil. All reconstructed source trits and exact field
elements are checked byte-by-byte by the supervisor. This constitutes
a genuine independent decoder and arithmetic calculation across
the backend process boundary; operator forward and inverse share
the original catalog specification.

Negative controls reject wrong route, bad operator ID, bad wire
version, reserved codes, padding corruption, archived data corruption
and altered HBI. The hosted CI reruns deterministic receipts and
then verifies the published archive hashes in a separate job.

Native CLI:
    pnp-native full-matrix-attest evidence evidence
    pnp-native full-matrix-verify evidence evidence
    pnp-native full-matrix-inspect evidence/PNP_FULL_MATRIX_RESULTS.bin

Evidence outputs: PNP_FULL_MATRIX_RESULTS.bin, PNP_FULL_MATRIX.hbp,
PNP_FULL_MATRIX.hbi, PNP_FULL_MATRIX.anchor and the hosted SHA
manifest. The original source archive and trained model bytes
remain protected; prior automatic post-GULP scratch deletion is unchanged.

**Acceptance:** the first execution passed both jobs in [run #44354419893](https://github.com/JesseBrown1980/P-NP/actions/runs/44354419893) on 2026-11-18. The subsequent dedicated wrong-floor/padding-control change is subject to a fresh hosted rerun before promotion. Original trained GNN forward remains a separate HOLD.


## First hosted result, 2026-11-18 (run #44354419893)

The native Rust 1.81 gate passed with 107 unit tests, no Clippy
warnings, an exact-integer LLVM check, and independently rechecked
published evidence. All 51,840 cross-system calls reconstructed both
the 27 original trits and 27 output field elements exactly.

| Source/room configuration | Calls | Actual recurrences | Memory hits | Visited rooms | Single-run wall |
|---|---:|---:|---:|---:|---:|
| Direct same-crossing processor | 8,640 | 8,640 | 0 | in-process | 139.261 ms |
| Direct source-keyed memo | 8,640 | 54 | 8,586 | in-process | 10.643 ms |
| Fischer HTTP-0, width 3, no recall | 8,640 | 8,640 | 0 | 3 | 493.261 ms |
| Fischer HTTP-0, width 3, recall | 8,640 | 54 | 8,586 | 3 | 435.330 ms |
| Fischer HTTP-0, width 27, no recall | 8,640 | 8,640 | 0 | 27 | 506.776 ms |
| Fischer HTTP-0, width 27, recall | 8,640 | 54 | 8,586 | 27 | 439.431 ms |
| Fischer HTTP-0, width 81, no recall | 8,640 | 8,640 | 0 | 54 | 516.303 ms |
| Fischer HTTP-0, width 81, recall | 8,640 | 54 | 8,586 | 54 | 365.735 ms |

Each room-capacity column has two matched native-process arms with
the same original task data and same cross-product of original
operators and four exact binary floors. HTTP-0 worker is sequential;
performance differences across widths are single-run observations,
not concurrent scaling or trained model advantage. The source
authenticated four-GNN-family and nine-checkpoint provenance,
54 native G4 missing-signal holds, 54 GAC 60D **metadata**
roundtrips and 18 exact third-seat matrix triads passed.

This is the first connected binary/ternary/Omnibit40/Fischer/HTTP-0
compute/witness trial. The next neural acceptance gate must implement
original-model tensor naming, shapes, stride/adjacency and activations,
and match independent source forward outputs on held-out graphs
before introducing learned GNN routing or learning claims.
