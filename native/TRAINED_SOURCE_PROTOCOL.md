# Historical trained inference source gate — HBI → HBP → SHA → SH → HASH

**Research gate:** P-NP Issue #28, Gate A, following exact-matrix certification PR #29.

The canonical source record is `policy/TRAINED_SOURCE_PINS.hbi`. It records **23** originally published assets at immutable Git commits and Git blob identities: **nine original trained model checkpoints**, three source/manifests groups, four original trained scoring planes, and the Shannon/HOOKWALL scoring stage. The public model corpus and privacy-restricted reverse-GNN sources are outside this finite inventory and retain explicit `HOLD_SOURCE` status.

## Source pins

| Reference | Commit |
|---|---|
| `JesseBrown1980/Asolaria-fnns-trained-and-reverse-gnns-many` | `c4b2458c88358c46155d3b308c594f9991720847` |
| `JesseBrown1980/Shannon-and-the-gnns-stage` | `32f9670a648dbbd02d64524e91150efeec8e4d2c` |
| Reference SHA implementation: `JesseBrown1980/qoled-internet-HEB-C-revisted-HTTP---free-0-hbi-hbp-sha-sh-hash/src/sha256.rs` | `06bcd697a011ebd2a4696220803a1a261f643d22` |

Original GSLGNN checkpoints have two manifest-declared SHA-256 pins:

- `gslgnn_w9_3_seq47.pt`: `6f82384ff2e313b1f34b48cca5c1c2bb3a8f9391bba54db7afbb36c5fda91ad5`
- `gslgnn_w9_3_seq47_v2.pt`: `9e50723b10c9de5c27baa5bc9386340bf72f76bcae2877064a47e4627a86602c`

The hosted workflow separately reads exact pinned Git source checkouts and computes SHA-256 over the *actual checkpoint bytes*, checking each value against its manifest. For other indexed files, the native receipt computes their SHA-256 from the exact Git-blob-pinned bytes; the verifier compares the result to its associated row.

## Canonical receipt traversal

1. **HBI**: source-pin index and output byte-offset index, with `seq/id/offset/len`.
2. **HBP**: canonical source record for each original file, carrying full Git commit, Git blob, exact SHA-256 digest, source identity and held trained-port status.
3. **SHA**: SHA-256 of every original byte stream, each canonical HBP row, and the HBP/HBI files.
4. **SH**: the lower-case 16-hex-character prefix of the full source SHA-256, used only as a compact routing fingerprint; the full digest remains authoritative.
5. **HASH**: ordered linked SHA-256 chain `hash_i = SHA256(prev_hash | "|" | receipt_sha_i)`, with `receipt_sha_i = SHA256(canonical HBP payload_i)`.

The terminal `PNP_TRAINED_SOURCE.anchor` carries the full HBI/HBP digests, frozen source-pin-index digest and final chain hash, with an internal anchor checksum. The unkeyed checksum establishes reproducible integrity **relative to the anchored Git revision / published archive**, not third-party identity or a signature. Source byte identity, evidence authenticity and neural-model equivalence are separate gates.

## Offline native replay

With both exact historical source checkouts available locally (for example as `source-train` and `source-stage`):

```text
native/target/release/pnp-native source-attest source-train source-stage evidence
native/target/release/pnp-native source-verify evidence
```

The builder generates `PNP_TRAINED_SOURCE.hbi`, `PNP_TRAINED_SOURCE.hbp` and `PNP_TRAINED_SOURCE.anchor` from original bytes. The validator recomputes SHA, SH, linked HASH, HBI offsets, source-pin identities, archive roots and chain terminal. It accepts no execution of original model code. The pinned Rust 1.81 compile is `--offline --locked`; original evidence checkouts are frozen by their exact full Git commits.

The hosted acceptance independently uses `sha256sum` for both pinned GSLGNN checkpoints and the finished HBI/HBP/anchor archive. It repeats generation byte-for-byte, rejects planted HBP, HBI, anchor and checkpoint corruption, and publishes every receipt for a second independent archive job.

## Transition to Gate D

The next engineering import is *exact original model semantics*. Each original trained `.pt` tensor must be decoded and quantized under a frozen, explicit integer/ternary rule, then tested against independently replayed source outputs on balanced and adversarial held-out data. All model inference ports remain `HOLD` until those acceptance checks pass. Training corpus and withheld reverse-GNN sources retain `HOLD_SOURCE`.
