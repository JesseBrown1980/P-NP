# Continuous causal stream vs fully framed bitpack with backend HTTP-0 fabric

## Original architecture and executable scope

The original Host8 design comprises addressable function rooms with eight-byte
state per room, native CPU execution and optional persistent disk rooms.
The historical C: and D: stores and private Asolaria recall corpus are
not prerequisites. Rooms are execution endpoints, not separate programs.
The actual transport in this gate is native HTTP-0 **in the backend only**:
loopback TCP with HTTP/1.1 wire framing. No page, HTML execution, browser,
JavaScript, Node, remote network or required physical disks.

Sources: [Host8 rooms](https://github.com/JesseBrown1980/asolaria-asi-os/blob/main/HOST8-ROOMS-GUIDE.md),
[Host8 integration](https://github.com/JesseBrown1980/asolaria-asi-os/blob/main/ROOM-MEMORY-INTEGRATION.md),
[original native HTTP-0](https://github.com/JesseBrown1980/qoled-internet-HEB-C-revisted-HTTP---free-0-hbi-hbp-sha-sh-hash),
[original HRM slow/fast adapter](https://github.com/JesseBrown1980/Asolaria-ASI-On-Metal-Fabric-and-matrix/blob/main/tools/falcon/omni-acer/lib/hrm-slow-fast.mjs)
and the already accepted P-NP native Fischer address function.

## Fair matched population

Freeze the exact earlier PR45 nine train-derived validation-selected models,
without retraining, held-out selection or new model parameters. Compare the
same 18 true held-out/count-preserving-shuffle streams (each 8,000 bits), in
three previously accepted route labels: **54 frames**, **432,000 event
occurrences**, **144,000 unique binary source-event bytes**.

Both arms include the same 8-byte magic, three-byte version/codec/model
header, two-byte frame count, 12-byte per-frame identifiers/count/length,
and 64-byte ASCII SHA-256 trailer. Bitpack stores no predictor. The
continuous causal arithmetic coder embeds exactly nine original frozen
model snapshots including two-byte length for each. One stream has one
causal context and one coder termination across all 8,000 events: this is
distinct from the preceding 20/100/160-event coder resets.

**Exact fully framed bitpack control: 54,725 bytes**, counted as
54 × (12 + 1000) + 8 + 3 + 2 + 64. Unframed bitpack 54,000 bytes stays
a separate reference. Direct arithmetic, Omnibit40's 294,615-byte
prior result, and PR45's 95,149-byte blockwise result use their
separate provenance. Charge full executable bytes once to each arm;
external HBI/HBP/anchor, SHA manifest, replay witness and retained
protected sources are itemized apart from codec bytes.

## Authentic Fischer routing and native microkernel state

For widths 3, 27 and 81, frame f is routed to lane f mod width. The
producer applies the existing Fischer modular address g^k mod p with
p=1,000,081 and g=7, k=27*floor(lane/27)+(lane mod 27). An
independent HTTP-0 service recomputes the address by a separately coded
repeated-multiplication algorithm and rejects mismatches.

Each addressed native room retains exactly one eight-byte working state
word across requests; the word is a dispatch checksum and is not
charged as a compressed archive. Measure the same 54 streams at every
routing width with the same reconstruction requirement. This tests
Fischer-addressed microkernel dispatch and transport overhead. Original
HRM slow/fast inference checkpoints are NOT LOADED by this gate; the
historical HRM and small codex integration is an identified next
experiment rather than an invented trained-model result.

## Independent third fabric witness

Seat 1: continuous encoder constructs the archive. Seat 2: the
previously accepted original arithmetic decoder verifies exact replay.
Seat 3: a **separately spawned Rust process** runs the independent
HTTP-0 function-room decoder, with its own snapshot parser, probability
logic, arithmetic coder and inverse. Seat 3 never receives original
event bytes, source PRNG seeds, the earlier entropy decoder or the
supposed target hash. It sends all 8,000 recovered bytes back via
HTTP-0, which the caller compares against original authenticated
events byte by byte. Both bitpack and range arms are tested at all
three widths: **324 independent process/message reconstruction calls**.

Corruption controls cover wrong snapshot version, noncanonical
payloads, framing changes and modified evidence. The existing
original-source SHA chain and protected-source retention policy are
preserved. Signed HBI/HBP/anchor plus independently rechecked published
SHA manifest are required. Rust 1.81 offline/locked tests and strict
Clippy must pass. **Hosted acceptance PASSED** in [GitHub run #44338318876](https://github.com/JesseBrown1980/P-NP/actions/runs/44338318876), including independent published-archive recheck. Precise measured bytes and route timings are recorded below.

## Native CLI

~~~text
pnp-native stream-fabric-attest evidence evidence
pnp-native stream-fabric-verify evidence evidence
pnp-native stream-fabric-inspect evidence/PNP_TLS_STREAM_BITPACK.bin
pnp-native stream-fabric-inspect evidence/PNP_TLS_STREAM_RANGE.bin
~~~

The stream gate creates PNP_TLS_STREAM_BITPACK.bin,
PNP_TLS_STREAM_RANGE.bin, PNP_TLS_STREAM_RESTORED.bin,
PNP_TLS_STREAM_FABRIC.hbp, PNP_TLS_STREAM_FABRIC.hbi and
PNP_TLS_STREAM_FABRIC.anchor, with resource measurements and an
SHA-256 manifest added by the hosted acceptance workflow.


## Accepted results — 2026-11-18, hosted run #44338318876

The same 54 frame inputs and the same 8,000 source events per frame were
reconstructed by the primary decoder and by every independently programmed
HTTP-0 room witness. All 103 native Rust unit tests passed, Rust 1.81
offline Clippy passed with zero warnings, source audit and compiled-integer
checks passed, and the separate evidence-recheck job passed.

| Actual payload | Stored bytes | Relative to fully framed bitpack |
|---|---:|---:|
| 54 × 8,000 bits, unframed bitpack reference | 54,000 | Reference only |
| Fully framed bitpack archive, with SHA trailer | 54,725 | Baseline |
| Full continuous causal range, including nine model snapshots | **54,200** | **−525 (−0.9593%)** |
| Previous PR45 blockwise entropy × Omnibit40 archive | 95,149 | Historical comparator |
| Previous PR44 Omnibit40 selected archive | 294,615 | Historical comparator |

Nine model snapshots occupy exactly **228 charged bytes** including
their per-snapshot lengths. Continuous coding is **40,949 B / 43.04%**
smaller than the PR45 blockwise archive, but it is **200 B larger
than unframed bitpacking**; we make separate claims for framed and
unframed references. All 54 continuous frames selected causal
arithmetic coding; the previous Omnibit40 operators remain accepted,
unchanged and measured separately.

The accepted new full native executable is **1,683,096 B**. When that
entire binary is charged once, continuous archive + binary totals
**1,737,296 B**, and framed bitpack + the identical binary totals
**1,737,821 B**. HBI/HBP/anchor, decoded witness and the protected
upstream source are separate costs.

| Fischer routing width | Independent HTTP-0 reconstruction messages | Measured group wall |
|---|---:|---:|
| 3 rooms | 108 | 22,286,116 ns |
| 27 rooms | 108 | 22,024,929 ns |
| 81 rooms | 108 | 22,414,411 ns |

Total **324 verified HTTP-0 messages** and **2,592,000 recovered
event occurrences**, exact by source comparison. This measures
client wire transport, native decoding, actual room dispatch and
the eight-byte room-state update together, not isolated pure-network
latency. The three measured timings are close; no performance ordering
across widths is established by one run. Fischer provides authenticated
address computation and routing, not data compression or trained HRM
inference in this gate.

Negative controls: mutated bitpack and continuous archive bodies
rejected; changed HBI witness rejected; corrupted model version
rejected inside the separate native worker; canonical replay and
regenerate-and-compare PASS; original source preserved. The earlier
PR45 owned post-GULP scratch discard also remained PASS.

**Scientific interpretation:** The positive 525-byte result is a
matched-frame payload result on this finite synthetic corpus. Model
overhead, compiled executable and retained evidence are reported
independently. Full trained HRM-to-codex pairing and standalone
transport-only ablation remain separate prospective experiments.
