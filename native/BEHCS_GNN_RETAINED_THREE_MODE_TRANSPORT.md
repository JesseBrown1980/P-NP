# Retained BEHCS word and trained-GNN files: Q-PRISM transport parity

**Date:** 2026-11-18/19 · **Status:** source-pinned executable integration · Rust 1.81

## Goal

This integration replaces the earlier deterministic expansion of the
original Q-PRISM 3,200-byte tuple with **retained, original public
artifact bytes**. It uses the same byte stream on three paths:

1. In-process native Q-PRISM 256→1024→256 transcode.
2. Single-shot localhost HTTP/1.1 binary exchange over TCP 127.0.0.1
   (a backend HTTP0-style room transport pilot; a threaded service, not
   separate cloud process or live cross-worker HTTP link).
3. GitRAM-style GitHub Actions artifact transport through twenty
   separately hosted sender and twenty receiver jobs, using the
   actual ten-bit symbols stored as little-endian u16 bytes.

All original file bytes are kept in a length-framed corpus, with original
source file labels and deterministic ordering. Each file's original Git
blob SHA is checked at runtime after pinned-commit checkout. Only
**public model checkpoints/manifests** are loaded. The private
training corpus and adversary-identifier reverse-gain sources are
excluded. Transport of a model checkpoint does not execute the trained
model.

## Source pins (14 entries)

Word-language source:
`JesseBrown1980/asolaria-behcs-256@818cc235a6db815e3b2591339cf9b42aad3e85a0`.

| Source file | Pinned Git blob |
|---|---|
| `data/behcs/codex/alphabet.json` | `8d4a3298576ce2c9d4a1683ffab667c8a743fc42` |
| `data/behcs/codex/catalogs.json` | `51af0b536c45e8e769066bfd886bf6f08daff75d` |
| `packages/deep-wave/src/cascade.ts` | `b9acdaff9b4537def53eca3d0fdb87a9c0dcab89` |
| `packages/deep-wave/src/third-cascade.ts` | `a3e4485ca4a60d1b12cf6ed398c2f886ba358ea7` |
| `tools/behcs/generate-falcon-glyph-language.js` | `2bf81fbc21a3851c4dbfcddeb227b0226e303836` |

Trained model source:
`JesseBrown1980/Asolaria-fnns-trained-and-reverse-gnns-many@c4b2458c88358c46155d3b308c594f9991720847`.

| Public file | Pinned Git blob |
|---|---|
| `models/gslgnn_w9_3_seq47_v2.pt` | `e3a9712b50f8e7b31590b0aa944e6a9ba6ea565b` |
| `models/gslgnn_w9_3_seq47.pt` | `6cddadc91ff4e6fa2bbfc315d90fa1bab0117c6b` |
| `models/prototype_model.pt` | `1d1be27874e5fa4a7ef88026561033a0e0aac3fe` |
| `models/contrastive_model.pt` | `76b7ec50d57e1a70cb9d755edffc75afc15e7a4b` |
| `models/baseline_model.pt` | `26caf5216a4df3a43250e40e1dd421180ca3c901` |
| `models/prototype_model_run2.pt` | `3660b469f648aa4287b9f6a19732543a22e72ccc` |
| `models/contrastive_model_run2.pt` | `6ab8743e0e3dca29de7387443a618272a2c0507f` |
| `manifests/gslgnn_w9_3_seq47_v2_manifest.json` | `f1eb3675ab5e8822d1511ce4b3062d51ee50b2dd` |
| `src/gsl_gnn.py` | `886b3b0c0cdbddba983fa8c3ae083c4520d38f0e` |

## Measured first local benchmark

[Run #44437161135](https://github.com/JesseBrown1980/P-NP/actions/runs/44437161135)
baseline job completed successfully and issued:

- Corpus bytes: **705,106 unique source bytes**, across fourteen files
- Serialized ten-bit representation: **1,128,170 bytes** per sender
- Original corpus SHA-256:
  `954fb2d16f694845408c2f5ef24220945138604416a7ddbfbf79ea16656693e7`
- Serialized Q-PRISM wire SHA-256:
  `8feb2f56d5062c7f887b3afa245253a232fd154fb5991f2cdae9781053cc7199`

| Same runner, one pass | Measured nanoseconds |
|---|---:|
| In-process native encoding | 2,412,460 |
| Native decoding plus independent 40-bit inverse | 5,740,241 |
| Localhost HTTP/1.1 transfer plus the same inverse checks | 7,158,382 |

These numbers are wall times of one hosted benchmark, not a latency
distribution or saturated throughput. The loopback measurement includes
starting the temporary server thread and exchanging the complete binary
payload; the native encode measurement is separate. For a fair throughput
comparison, repeat many trials and isolate per-mode startup, SHA,
serialization, and transfer costs.

## Hosted artifact transfer semantics

Every sender transcodes **the same complete 705,106-byte retained
source corpus**, resulting in twenty authenticated *replicas* and a
logical 22,563,400-byte serialised symbol stream across twenty senders.
Those twenty replications are **not 14.1 MB of distinct training data**.

Each receiver downloads only predecessor's source-bound binary and
one accepted 20-member root. It reconstructs the 705,106 original bytes
with both the existing native Q-PRISM kernel and an independently written
40-bit decoder; every file is covered by the corpus framing. The final
job compares the same source and wire digests across native, local HTTP,
and GitRAM. It also checks exactly twenty forward and twenty reverse
receipts and keeps separate wave roots.

The acceptor fails closed on a source Git commit/blob mismatch, wrong
binary length, invalid 10-bit glyph, nonzero padding, changed original
bytes, unexpected artifact roster, wrong leaf predecessor, bad root,
or altered reverse receipt.

### Remaining integration gates

- Map the retained models' **actual training-time feature schemas** to
  BEHCS word/graph features, load the original checkpoint weights and
  run genuine inference with independently checked prediction parity.
- Extend the local HTTP pilot into authenticated independent-process
  service and remote hosted native function endpoints; compare bounded
  queue and byte-verified handoffs on identical data.
- Benchmark repeated, warmed runs to compare throughput, p50/p95 latency,
  artifact upload/download overhead, and memory/GULP reclamation.
- Preserve source-private training data off GitHub and keep GNN scoring,
  word-language semantics, and Q-PRISM exact reconstruction as separate
  evidence planes.
