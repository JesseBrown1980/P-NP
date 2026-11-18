# Brown Ledger reversible freeze/play — exact native pilot

## Original source and the Shannon payment

The 2026-07-22 Brown Ledger is the shared-center Rime/TRIME closure
and entropy record. Primary archived receipts:
RIME-CONTAINER-RECEIPTS-2026-07-22.txt, REFEREE-HANDOFF.md,
RIME-DEEP-RESEARCH-AUDIT-2026-07-22.md. Its separately published
[three-seat verification PR #42](https://github.com/JesseBrown1980/what-is-asolaria---how-do-we-get-reductions-in-everything/pull/42)
reports forward/reverse order-2 scores 3.1392/3.1388,
order-4 2.2167/2.2158, order-6 2.1594/2.1555 bpc. The
2.0 bpc floor held in that finite PPM-lite harness; it is not a
general entropy threshold.

The exact three-value closure is:

    sum = a + b + c
    center = floor(sum / 3)
    remainder = sum mod 3      (canonical 0,1,2)
    negative = a - center
    positive = b - center

    a = center + negative
    b = center + positive
    c = center + remainder - negative - positive

Shared-center addressing pays for the two separations, remainder,
catalog and decoder. Arbitrary data cannot be reconstructed from
a 2D shadow alone.

## Rust implementation: native/src/brown_ledger.rs

1. Freeze a 27-cell matrix into nine canonical 2D center/negative/
   positive/remainder records, with full source SHA-256. The raw
   27-cell matrix is not also held inside the encoded snapshot.
2. Play by decoding and checking every center/remainder, signed varint,
   range, record endpoint, and complete original SHA-256.
3. Freeze a sparse signed i128 difference between two snapshots,
   retaining both SHA-256 identities. Play forward by addition and
   reverse by subtraction. Enforce both independently reconstructed
   endpoints and reject changed payloads.
4. Address zero cell 13 first, then six face, twelve edge and eight
   corner cells using original Fischer modular addresses. Recover
   each endpoint through the historical smooth-group Pohlig–Hellman
   digit cascade (prime-power towers 16/27/5/463), CRT-recombine,
   and verify the canonical room/glyph against the independent forward
   modular exponentiation. The inverse is exact for the pinned
   1,000,080-order group, not a general discrete-log breakthrough.
5. Replay eight trits through the 256+64 binary floor and inverse.
   Replay canonical binary bytes through the original Q-PRISM
   256↔1024 representation and exact inverse.
6. Verify 3/9/27/81-room waves with the existing N-Nest three seats:
   worker, reflection and independent frozen matrix fabric. Plant
   a balancing fault at every recursion-width control.
7. Reject mutated center fields, mutated signed deltas,
   truncated/extended records, and extreme i64 boundary failures.

No new dependencies, float, unsafe, Node or Python runtime are part
of the Rust kernel. Original historical implementation files remain
byte-identical.

## Information ledger and interpretation

The certified receipt records actual raw i64 bytes, serialized frozen
slice bytes with every closure field and checksum, signed sparse
transition bytes including source/target identities, checked matrix
cells, PRISM/ternary roundtrips and mutation rejections. Source
catalogs, trained model weights, and decoder executable sizes must
be charged if a subsequent comparison claims compression. The
Brown Ledger's 2.0 bpc observation belongs to its own original corpus
and model class, not an arbitrary-data lower bound.

A saved transition can be applied in either direction from an
authenticated endpoint. This does not reverse an arbitrary SHA-256
digest or invert a one-way cryptographic function. All 27 source
values and all 27 resulting values must remain independently
checkable. CPU and transport costs are measured separately.

## Test/acceptance

    cd native
    cargo +1.81.0 test --all-targets --locked --offline
    cargo +1.81.0 clippy --all-targets --locked --offline -- -D warnings
    cargo +1.81.0 build --release --locked --offline
    ./target/release/pnp-native brown-ledger-attest

Expected finite gate: 120 rooms (3+9+27+81), 3,240 played
source cells, 120 mixed-radix and Q-PRISM roundtrips,
244 tamper or triad fault rejections. Exact totals are checked
in code and the host workflow.

Next: introduce bounded MTP shell presentations, original-source
FNN/GNN forward parity and trained reverse-gain arbitration with
a matched byte/time comparison. These neural components remain
HOLD_FORWARD until actual original checkpoint tensor/graph execution
matches independently produced held-out reference outputs.


## OmniMets source-mapped monitoring and convergence (native extension)

The historical OmniMets package is in
[JesseBrown1980/asolaria-behcs-256](https://github.com/JesseBrown1980/asolaria-behcs-256/tree/818cc235a6db815e3b2591339cf9b42aad3e85a0/packages/omnimets).
It has two distinct functions:

- mets.ts records seven lane counters (muscular, skeletal, nervous,
  circulatory, endocrine, digestive, cross): dispatch, promotion,
  deferral, halt, error, latency and sample count, with event rollup.
- convergence.ts defines 6 bodies × 6 reflections × 6 waves ×
  6 phases × 6 lanes × 12 iterations = 93,312 theoretical cells.
  Its configurable convergence rule requires K consecutive independent
  results (historically K=3), and ratification before seal.

The native src/omnimets_ledger.rs adapts these specific observable
semantics to the source-checked five-level kernel, and original Git
tree/blob identities are pinned in CI. It is a bounded K=3 convergence
gate, not an execution of all 93,312 historical convergence cells.

In the native acceptance test, 120 27-trit vectors pass 600
floor messages (64/256/1024/4096/16384 symbols), 480 exact
up/down floor rebasings, and 360 three-child presentations.
At every source room, one trial deliberately requests a
noncertified Omnibit operator at the experimental 16,384 floor,
proving automatic HALT/ERROR and immediate owned-room GULP.

OmniMets counts physical microkernel dispatches:
- 600 five-floor kernel calls;
- 360 child MCP-style room calls and 360 parent calls;
- 120 planted invalid operator calls.

This yields 1,440 total dispatches, 1,320 promotions, 120
halts/errors, 120 recorded early-seal deferrals, 120 ratifications
and 120 seals. Every dispatch has an integer nanosecond latency
sample. Both a per-lane receipt and a SHA-256-linked ordered event
root are emitted. Counts measure executed operations, not a
forecasted agent population.

The 16,384-symbol floor is separately tagged as an experimental
8-trit code. Its reversible representation is tested independently;
the original source Omnibit40 catalog applies at its four certified
floors. Original trained GNN/FNN inference remains HOLD_FORWARD;
OmniMets telemetry does not substitute for trained model inference.

The N-Nest source separately requires a parent to recompute and
verify child presentations, propagating failures upward. OmniMets
ratify-before-seal measures and gates that three-child convergence.
