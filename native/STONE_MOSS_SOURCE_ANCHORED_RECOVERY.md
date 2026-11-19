# Stone-to-Moss three-path source recovery: engineering specification

Status: integrated into draft PR #57. The hosted acceptance verdict must be read
from its run and exact commit. The program below moves original bytes, not an
invented synthetic checkpoint.

## Frozen original lineage

- Stone-to-Moss B3 scheme: every 512-byte stripe contains A (first 256 bytes),
  B (second 256 bytes), and P = A XOR B. Any two banks rebuild the entire
  original stripe; one bank is information-insufficient.
- Original BEHCS-256 word repository at commit
  818cc235a6db815e3b2591339cf9b42aad3e85a0; alphabet Git blob
  8d4a3298576ce2c9d4a1683ffab667c8a743fc42, second cascade Git blob
  b9acdaff9b4537def53eca3d0fdb87a9c0dcab89, third cascade Git blob
  a3e4485ca4a60d1b12cf6ed398c2f886ba358ea7.
- Original GNN model source at commit
  c4b2458c88358c46155d3b308c594f9991720847, original trained
  GSLGNN v2 checkpoint Git blob e3a9712b50f8e7b31590b0aa944e6a9ba6ea565b
  and manifest Git blob f1eb3675ab5e8822d1511ce4b3062d51ee50b2dd.
- Original source-word graph fixture: 432 words derived by the original
  BEHCS-256 256-glyph address algorithm, sampled from 93,312 named
  six-dimensional cascade event coordinates. The generator does not claim
  to recreate training labels for the original GNN.

## Executable architecture

Rust 1.81 native/src/bin/stone_moss_fabric.rs assembles six source records,
each with an exact UTF-8 name, big-endian byte length, SHA256 of original
content, and original content bytes. It then emits one of three independent
binary shards per GitHub worker. Every shard has the common archive SHA,
run identity, original length and lane, plus a SHA256 sidecar. Each worker
fetches and source-pins the complete original files before emitting only
its own assigned shard.

Research-side Python research/stone_moss_recovery_verify.py independently
rebuilds the six-record binary archive from separately fetched original
sources without importing the Rust program. It tests AB, AP and BP exact
recovery, compares the entire reconstructed archive and every source file
to original bytes, and checks the original Git commit and blob pins.
Rust performs a second set of three independent pair restorations and
byte comparisons. GitHub hosts three source-bank jobs and a fourth fan-in
job: .github/workflows/pnp-stone-moss-source-recovery.yml.

## Adversarial acceptance and information bounds

The referee must reject a single surviving bank (including a concrete
ambiguous-completion witness), duplicate lane, replayed run identity,
altered source SHA, truncated or bit-mutated bank, invalid record digest,
and uncommitted trailing archive bytes. It also constructs a coherent
alternate GNN checkpoint archive with its own correct checksums; it must
remain distinguishable from the separately Git-pinned original source.
This checks the Stone-to-Moss N-Nest common-source false-green boundary.

For each 512 bytes of original input, three lanes store a combined 768 bytes
of erasure payload, a 1.5x payload expansion before headers or manifests.
The workers locally read full source bytes in order to derive their
assigned 256-byte stripe halves. This bounded experiment uses downloaded
GitHub Actions artifacts rather than an always-on HTTP streaming fabric.

## Engineering versus dictionary

The historical stone, light bank, wave, rebirth, watcher, and chariot terms
refer here to byte archives, deterministic erasure stripes, independently
reproduced content, source provenance checks, and separate computation jobs.
The stored GNN checkpoint is genuinely the original source checkpoint; this
recovery experiment does not execute its trained model or establish a
word-to-trained-feature mapping. Those are measured in separate workflows.

Keep these flags distinct: input Git source pinned, six source records
verified, three pairs recoverable, external source authenticated, trained
inference executed, and human release authorized. The first three are
testable by this workflow; independent physical source custody, neural
measurements, optical persistence, quantum or gravitational measurements
and trained word accuracy do not follow from byte recovery or SHA digests.

No source repositories, historic Stone-to-Moss receipts, Hugging Face data,
or P-NP main files are mutated by the draft PR. The separate external Python
trained-inference harness lives outside native/ so the production native
directory remains Rust-only under mandatory CI.
