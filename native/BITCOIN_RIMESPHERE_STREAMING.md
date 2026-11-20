# Original Bitcoin SHA256d → streaming ternary RimeSphere → GGUF/Fischer play

**Research status: code prepared on a separate stacked branch; exact-hosted
Rust 1.81 and independently reconstructed original Bitcoin receipts required
before acceptance.** Existing P-NP draft PR #57 remains fixed while its
already queued GitHub evidence runs are pending.

## Why this is the next architectural layer

The earlier Bitcoin GGUF reference freezes the complete original source
directly into two tensors. Its source reader and writer work on an in-memory
copy. The new **RimeSphere is the processing/data transport**, and the GGUF
is the immutable, byte-authenticated hand-off to the downstream native SAT
solver and Bobby Fischer addressing.

The streaming pump consumes **729 source bytes per slice**, emits six
balanced-ternary symbols for each original byte, and places the symbols in
27-cell Brown/Hilbert rooms. Exactly 729 source bytes give 4,374 trits,
162 addressed rooms, and **two 81-room spheres**. A final short slice gets
only the documented number of padding trits, which must all be zero.
The fixed mapping is copied from the existing six-trit Bitcoin matrix
funnel, rather than altering the original SAT source or trying to infer
the public genesis nonce.

Each slice has an immutable **global identity**: (slice number, local
Fischer modular address). `fischer_address(cell, room)` from the
original P-NP `freeze.rs` returns `7^(27·room+cell) mod 1,000,081`
on local rooms 0–161, with the existing `fischer_prime_tower.rs`
reconstructing the original local cell/room. The slice number matters:
the underlying finite smooth-prime Fischer address **repeats** across
larger streams. No global cryptographic discrete logarithm is claimed.

## Streamed artifacts

`native/src/bitcoin_rimesphere_stream.rs` implements the bounded-RAM
data pump and exact source readback. It reads at most one 729-byte
source buffer at once during ingestion, emits an append-only `PNPRIME1`
stream, and produces one index row for every gulp. Each record retains:

- Original byte offset, source count, and source-bound sequential slice ID
- Original chunk SHA256 and canonical balanced six-trit source plane
- Exact 27-cell room count and zero-padded final room
- A rolling SHA256 receipt chaining all prior slices and the current
  source/plane digests

The original source data can be **reconstructed in original byte order**
from the stream alone. The native replay checks every six-trit group,
padding, original-source chunk hash, rolling link, local Fischer sample,
and selected Brown Ledger 27-cell freeze/play. It then checks the
whole reconstructed source against the separately pinned authentic
Bitcoin SHA256d CNF.

`native/src/bin/bitcoin_rimesphere_stream.rs` writes:

| Artifact | Meaning |
| --- | --- |
| `BITCOIN.rime` | Original source represented as streamed 27-cell ternary rooms |
| `BITCOIN.hbi` | Slice-by-slice source offsets, SHA256s, and rolling roots |
| `BITCOIN.replayed.cnf` | Every original Bitcoin clause reconstructed from the sphere |
| `BITCOIN.rimesphere.gguf` | Real standard GGUF v3, full original CNF tensor and all-unknown SAT-state tensor |
| `BITCOIN_RIMESPHERE.hbp` | Source hashes, bytes, trits, rooms, Fischer reads, wall costs and verification status |

The GGUF passes the existing strict parser and original Fischer
address playback. The original Bitcoin SAT clauses remain complete,
with the public known nonce deliberately excluded from the active
search state.

## Independent acceptance

The separate Python standard-library referee
`research/bitcoin_rimesphere_stream_referee.py` parses all RIME binary
records from scratch, checks the entire original CNF byte order,
checks the exact mapping of every byte to six trits, index lines,
rolling digests, geometry, local Fischer address samples, and the
real GGUF v3 tensors against the original CNF. It also runs multiple
mutated-stream controls and eight incorrect Bitcoin genesis-header
controls. It never uses Rust `BrownSlice` or its decoder.

Source-attested cases:

| Case | Variables | Clauses | SHA256 |
| --- | ---: | ---: | --- |
| `nonce32` | 239,112 | 877,879 | `6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3` |
| `merkle_nonce288` | 361,188 | 1,325,357 | `601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b` |

Workflow: `.github/workflows/pnp-bitcoin-rimesphere-stream.yml`.
It is limited by GitHub's platform maximum for hosted jobs rather than
by the earlier artificial five-minute SAT benchmark. Both original-source
cases must pass independent replay and archive readback.

## Scope and next connection

**Stream play, GGUF play, and SAT solving are distinct stages.** This
stage certifies the first two: original Bitcoin equations can actually
travel through a nested ternary RimeSphere and reconstruct byte-for-byte,
ready to be consumed by the previously implemented GGUF-resumed solver.
It reports `sat_search=NOT_RUN`; the search has not been started by this
new ingestion workflow.

The source representation is **not a size compressor**. Its raw
27-cell signed-byte plane uses approximately six bytes per original
source byte, plus fixed per-slice headers and HBI indexing. It is a
reversible, independently addressable computational arrangement. The
compression/learned-constraint hypothesis remains a separate benchmark.

For truly **continuous SAT play** across runner sessions, the next stage
must persist the entire DPLL search frontier, branch polarity state and
backtracking stack along with the frozen GGUF. A last-assignment-only
snapshot cannot represent the unfinished search. Once this exact
frontier playback passes independent restart parity, RimeSphere chunks
can support ongoing CPU/HTTP workers without an artificial 300-second
algorithmic termination condition.

Original Mythos Blue GGUF and historical RIME laws are preserved
unchanged; this is a new native-source integration.
