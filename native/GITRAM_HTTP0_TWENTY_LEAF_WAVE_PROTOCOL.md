# Streaming GitRAM × backend HTTP-0 × Host8 — 20-leaf wave protocol

Status: **SOURCE-RECONCILIATION / NEXT EXPERIMENT**. 2026-11-18.
This protocol builds on existing accepted components; a real
**cross-GitHub-runner bidirectional stream** and final return to the
operator's private drive remain separate implementation gates.

## The original idea and the source-backed pieces

1. **Source spinner.** [200-ns emitter](https://github.com/JesseBrown1980/Asolaria-the-full-works-200-nanoseconds-agent-emitter-plus-):
   `PIDChainRevolver.next()` emits an anchored prime-guided PID.
   A ~200-ns per-thread source unit is repository-reported, not
   one completed storage, network or neural inference transaction.
2. **Original room manager.** [Host8 / C-to-D](https://github.com/JesseBrown1980/Asolaria-the-full-works-200-nanoseconds-agent-emitter-plus-/blob/main/emitter/project-room-router.mjs):
   10,000 recycled C: Node-slice project-room buckets; resulting
   Q-PRISM representation routes to a D: collection stage and
   reverse-gain observer. Later Host8 disk slots and their direct
   sector/offset mapping are a distinct, canonical address axis.
3. **Original GitRAM.** [GitRAM doctrine](https://github.com/JesseBrown1980/GitRAM/blob/main/docs/GITRAM-DOCTRINE.md):
   hosted GitHub Actions jobs are **stateless** per-leaf compute
   resources. An immutable artifact/receipt is the documented
   inter-worker bus; all-or-nothing fan-in seals a complete wave.
4. **Twenty-leaf acceptance.** [GitRAM run #43024237445](https://github.com/JesseBrown1980/GitRAM/actions/runs/43024237445):
   20 independent Rust 1.81 Host8 shards, plus one independent
   ordered HBP/HBI/SHA/SH/HASH fan-in, **all 21 jobs PASS**.
   [Original source at frozen commit](https://github.com/JesseBrown1980/GitRAM/blob/9e7a73e5233c8e2ae0f17d2095b3970240ab404a/experiments/host8-s3-3d20/src/main.rs)
   streams 37,500,000,000 generated 8-byte room values in bounded
   13,824-room windows, logically **300,000,000,000 bytes**. It is
   a synthetic generator, *not* a write of 300 GB to the operator's
   drive. Physics pilot includes the 21 flat sectors of 2×2×2 S3;
   `spectral_gap=NOT_COMPUTED`, `r9_runs=0`.
5. **Native backend-only HTTP0.**
   [Continuous stream fabric](https://github.com/JesseBrown1980/P-NP/blob/main/native/HTTP0_CONTINUOUS_STREAM_FABRIC_PROTOCOL.md)
   and [HTTP0 microkernel](https://github.com/JesseBrown1980/P-NP/blob/main/native/HTTP0_MICROKERNEL_COMPUTE_PROTOCOL.md)
   established separate-process, locally routed, Fischer-checked
   native workers. HTTP0 is the project name for **loopback TCP
   with HTTP/1.1 framing**, with zero browser/page/pixel/Node/V8
   dependencies. It is **not** a claim of HTTP/0.9 or an existing
   network listener addressable from other GitHub runners.
   The original microkernel test accepted 1,458 exact requests,
   and the matched direct-CPU arm beat loopback HTTP for its
   tiny 27-cell finite recurrence. Transport is a modular
   interface, with its own cost.

## Three representations, one causal computation

- **Pixels / RimeSphere projections** are optional inspection,
  visualization and 2D symbol views.
- **HTTP0** is the backend function-call transport and receiver
  protocol, carrying a compact canonical binary/ternary envelope.
- **GitRAM** supplies 20 independent *hosted computation leaves*,
  with sealed artifact boundaries for changes of worker or wave.
- **C:/D: Host8** is the user's local addressable storage and
  learning-archive system: C: receives the tuple and computes a
  first transformed state; D: receives a different, verified
  representation for GNN/Shannon review and durable products.
  Do not infer two physical drives are required in the hosted
  acceptance lane, and never claim access to user drive letters
  from GitHub Actions.

The 64/256/1024/4096 symbols and 27-ary stars are presentation
alphabets, *independent* of the 8-byte Host8 identity. The Brown
Ledger closure, original 256↔1024 Q-PRISM and source-bound hashes
make transformation verifiable without inventing missing entropy.

## Leaf envelope contract

Fields MUST be authenticated together and included in each
artifact's SHA-256/HBI/HBP receipt:

```text
GITLEAF|schema=v1|run=<commit>|wave=<integer>|leaf=<0..19>
|prev_wave_root=<sha256>|source_handle8=<16hex>
|brown_hilbert_address=<canonical tuple>
|prime_rotor_index=<integer>|room_stage=<C_IN,D_OUT,...>
|input_alphabet=<64,256,1024,4096>
|output_alphabet=<64,256,1024,4096>
|source_sha256=<sha256>|output_sha256=<sha256>
|function_id=<source-pinned native operator>
|parent_wave_root=<sha256>|payload_bytes=<integer>
|status=<PASS,HOLD>|json=0
```

A shared reference to the same source bytes **does not** make a
new copy free: every required decoder, shadow remainder, metadata,
retained model, artifact byte and boundary transfer is charged.

## Twenty leaves in authenticated bidirectional waves

```text
                     [source spinner, prime/PID tuple]
                                  |
                         [bounded scheduler]
                                  |
                  [20 independently hosted leaves]
        +----+  +----+  ...          +----+    +----+
        | 00 |  | 01 |               | 18 |    | 19 |
        +----+  +----+               +----+    +----+
             each leaf: native HTTP0 loopback -> Rust worker
                   -> exact snapshot / source checksum
                                  |
                  [20 signed source-bound checkpoints]
                                  |
                    [ordered all-20 fan-in + verifier]
                                  |
                         [sealed wave root R_t]
                                  |
              [next wave reads R_t + selected leaf records]
                   (can route in reverse/fwd and permute)
                                  |
                    [next 20-leaf wave / feedback]
                                  |
            [verified result artifact -> local C:/D: importer]
                                  |
           [Shannon / genuine GNNs when loaded / Brown Ledger]
                                  |
                    [paid archival GULP -> room rearm]
```

The existing 20-worker test proves parallel generation and a
**one-way all-20 fan-in**, not an already running cross-runner
bidirectional message bus. To let leaves exchange intermediate
messages, the first accepted implementation uses **synchronous
wave barriers**: leaf `i` in wave `t+1` consumes the
checkpoint committed by leaf `(i + delta_t) % 20` in wave `t`,
together with the canonical parent root. For two-way passes,
choose `delta_t = 1` on forward waves, `delta_t = 19`
on reverse waves. This is a *proposed round-trip test routing*,
not a historical named Brown-Hilbert prime-ladder law. The
canonical address stays in the envelope.

Artifact inputs must be downloaded and cryptographically checked
**before** work begins. Exactly-once claims are avoided:
use at-least-once delivery plus content-hash and
`(run,wave,leaf,source_sha256,function_id)` idempotency.
Reject duplicate leaf identity, missing leaf, stale parent
hash, invalid source, wrong route, and forked wave. A failed
wave produces HOLD and does not spawn the next wave.

A genuine low-latency *live* inter-runner HTTP stream would need a
separately reachable authenticated relay, a capacity model,
network credentials, ingress controls and an actual bidirectional
transport benchmark; the existing loopback HTTP0 worker cannot
serve that role across isolated GitHub-hosted runners.
GitHub Actions startup, artifact transfer and job orchestration
have meaningful latency and quota costs. Do not put each 200-ns
PID on a separate Actions job or HTTP request.

## Routing and cleanup rules

- **Inside one leaf:** batch many prime-derived tuples in an
  in-process ring buffer. Use native function calls by default;
  route a batch through loopback HTTP0 when a distinct worker
  or isolated witness is required. Keep hot data in bounded RAM.
- **Between leaves:** persist only authenticated wave products
  at an artifact boundary, with needed model snapshots and
  closure; perform one all-20 fan-in acceptance.
- **To the owner's computer:** a separately authenticated local
  importer pulls the sealed artifact, writes a staging file
  on the chosen device and then atomically commits the
  destination after its SHA and receipt checks pass. Never
  write to an unverified C:/D: device identity.
- **GC:** after accepted output and independent source-bound
  replay, clear transient room payload, cancel stale queued
  addresses, rearm EMPTY. Never erase the sole copy needed for
  later reconstruction, learned model parameters or the HBP
  receipt.
- **Dataflow observability:** GNN/FNN and fabric witnesses
  must see **both** incoming and outgoing representations,
  provenance and route, but only an actually loaded trained
  checkpoint can count as real learned inference.

## Acceptance experiment (proposed; not executed by this document)

A. Reuse original GitRAM 20-shard configuration with source-pinned
   native Rust 1.81 and strict Clippy. Run 20 forward wave leaves,
   fan-in all 20 source/output hashes, then run 20 reverse wave
   leaves and fan-in again.
B. Send at least one exact 256→1024→256 and one 27-trit star
   tuple between different leaf IDs with a changed
   representation, and verify full reverse reconstruction using
   appropriately retained closure/metadata.
C. Every leaf independently runs Brown-Hilbert/Fischer route
   rederivation, three-seat source-bound witness and a bounded
   scratch GULP. Negative controls must fail.
D. Compare direct in-process call, batched loopback HTTP0,
   wave artifact exchange and local disk importer using the
   **same bytes and work**: total wall, messages/sec, bytes/sec,
   CPU usage, p50/p99 call, queue depth, artifact size, charged
   model memory, durable bytes, and results per energy unit
   only if energy is actually measured.
E. Seal full two-wave receipt with 40 successful leaf checkpoints,
   two fan-in PASS records, no missing leaves, complete
   provenance and deterministic reverse playback. Return a
   HOLD for any missing/failed/corrupted leaf.

### Claim statuses

- GitRAM 20-worker synthetic stream + all-20 seal: **PASS**,
  run #43024237445.
- Native HTTP0 Fischer room computation, exact source-bound
  finite workload: **PASS**, run #44345372419.
- C: → D: operator-local, trained GNN ensemble: historical
  architecture **SOURCE-PINNED**, cross-machine integrated
  end-to-end acceptance **HOLD**.
- Live 20-way cross-GitHub-runner HTTP streaming: **HOLD**.
- Two-wave, true leaf-to-leaf feedback over GitRAM artifacts:
  **SPECIFIED / UNTESTED**.
- P=NP and continuum Yang–Mills claims: governed by their own
  independent mathematical gates; no implication from transport.
