# Fischer HTTP-0 function-room computation and source-bound recall test

## Question and source architecture

The accepted PR46 HTTP-0 third witness verified 324 exact event
reconstructions using native function rooms and Fischer modular addresses.
This next experiment measures the rooms doing **actual finite computation**,
with and without native room recall. It uses backend-only HTTP-0, a separate
native worker OS process, and per-room persistent in-process state; no
browser, web page, C:/D: drive, GPU, Node or JS runtime is required.

The provenance references are the original Host8 eight-byte room law,
P-NP's native integer Fischer address, PR43 source-authenticated binary
heldout/shuffle traces, and the historical HRM slow/fast **shape stub**
from the Asolaria on-metal-fabric repository. A trained HRM checkpoint
is **not loaded**. Accordingly the last arm is named
HRM-*shape-proxy* routing, and the real trained-HRM predictive advantage
remains an independent HOLD.

## Matched workload and controls

Reconstruct the 18 original PR43 8,000-event heldout/shuffled traces,
then produce 54 deterministic jobs by pairing each authenticated stream
with each of the three already accepted route labels. The SHA-256
digest of each source event stream and route label determines its seed;
27 source-derived balanced-ternary coefficients are stored in each
job message. The source generator, original events and hoped-for answer
are not given to the HTTP-0 worker.

Each task executes a genuine finite 27-cell ring recurrence for
**243 ticks**, using strictly integer modulo 1,000,081:

    v0[i] = (seed + 13*(i+1)^2 + 17*trit[i]) mod p
    vt+1[i] = (3*vt[i] + 5*vt[i+1] + 7*vt[i-1]
               + 11*(trit[i] + t mod 3)) mod p

All additions and multiplications fit exactly in u64. A canonical
witness retains all 27 resulting field elements (216 bytes) per job.
The source-side reference and the separate worker calculate by
independently written algorithms, then compare **all 27 outputs**.
Three deterministic job orders (forward, reverse, rotate 17) give
162 requests per arm and guard against a favorable single stream order.

Arms (same 54 jobs, same three repetitions):
1. Direct native worker, no cache (162 computations)
2. Direct native worker with memo (54 computations, 108 hits)
3. Native HTTP-0 Fischer-addressed function rooms, no recall
4. HTTP-0 Fischer-addressed rooms with actual per-room result recall
5. HTTP-0 deterministic HRM-shape proxy / Fischer-addressed rooms,
   with the identical per-room recall implementation

For arms 3–5, perform separate comparisons at arity 3, 27 and 81:
**nine network arm/arity cells × 162 = 1,458** exact native
message/compute/replay operations. Each cell uses a new child process
and a new cache to prevent cross-arm contamination. Address keys are
derived from the original Fischer prime 1,000,081 and generator 7;
the remote service recomputes the modular address independently and
rejects a false address. For the HRM proxy, the predicted route is a
fixed deterministic function of the 27 trits and one of the eight
historical trajectory-shape labels, not a learned score.

Each function room keeps an eight-byte state checksum; cached final
results are separately stored with their exact source task key.
The 54-entry recall payload occupies 54 × (37-byte task key +
216-byte result) = **13,662 logical bytes**, excluding container,
allocator, HBI/HBP and process overhead. Direct memoization is a
mandatory control for attributing any gain specifically to routing.
No size advantage is claimed for addressing itself.

## Acceptance gates

- Rust 1.81, offline+locked, all-target Clippy -D warnings, no float,
  unsafe, V8, Node or JSON in the executable.
- 54 source-bound tasks and 13-row HBI/HBP/SHA/SH/HASH receipt.
- Every independent worker result equals all 27 source reference
  outputs; every task passes all three replay orders.
- Every no-cache HTTP-0 cell records 162 actual computations, zero
  cache hits; every recall cell records 54 actual computations and
  exactly 108 source-keyed cache hits.
- Actual Fischer modular addresses and independent remote route
  recalculations; wrong address and nonternary payload rejected.
- Per-arm elapsed wall values are output in hosted CLI logs, but
  excluded from the byte-deterministic HBI/HBP receipts.
- Corrupted output and receipt rejected, regenerated HBI/HBP and
  outputs byte-identical, original protected source retained,
  independent published-artifact checksum job succeeds.

Commands:

    pnp-native microkernel-compute-attest evidence evidence
    pnp-native microkernel-compute-verify evidence evidence
    pnp-native microkernel-compute-inspect evidence/PNP_MICROKERNEL_RESULTS.bin

Output files: PNP_MICROKERNEL_COMPUTE.hbp,
PNP_MICROKERNEL_COMPUTE.hbi,
PNP_MICROKERNEL_COMPUTE.anchor, PNP_MICROKERNEL_RESULTS.bin
and hosted PNP_COMPUTE_DIGESTS.sha256.

## Measurement boundaries

The room worker is independently programmed and runs in a separate
process, so each exact stencil result is genuinely recalculated or
looked up inside an HTTP-0 room. It is not a third-party audited
numerical theorem, a real trained HRM deployment or 20,000 concurrent
OS processes. Timing combines TCP loopback request, serialized
message, scheduler, cache and compute overhead. CPU/GPU/physical-drive
production extrapolations need additional measurements.

**Status:** hosted PASS in [run #44345372419](https://github.com/JesseBrown1980/P-NP/actions/runs/44345372419), both Rust 1.81/source-certification and independent published-artifact SHA jobs.


## Hosted results: 2026-11-18, run #44345372419

Both jobs passed; 105/105 native unit tests passed with Rust 1.81
offline/locked and strict zero-warning Clippy, and the evidence
manifest was independently rechecked in another hosted job.
The full native executable in this run was **1,777,944 bytes**.

The immutable workload is 54 jobs, 27 integer cells per job,
243 recurrence ticks per real calculation, and three orders:
162 requested outputs per arm. The separate HTTP-0 worker independently
calculated the outputs and the controller checked all 27 result
values for every call; invalid trits, wrong Fischer addresses,
archive mutation and HBI mutation were rejected.

| Arm | Room capacity | Actual visited rooms | Recalculations | Source-key recall hits | Wall (ns, 162 calls) |
|---|---:|---:|---:|---:|---:|
| Direct CPU (one process) | — | — | 162 | 0 | 2,027,005 |
| Direct CPU + memo | — | — | 54 | 108 | 701,254 |
| Fischer HTTP-0 | 3 | 3 | 162 | 0 | 14,440,866 |
| Fischer HTTP-0 + recall | 3 | 3 | 54 | 108 | 12,877,488 |
| Fischer + HRM-shape proxy + recall | 3 | 3 | 54 | 108 | 12,873,962 |
| Fischer HTTP-0 | 27 | 27 | 162 | 0 | 14,175,428 |
| Fischer HTTP-0 + recall | 27 | 27 | 54 | 108 | 13,336,697 |
| Fischer + HRM-shape proxy + recall | 27 | 26 | 54 | 108 | 13,505,271 |
| Fischer HTTP-0 | 81 | 54 | 162 | 0 | 15,582,872 |
| Fischer HTTP-0 + recall | 81 | 54 | 54 | 108 | 13,587,495 |
| Fischer + HRM-shape proxy + recall | 81 | 42 | 54 | 108 | 13,704,450 |

Nine actual HTTP-0 arm/arity cells × 162 = **1,458** independently
checked requests. Unoccupied rooms at capacity 81 are explicitly
reported: no assertion that all 81 physically execute work.

- The 54 cache entries contain **13,662 logical bytes** of
  task+result keys/payload, plus actual BTreeMap/heap overhead not
  hidden in that number. The 8-byte state word per visited room is
  separate. The same cache key and payload are used in both Fischer
  recall and deterministic HRM-shape proxy arms.
- For these 243-tick short tasks, direct CPU memoization is the
  fastest recorded configuration. HTTP-0 recall reduced actual
  computations 162 → 54 and showed small matched wall-time
  improvements over HTTP-0 with no recall (about 6–13% across widths).
  The HTTP-0 transport overhead outweighed computation saved
  compared with in-process baseline.
- The HRM-shape proxy has **no demonstrated consistent speed
  improvement** over plain Fischer room recall. Trained HRM weights
  were NOT_LOADED. Routing alone therefore cannot be credited
  with prediction improvement.
- Each room worker currently executes incoming HTTP-0 messages
  **sequentially**, not in a parallel multi-thread pool. This is a
  measured single-run latency comparison, not a scaling law or
  hardware-agnostic speed ranking. Next gates: batched transport,
  pooled concurrent CPU workers, cache-memory charging under
  sustained load, and truly trained HRM inference on held-out tasks.
