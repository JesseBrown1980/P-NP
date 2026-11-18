# Host-8 and OmniMets legacy reuse — source reconciliation (2026-11-18)

## Decision: port and bind original semantics, do not invent another disk router

The current Brown Ledger / layered-sphere Rust kernel is a **small, in-process protocol
acceptance harness**. The storage-addressing architecture predates it. The original
Host-8 room maker, Brown–Hilbert/prime-cylinder rules, OmniMets/OmniScheduler
telemetry and GNN/Shannon/white-room stage are separate, source-grounded components.
Future functional work must bind their canonical algorithms and receipts, with
byte-for-byte parity before promoting a replacement.

### Primary historical artifacts

- [Original Host8 Rooms on Storage technical audit](https://github.com/JesseBrown1980/Asolaria/blob/main/docs/HOST8-ROOMS-ON-STORAGE.md).
- [Original Host8 Rust room-maker](https://github.com/JesseBrown1980/Asolaria/blob/main/docs/handoffs/acer-host8-rooms-guide-2026-10-19/host8rooms/src/main.rs):
  `TritWord` from `tribit/mod.rs`, 8-byte `SplitMix64`-finalizer
  invertible stub, `locate(n)=(n/rps,(n%rps)*8)`, command set
  `make/verify/probe/seal`. That is the canonical disk layout.
- [Host8 room build/verification](https://github.com/JesseBrown1980/Asolaria/tree/main/docs/handoffs/acer-host8-rooms-guide-2026-10-19/receipts):
  original 125 sectors × 100,000,000 rooms × 8 bytes = 100,000,000,000
  bytes and 12.5 billion rooms, sector 0..124 (presented with balanced
  negative/positive labels), readback of every stub reported with zero
  mismatches; independently reconstructed and three sectors fully
  rechecked in the cloud audit.
- [Original 50-million-room arena](https://github.com/JesseBrown1980/Asolaria/blob/main/docs/HOST8-ROOMS-ON-STORAGE.md):
  400 million bytes; modulo-slot aliasing must be distinguished from
  the later injective direct-sector window.
- [Q-PRISM Host8 cube code](https://github.com/JesseBrown1980/Q-PRISM-human-organoid-neural-stream-as-a-high-dimensional-control/blob/main/host8/qprism_cube_host8.rs):
  SHA-prefix/Host8 identity and 3,200 **bytes** -> 2,560 10-bit
  glyphs -> 3,200 bytes (exact 256↔1024 representation).
  A 3,200-byte tuple is not 3,200 separate tuple rooms.
- [Prime cylinder spinner source](https://github.com/JesseBrown1980/Asolaria/blob/main/prime_cylinder_expansion.py):
  branch-count `product(p_i**n)`; this is a generative address
  geometry and its measured result is arithmetic, not a 200-ns HDD I/O.
- [OmniMets original source](https://github.com/JesseBrown1980/asolaria-behcs-256/blob/main/packages/omnimets/src/mets.ts):
  per-lane dispatch/promote/defer/halt/error/latency counters plus
  persisted NDJSON telemetry. That module is a telemetry collector;
  the router and scheduler are separate packages and policies.
- [Host8 / OmniMets migration specification](https://github.com/JesseBrown1980/asolaria-trilateral-host8-kernel-ladder-20260708/blob/main/artifacts/TRILATERAL-HOST8-KERNEL-LADDER-20260708.md):
  explicit **port, not reinvent** intent. Preserve `ROOMSTUB`,
  Host8 identity, event taxonomy, PID/idempotency/stale-lock policy,
  bounded retry, and HBI/HBP receipts. The July packet is design
  evidence (`process_launch=0`), not a deployed Rust port.
- [Shannon and GNN post-trigger stage](https://github.com/JesseBrown1980/Shannon-and-the-gnns-stage):
  HOOKWALL -> GNN ensemble (L0/L4 and G1/G2/G3/G4) ->
  Shannon/OmniShannon -> white rooms KEEP/COMPACT -> GULP ->
  Path-1 retained recall or Path-2 CRT reconstruction, with
  deterministic inverse/re-projection witnessing.

### Architecture to reuse

```
source pulse / prime-cylinder spinner
  -> Brown-Hilbert tuple + exact Host8 address
  -> original disk sector-offset / preallocated ROOMSTUB
  -> [EMPTY -> LOAD -> RUN -> COLLECT -> EJECT -> EMPTY]
  -> immutable, verified output envelope + chosen next-shell tuple
  -> next shell / drive receiver address
  -> Hookwall + real trained GNN score (when loaded) + Shannon
  -> white-room KEEP/COMPACT and paid HBI/HBP/GULP
  -> optional freeze / arbitrary-center Fischer inverse + rewind
```

Temporary scratch is discarded after committed output. Archival
signatures, source bodies required for recovery, trained weights
and learned/reduced products have different retention policies.
Deterministic watchers are not evidence that a trained GNN was loaded.

Each 64/256/1024/4096 floor is a **symbol cardinality**
(6/8/10/12 bits per symbol). Host8 is separately an **8-byte
identity or room stub**. A 16,384-symbol floor is an experimental
new rung and must not borrow the original Omnibit40 14-bit
certification. Numeric 2D shadows are projections; original data
are restored only when the retained store or collectively
sufficient closure/shadows supply the missing information.

### Timing and data movement: separate three clocks

1. Spinner *issue interval* (operator recalls 200 nanoseconds):
   record on CPU with actual implementation, thread count and
   scheduler. Historical documents checked here have not
   authenticated a 200-nanosecond end-to-end storage transaction.
2. Room *resident cache* time: original ACER receipt reports
   299,885 reads/second (3,334 ns) on warmed D: sector; one cloud
   `pread` benchmark reported ~554-658 ns per hot room.
3. Room *cold disk* time: ACER receipt reports at most
   98 random reads/second (~10.16 ms/read) across sectors.
   The spinner can emit while the storage queue is busy; emission
   frequency is not service completion frequency.

The original WSL `/mnt/d` route measured 590 lookups/second versus
516,528/second on the Linux-native temporary file for the same
class of bytes. The verified implementation should avoid this
path penalty and use retained hot mailboxes or batched I/O.

### Acceptance additions for native integration

- Replay original `host8rooms` direct-location and invertible
  mix/unmix test vectors in independent Rust; pin source blob.
- Preserve original 8-byte stub, 27-trit address, HBI/HBP offset and
  256↔1024 Host8 Q-PRISM tuple identity.
- Introduce **one** bounded, reusable native room receiver with
  explicit EMPTY/LOAD/RUN/COLLECT/EJECT states and no new process per
  signal, keyed by original RoomStub and idempotency key.
- Three-seat MTP/Fischer/Fabric verifies the committed envelope
  before forwarding C-drive -> D-drive or equivalent distinct tiers.
- Compare 1/4/16/64 threads, 50-message cleanup batch and
  immediate scratch clearing, and hot/cold cache behavior.
  Report pulse rate, completed transactions/sec, p50/p99,
  write amplification and durable bytes separately.
- Keep training and persistent GNN model checkpoint memory
  separate from scratch GULP; link real, source-pinned forward
  inference only after independent held-out parity.
- Test interrupted transfer, duplicate input, dropped message,
  route collision, stale idempotency and archive replay.
- An application or external action remains subject to its
  original authorization and policy. Ordinary verified routing
  need not wait for per-message human approval.

## Status

This document reconciles source contracts and measured receipts.
It does not assert that the current P-NP pilot has mounted or
executed the operator's C: or D: drives, implemented a true
200-nanosecond disk completion, or trained the historical GNN
checkpoints. Those are the next separate acceptance gates.
