# Q-PRISM 20-hosted-worker source-pinned symbol transfer (2026-11-18)

## Purpose and evolution

This is the next test after the **27-cell Brown Ledger synthetic pilot**
and the **93,312-event typed-word wave**. It exercises an actual 256-bitstream
to **1024-alphabet** representation change before independently crossing the
GitHub Actions worker boundary. The receiving worker must recover the
original data and compare it to source-pinned originals, not just validate a
receipt string.

The source of truth is the original
`JesseBrown1980/Q-PRISM-human-organoid-neural-stream-as-a-high-dimensional-control`
commit `b4a21a25381d6920b7aab92c94c3c3e1420fc9e4`, file
`host8/qprism_cube_host8.rs`, Git blob
`992ffd793f6c98f4f03b33ff0d1154b52d0174b0`.

The source's **3,200-byte** reference tuple transcodes to **2,560 10-bit
symbols** with SHA-256 of input
`fe28e4a1813ea87aa55d9e4aad3e8ac028a2142890e39b3ab8e5eb40ba4ef05e`
and SHA-256 of little-endian 16-bit serialized symbols
`9593d266e75e5494172c62f4ceaebc2c4bd9a9cb9476f528f9d71543f313d4ab`.
These are immutable golden checks.

## Scale and physical payload

| Unit | Logical source | 1024 symbols | Physical LE-u16 wire bytes |
|---|---:|---:|---:|
| Per forward leaf | 1,024,000 bytes | 819,200 | 1,638,400 bytes |
| 20 separately hosted senders | 20,480,000 | 16,384,000 | 32,768,000 |

Each large payload is a deterministic test expansion combining the **exact
source-code file bytes** and the independently reproduced **3,200-byte
reference tuple** with a source-leaf rotation. The pilot does **not** treat
this expanded traffic as 20 MiB of independent empirical model weights,
trained corpus, or new observed sensor data. Its goal is physical bytes
through a representation transform and a genuine inter-worker artifact
boundary.

The forward stage writes a binary `symbol-NN.bin` artifact containing
**819,200 distinct 10-bit symbol positions** as little-endian 16-bit
words, *not* the original source bytes and not a hash-only surrogate.
Each leaf also writes `forward-NN.hbp` binding its original source
digest, serialized wire digest, leaf id, next destination, source commit,
source code blob, original cube digest, byte count, and symbol count.

Twenty GitHub-hosted workers run independently. A fan-in on a separate
host downloads the **actual 20 binary payloads**, independently decodes
and rederives the source, validates all member SHA-256 digests, and seals
a root with 20 unique canonical entries.

The next twenty independently hosted reverse workers each download only
**their specific predecessor's binary payload** and the accepted roster
root (leaf j consumes leaf (j+19)%20). Each receiver:
- authenticates the original source Git commit and blob;
- rechecks its symbol bytes and exact source-root membership;
- decodes using the native Q-PRISM 1024→256 kernel;
- separately performs the **independent 40-bit, four-symbol→five-byte**
  inverse implementation, using explicit 40-bit integer operations;
- compares both recovered byte arrays against the source-derived
  original, and emits an authenticated reverse receipt.

An independent final verifier checks the complete twenty-leaf roster,
all wire bytes, all twenty signed source reconstructions, and emits
`QP20_FINAL.hbp` with both roots. The test rejects omitted/extra/
reordered artifacts, foreign source, altered run, truncated or invalid
glyphs, invalid root, wrong predecessor, and changed raw content.
The Rust test harness also tests reference golden vectors, wrong
framing, invalid glyph codes, altered symbols and root-row tampering.

## Operational cost and claims

The per-leaf upload/download transport uses immutable GitHub Actions
artifacts. The source must be materialized and checked out at the
original Git commit in both encoder and decoder environments. Each
reverse worker downloads only 1.64 MiB of encoded payload instead of
all 32.77 MiB. Source/glyph hashes, root digests, and exact bytes are
acceptance criteria; timing receipts are separate noncanonical run
log measurements.

Run the dedicated
[Q-PRISM transport workflow](../../.github/workflows/pnp-qprism-hosted-transport.yml)
and archive its final receipt as `qp20-source-transport-final`.
Keep `P-NP` draft PR #57 and the mandatory Rust 1.81 strict policy
gate as independent acceptance requirements.

This is an **artifact-mediated, source-authenticated, tested
representation-and-transport path**. Live inter-runner direct HTTP
transport, real non-expanded sensor data, trained GNN inference and
actual C:/D: room materialization have separate verification gates.
