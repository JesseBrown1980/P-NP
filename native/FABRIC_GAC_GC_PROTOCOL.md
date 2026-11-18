# Gate D bridge — matrix-wide neural fabric, GAC tuples and loss-preserving GC

This gate follows the original **Asolaria-as-a-neural-network** law and GAC authority tree. It restores the *fabric-wide substrate* around the previously authenticated nine GNN/FNN checkpoint files, rather than treating the weights as the entire system.

## Immutable original sources

| Original source | Pinned commit | Accepted role |
|---|---|---|
| [ASOLARIA-AS-NEURAL-NETWORK](https://github.com/JesseBrown1980/ASOLARIA-AS-NEURAL-NETWORK) | `4cc7624fb51ff8a64051ace7a1ff62edaa51d422` | Neural-fabric law, 60D root, slice engine, trilateral evidence and storage-backed path |
| [Asolaria-gac-working](https://github.com/JesseBrown1980/Asolaria-gac-working) | `4def18c4bf103a7b5df54c3c0eca7f1d0135b7b4` | 60-dimensional tuple and six-tier/six-scope access policy, GAC-L2 authority and Host-8 migration |
| [asolaria-behcs-256/omni-gulp-gc](https://github.com/JesseBrown1980/asolaria-behcs-256/tree/818cc235a6db815e3b2591339cf9b42aad3e85a0/packages/omni-gulp-gc) | `818cc235a6db815e3b2591339cf9b42aad3e85a0` | Original rotate/archive collector and event summarizer |

All **10 source documents/files** have exact Git blob identities in [GAC_GC_SOURCE_PINS.hbi](../policy/GAC_GC_SOURCE_PINS.hbi). The native certifier checks actual checked-out commit IDs, original file Git blobs, and complete SHA-256 bytes, and stores them in an independent HBI/HBP indexed receipt bundle. The records also include an ordered SHA-256 link chain, 16-character SH routing fingerprints, HBI row offsets/lengths, SHA-256 of original sources and an HBP/HBI archive anchor. SHA/HASH here are integrity commitments relative to anchored Git evidence, not signing keys.

## Architecture boundaries to carry into the native system

The Asolaria neural-network law describes **multiple coupled execution planes**: frozen/borrowed neural slices, the Brown–Hilbert cube/room matrix, trained GNN edges, 9-stage omniflywheel, reflective correction, Shannon/Hookwall/Fischer policy, white-room review and durable storage. The full fabric is more than a single `.pt` graph model.

The **LAW-SLICE-ENGINE** distinguishes a registered address/room from an *executing* agent. The crank sequence is `POP_FROM_POOL → PID_SIGNAL → AGENT_ROOM → RESULT_TO_GULP → ERASE`; erasing the ephemeral runtime room must preserve its result/evidence. This GitHub gate does not connect or actuate any live engine or PID-office registrar.

The original GAC repository defines 60D tuple addresses as the current source frame, with prior 47D/49D/35D as historic bridge strata. The BEHCS-1024 access lattice has **six tiers** (`PUBLIC, RESTRICTED, STEALTH, HIDDEN, SHADOW, SECRET`) and **six scopes** (`drive, usb, device, folder, file, pid`), yielding 36 distinct access cells. The native `FabricTuple` stores exactly 60 signed integer coordinates and an explicit access tier/scope; its 36-cell ordinal encoding round-trips all distinct cells. Individual axis names and glyph/BEHCS decoders remain controlled by the original fabric and require separate source-parity tests before interchange of physical addresses. No claimed 60D memory allocation is triggered by a tuple declaration.

**Authority separation:** GAC-L2 is a governance layer, not authorization to promote/retire/live-launch; operator/council cosign and Hookwall gates remain authoritative. The existing G4 `MISTAKE_FLAGGED` → Fischer Tier-0 block is preserved. Original bilateral neural-fabric evidence uses Acer and Liris plus a possible third independent verifier. A repository file alone is not a fresh live-fabric observation. Source-guided implementation status is `E=0`, read-only.

## Original Omni GC decision semantics

The original `packages/omni-gulp-gc/src/gc.ts` rotates, archives and summarizes NDJSON event files and **never deletes archived audit history**. The native `decide_gc` recreates the measurable decision boundaries without performing file operations:

| Original threshold | Native read-only decision |
|---|---|
| Missing log | `ABSENT` |
| Below 50 MiB and below 100,000 lines | `RETAIN` |
| At least 50 MiB | `ARCHIVE_CANDIDATE_BYTES` (skips expensive line count) |
| Below 50 MiB and at least 100,000 lines | `ARCHIVE_CANDIDATE_LINES` |
| Strictly larger than 500 MiB sanity limit | `HOLD_SANITY` |

Every result has `delete=0`, `execute=0`, `operator_approval=HELD`. The original Node version performs local rotating file operations; the new accepted Rust module is **a planning/proof plane** until source-matched atomic-rotation, mirror-summary and crash-recovery semantics can be verified. It never modifies the source logs, deletes an archived body, or marks an operator decision as approved.

The adjacent `gulp.ts` collects patterns grouped by event-kind and glyph and returns a top-ten summary over a recent window. Classification and severity scoring were still a future stage in that original implementation. Its source is authenticated here; precise source-matched pattern grouping/feedback, GC archive byte identity and GNN receipt routing remain independently tracked.

## HBI → HBP → SHA → SH → HASH acceptance

With the three source revisions checked out:

```text
native/target/release/pnp-native fabric-gc-attest source-gac source-neural source-omni evidence
native/target/release/pnp-native fabric-gc-verify source-gac source-neural source-omni evidence
```

The output is `PNP_FABRIC_GC.hbp`, `PNP_FABRIC_GC.hbi` and `PNP_FABRIC_GC.anchor` with **16 exact receipts**: 10 source identities and six archive-policy boundary tests. The independent checker requires exact byte equality against regenerated HBI/HBP/SHA/SH/HASH evidence. Hosted CI independently hashes all archive files, repeats attestations, checks the index count and tests planted HBI/HBP/anchor corruption and a modified original neural law file.

The native runtime remains Rust 1.81, dependency-free, integer-only, `unsafe`-free, and `json=0`. The source JavaScript is treated as inert evidence, never executed in the accepted process.

## Remaining full-fabric restoration

- Source-matched GULP event grouping and feedback receipts with source-preserving sample handling.
- Lossless archive copy/move protocol, restart/crash replay and protected cosign spine.
- Full 60D original-axis tuple/glyph/BEHCS-1024 parity and GAC access enforcement against real signed source inputs.
- Authentic model tensor-name/shape/stride bindings and all four trained GNN/FNN forward passes, plus balanced held-out behavior.
- Live Host-8/Omni dispatch and authority/cosign behavior, which require the original operator-gated processes and bilateral/third-seat checks.

These remain `HOLD` until independently verified. Current work certifies source identity and bounded read-only policy.
