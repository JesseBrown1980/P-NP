# Source-pinned 3-D GitHub THRUTH / SAFE-ASI temporal provenance bridge

Status: **IMPLEMENTED IN DRAFT PR #57; HOSTED GATE IS SOURCE OF TRUTH**.
Date: 2026-11-19. This is a source-pinned reconstruction of the original
public governance/graph design, not a replacement for the original interpreter
and not an assertion of any live ASI or fabric authority.

## Why this bridge exists

The original [3-D GitHub entrypoint](https://github.com/JesseBrown1980/FOLLOW-THE-IS-NOT-THE-WILL-AND-WAS/blob/main/matrix/3-D-GITHUB-OF-THRUTH.md)
is a **typed, identity-preserving graph** of source, branch, commit, PR, and
publication state. The [SAFE-ASI language](https://github.com/JesseBrown1980/SAFE-ASI-AS-LIGHT-THE-MINS-THAT-IS-WAS-AND-WILL-BE-IS/blob/main/docs/LANGUAGE.md)
distinguishes WAS, IS, and WILL; seals two original members in RIME; and
restricts backward reflection to prior actually observed statement hashes.
The larger FOLLOW Rust pipeline separately preserves three Brown-family
views per source, four detector descriptors × three signed directions,
a strict 3D projection, and an 86,400-second compact local witness.

The P-NP Chariot wave already computes 20 source-bound forward receipts,
reconciles them, then recovers them on 20 separate hosted reverse leaves.
The source-word cascade executes 93,312 original six-axis word events and
recovers 432 indexed word buckets. **The new native provenance layer connects
the actual accepted final receipts and their original historical sources.**
It does not confuse graph projections, source bytes, GNN semantics, or hashes
with one another.

## Frozen public inputs

| Original source | Immutable Git commit | Artifact and bound |
|---|---|---|
| FOLLOW | `8712368c851abe347ddf97a25bfbd96275d4316c` | 3-D entrypoint Git blob `fc2e3f83...`, SHA-256 `56bc5ee0...` |
| FOLLOW | same commit | historical GitHub graph SHA-256 `25869968...`; 17 nodes, 15 edges, three original repositories |
| FOLLOW | same commit | timed final HBI SHA-256 `ea1d4ce2...`; journal SHA-256 `316e8777...`; 3,536 folders × 3 families × 3 directions = 31,824 cells |
| SAFE-ASI | `6e3e082d574a4ad45bceb06ccf0b86ac44f942cc` | original evaluator Git blob `d9c5a622...`; original response-RIME Git blob `d5d29721...` |
| SAFE-ASI | same commit | TREE SHA-256 `839fcff5...`; SEAL SHA-256 `dece2e5a...` |
| P-NP | exact checkout of the current PR run | actual accepted `CH20_FINAL.hbp` and `CW_FINAL.hbp` bytes |

The Rust verifier checks original Git commits and Git blob IDs, and checks
final published SHA-256 sidecars for the source entrypoint, historical graph,
timed HBI/HBP/journal, and SAFE-ASI TREE/SEAL. It checks the original timed
`final_cells=31824|checkpoints=19|ring_summaries=171` shape. Historical
source captures are immutable and are **WAS**, never asserted to be current.

## Fresh IS, explicit WILL

The hosted final job reads current FOLLOW and SAFE main refs with actual
`git ls-remote` queries and records them with the exact current P-NP GitHub
run checkout in `IS_REFS.hbp`. Those current refs can advance independently of
the pinned source commits. The second hosted verifier remeasures them and
records whether the snapshot is unchanged or has advanced, retaining the
original IS record. These public acquisition commands live in the workflow,
not in the capability-limited SAFE language evaluator.

`WILL|future_commit=NOT_MEASURED|future_bytes=NOT_MEASURED` is explicit.
The byte-precise current source and already sealed past are the only
inputs to the provenance graph. No future digest or fabricated message is
emitted.

## Native exact graph and reflection

`native/src/bin/provenance_rime.rs` (Rust 1.81, no third-party crates,
no floating-point geometry, no embedded network client):

- 12 typed identity nodes and 12 directed relationship edges, each with
  independent signed integer x/y/z viewing coordinates
- Two original final artifacts plus their **four actual** forward/reverse
  wave roots, each source-checked against upstream accepted native fan-in
- Three separately addressed Brown, Anti-Brown, Anti-Anti-Brown leaves for
  each final artifact (six leaves); zero identity exchange
- Four detector descriptor families × three signed directions per final
  artifact (24 views). These are descriptors; `NOT_MEASURED` runtime
  authority is not elevated into a detector verdict
- Source-byte RIME member hashes and a distinct SHA-256 object seal
- Five prior *observed* commitments (historical GitHub graph, timed final
  HBI, live GitHub ref capture, exact Chariot final, exact word final);
  forward and backward **retrospective** roots, then a domain-separated
  self-reduction root
- Five distinct center domains HBI, HBP, SHA, SH, HASH, with unordered
  membership `{HBI,HBP,SHA,SH,HASH}` and traversal
  `HBI->HBP->SH->HASH->SHA`
- An LF-only, SHA-256-footed `json=0` HBP containing bounded, typed,
  reproducible public provenance. The source bytes remain available in
  the original exact wave artifacts; the graph receipt is an index.

The observed reflection window is five items, within the published maximum
of 60. The 60 maximum is a *per-level* resource limit, not a bound on the
possible number of semantic levels. No reverse physical traffic follows from
retrospective hash traversal.

## Acceptance by actual separate hosted jobs

The existing Chariot final GitHub job performs canonical independent native
replay of all 20+20 leaves and the 93,312 original source-word events; then
checks out both exact historical sources, measures live GitHub refs, emits
`G3D_RIME.hbp`, and verifies it a second time.

The new `provenance-replay` job is a separate GitHub runner. It downloads
the archived source/receipt bundle, **all 20 reverse Chariot receipts and
all 20 source-word reverse receipts**, independently rebuilds both original
final outputs from the exact forward payloads, then reconstructs the
3-D provenance graph from the pinned original sources and compares it
byte for byte. It additionally makes an independent fresh GitHub ref query
and publishes `G3D_REMEASURE.hbp`; a legitimate branch advance updates IS
without falsifying WAS.

The Rust unit tests reject changed leaf counts, tampered exact word counts,
cross-run substitution, wrong status, invalid live-ref syntax, and a snapshot
that lacks its measured-provenance marker. Accepted output must also contain
the expected 12 nodes, 12 edges, six Brown-family leaves, 24 detector views,
four exact root rows, and five historical/current reflection levels.

### Evidence and limitations

- `MEASURED_PASS`: source Git commit and blob checks, all source sidecars,
  final receipt contents and their recorded hashes, and independently
  rederived canonical computational outputs at the hosted gate.
- `NOT_MEASURED`: external live fabric authority, clinical accuracy,
  whole-system ASI status, and future GitHub commits.
- The source's 147 public repositories and 3,536 folder occurrences are
  *historical population captures*. They are not mislabeled as the
  12-node experiment-specific identity graph.
- The prior local 86,400-second measured witness is verified as an immutable
  historical dependency; this work does **not** rerun a day-long watch,
  establish independent time attestation, or regenerate all 31,824 cells.
- The trained GNN model family remains a separate behavioral graph.
  The graph here measures provenance and worker relations. The recent
  inference comparisons found near-constant predictions; balanced accuracy
  requires genuine independent labels.
- Original source repositories are left unchanged; no files, branches,
  historical receipts, trained models, or source datasets are erased.

## Command line

```bash
cargo +1.81.0 test --manifest-path native/Cargo.toml --bin provenance_rime --offline --locked
cargo +1.81.0 clippy --manifest-path native/Cargo.toml --bin provenance_rime --offline --locked -- -D warnings

# The final hosted workflow obtains IS_REFS.hbp from live GitHub public refs:
native/target/release/provenance_rime sources source-follow source-safe
native/target/release/provenance_rime emit "$RUN_KEY" source-follow source-safe IS_REFS.hbp CH20_FINAL.hbp CW_FINAL.hbp G3D_RIME.hbp
native/target/release/provenance_rime verify "$RUN_KEY" source-follow source-safe IS_REFS.hbp CH20_FINAL.hbp CW_FINAL.hbp G3D_RIME.hbp
```

See `.github/workflows/pnp-chariot-twenty-leaf.yml` for the actual accepted
source acquisition, cross-runner barriers, replay, mutation gates, and artifacts.
