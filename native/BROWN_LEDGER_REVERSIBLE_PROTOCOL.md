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
