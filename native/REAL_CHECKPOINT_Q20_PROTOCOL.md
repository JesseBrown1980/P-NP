# Gate D — original nine checkpoint storage and integer Q20 import

This is a **real-source, byte-authenticated tensor-storage import gate** for the full multi-GNN ensemble. It follows [the ten-family model map](MULTIGNN_GATE_D_PROTOCOL.md) and the canonical **HBI → HBP → SHA → SH → HASH** source ledger.

## Source and coverage

The nine original `.pt` files are read from the pinned 2026 Asolaria trained model repository, commit `c4b2458c88358c46155d3b308c594f9991720847`, and checked individually against the full SHA-256 receipts established in Gate A. This covers L0 baseline, L1 prototype (three runs), L2 contrastive (three runs), and L4 GSLGNN v1 and v2 (two files). Identical historical checkpoints are still recorded as distinct original filenames when published under separate run names.

The parser treats the original `.pt` files as **inert archives**. It does not unpickle, execute or dynamically import any checkpoint class. It reads ZIP end/central/local records with strict checked byte offsets, verifies uncompressed record sizes and CRC-32, rejects path escapes, duplicates, split/unsupported archives, and records exact original bytes. Any unsupported archive format or invalid source digest fails closed.

## Integer-only conversion rule

When the inert `data.pkl` metadata is limited to the original `FloatStorage` class and all the archive's numbered tensor storage byte streams are multiples of four bytes, every little-endian IEEE-754 **binary32 storage word** can be read as an exact `u32` bit pattern and converted without a floating-point instruction:

1. Extract sign, exponent and significand using bit masks and integer shifts.
2. Scale the exact represented value by `2^20` to obtain signed fixed-point Q20.
3. Round to nearest integer, ties to even, entirely in integer arithmetic.
4. Reject NaN, infinities, Q20 overflows, unsupported mixed/ambiguous tensor storage types.
5. Hash domain-separated ordered Q20 `i64`-LE words together with their original storage path identities.

For any accepted finite weight, rounding error is at most **half a Q20 quantum** per coefficient. A valid ZIP with unrecognized tensor types is recorded `HOLD_DTYPE` rather than mislabeled as quantized. A valid archive still requires forward-inference acceptance.

The conversion also materializes one **native binary `PNP_Q20_<model-id>.bin` file per supported checkpoint**, with a domain-separated header, bounded storage count, per-storage canonical original path, explicit word count and consecutive signed `i64` Q20 little-endian values. `decode_q20_stream` reconstructs each original storage's integer array and rejects truncated, duplicate or malformed records. All nine original files in the hosted measurement have only recognized 32-bit floating storages: **115 storage records and 191,661 Q20 values** across the nine checkpoint files (including independently named historical run duplicates). Every converted byte file is SHA-256 checked, separately archived and subject to planted corruption controls.

The stored sequence is not yet a neural forward implementation: Python/PyTorch pickled tensor **names, shapes, strides, graph operators and activations** require separate source-authenticated decoding and independent reference-output parity before any `TRAINED_FORWARD_PASS` label.

## Verification commands

```text
native/target/release/pnp-native source-attest source-train source-stage evidence
native/target/release/pnp-native checkpoint-attest source-train/models evidence evidence
native/target/release/pnp-native checkpoint-verify source-train/models evidence evidence
```

Outputs: `PNP_CHECKPOINT.hbp`, `PNP_CHECKPOINT.hbi`, `PNP_CHECKPOINT.anchor`, and nine actual `PNP_Q20_*.bin` weight files. Each checkpoint row carries its original model identity, original SHA-256, Git blob, source-HBP root, exact ZIP and storage byte counts, conversion status, Q20 hash if valid, and a `forward_parity=HOLD` decision. HBI indexes exact row offsets and digests. SHA and 16-character SH prefixes feed the linked HASH chain. The anchor is an integrity commitment relative to pinned public Git sources and independently published archive digests, not a third-party signature.

Hosted Rust 1.81 CI verifies all nine original files, source and checkpoint hash identity, byte-exact reruns, planted original model/HBI/HBP/anchor tamper rejection, source-only static checks, compiled integer-only IR, zero-warning Clippy, and an independently uploaded archive replay. This stage does not require Python, Node, JS, V8, JSON execution or external Rust crates.

## Remaining trained-network parity

Gate D continues with exact inert tensor-key/shape extraction, per-operation integer execution for all four original neural model classes, source-frozen float-oracle outputs from independent research evidence, and balanced held-out confusion matrices. The original historical GSLGNN split has 315,209 positive and 40 negative examples; this imbalance is preserved in the scientific status labels.
