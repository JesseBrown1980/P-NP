# Constellation CPU disk playback — source-pinned four-axes gate

**Scope:** The original November 15, 2026 four-axes Rime GGUF, pinned to
`JesseBrown1980/Quant-anything-0-loss@e979ebe114f32c5b6ab9c2325e252c4b8ce28ddd`.
The source is 4,908 bytes with SHA-256
`9fe92da2b064e39ad52b389fb92d0c59739e1fd7d9db5b82c6d25c0ebf8fa735`.
The accepted source generator is
[`programs/kernel-pump/four_axes_rime.py`](https://github.com/JesseBrown1980/Quant-anything-0-loss/blob/e979ebe114f32c5b6ab9c2325e252c4b8ce28ddd/programs/kernel-pump/four_axes_rime.py).

## What the native adapter now does

1. Admit exactly the published GGUF version, byte length, and SHA-256; parse its
   original metadata, architecture, Rime prime/root, and 24 tensor names.
2. Validate GGUF-v3 directory framing, data alignment, non-overlapping extents,
   and all 24 × 27 supported integer-valued binary32 slots. Source values are
   interpreted by integer bit operations to comply with the native integer-only
   policy. The reader does not use any third-party GGUF runtime.
3. Keep the accepted file as a **disk store**, opening and seeking to the four
   addressed bytes on each play. Each address combines tensor position and the
   original Fischer 27-glyph modular function.
4. Independently check 12 paired tensor transforms: the 27-term forward Rime
   transform matches the stored rimed coordinates; the inverse transform
   recovers the original stored coordinates. Together these are 324
   paired-coordinate checks, each evaluated in both directions.
5. Verify the whole-file SHA at admission and after the completed replay
   boundary, and reject changed bytes, lengths, coordinates, indices, framing
   and false Fischer addresses.
6. Send one bounded 11-byte request and receive a 15-byte response over real
   TCP loopback, comparing the returned coordinate against an independently
   performed source-file address read.

The three paths in this first gate are **direct store**, **forward Rime**, and
**inverse Rime**. They are source-bound computational views. They are distinct
from the RAINBOR NN/GNN/FNN models and from the orthogonal XY/YZ/ZX projection
witnesses. Those have separate acceptance requirements. The source-derived
Rime vector is an actual 27-coordinate frozen representation; passing this gate
does not assert a decoder for an arbitrary generated body.

## Execution

From the repository root, with the published file downloaded to disk:

```sh
native/target/release/pnp-native rime-disk-attest /path/to/gguf__axes__ASOLARIA-FOUR-AXES-RIME.gguf
```

The approved `.github/workflows/pnp-native-certification.yml` already
downloads and SHA-verifies the genuine artifact and the separate Constellation,
kernel-pump, and original master-table inputs. The new native acceptance runs
immediately after that source gate. All native tests, Rust 1.81 locked build,
strict Clippy, no extra crates and repository audit must also pass.

## Accepted boundaries and next work

- **Original Constellation description, kernel-pump description, 3,174-byte
  Constellation key, and 33,554,432-byte expanded master probability table**
  remain separately identified. Original byte-stream decoding is covered by
  [CONSTELLATION_MASTERKEY_PROTOCOL.md](CONSTELLATION_MASTERKEY_PROTOCOL.md).
  The GGUF tensor address reader cannot substitute for that stream decoder.
- The HTTP-0 pilot is one short loopback request in one process using an
  isolated worker thread, not a long-lived multi-process service or concurrent
  scheduler. Transport framing and source identity require an independent
  receiving-process verifier before production use.
- Original source files are never overwritten or emitted to receipts. Temporary
  scratch may be collected after the source-witness boundary. Immutable
  source fingerprint remains committed.
- The **Pump** explores possible candidates and addresses outward. The
  **Funnel** compares and retains successful reusable evidence inward,
  including source-specific training material when justified. **Freeze**
  commits verified state and source identity; **play** reads immutable
  addresses. Next generations are committed only after the witness accepts.
- Full XY/YZ/ZX shadows have rank 19 on a 27-cell cube, so eight additional
  independent residual coordinates are required for a general exact inverse.
  Reuse the existing `freeze.rs` reversible column witness, and specify the
  exact eight-coordinate complement before claiming three-shadow recovery.
- Later benchmarks must compare identical held-out SAT/UNSAT work across direct
  CPU baseline, existing ternary/Fischer solver, frozen CPU Rime guidance,
  and Pump–Funnel scheduling. Independently check SAT assignments or UNSAT
  proofs. Account for freeze, source load, disk/page cache, transport, and
  witness time in end-to-end measurements.

**Status:** Implementation and CI acceptance gate added in a draft branch.
Production PASS requires the hosted workflow result and independent review.
