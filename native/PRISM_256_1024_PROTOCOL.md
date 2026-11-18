# Gate E-preflight — original Q-PRISM 256↔1024 reversible rung

This is the first exact COMB/PRISM representation-level acceptance gate following the [GAC, neural fabric and archival policy gate](FABRIC_GAC_GC_PROTOCOL.md). The canonical implementation reference is [Q-PRISM Host-8 `qprism_cube_host8.rs`](https://github.com/JesseBrown1980/Q-PRISM-human-organoid-neural-stream-as-a-high-dimensional-control/blob/b4a21a25381d6920b7aab92c94c3c3e1420fc9e4/host8/qprism_cube_host8.rs) at Git commit `b4a21a25381d6920b7aab92c94c3c3e1420fc9e4` and Git blob `992ffd793f6c98f4f03b33ff0d1154b52d0174b0`.

## Exact 40-bit representation equivalence

The original algorithm treats each byte as one base-256 digit and every symbol as a base-1024 digit, preserving bit order and the underlying integer. The smallest exact aligned unit is **five bytes = 40 bits = four 10-bit symbols**.

- `encode(bytes)` reads bits in most-significant-first order and returns each 10-bit BEHCS-1024 digit. For partial trailing blocks, it pads only the lowest unused bits of the final symbol with zero.
- `decode(glyphs,nbytes)` reverses the stream and requires an exact glyph count, every glyph at most 1023, canonical zero padding, and an explicitly known byte length. A malformed representation produces a HOLD/error rather than silently truncating a different source.
- The native code tests every possible 2-byte source (65,536 possibilities), each length from 0 through 101 bytes, source-known examples, and a 3,200-byte reference cube. The 3,200-byte cube creates exactly **2,560 symbols**, representing the original 25,600 bits with no added information.
- The original source's historical encoder and decoder are independently compiled as a separate Rust test target at the frozen Git commit. The accepted native runtime retains the same integer-only, crate-free Rust 1.81 rules as the rest of P-NP.

## Original reference results

The original 3,200-byte deterministic Q-PRISM test sequence `((i * 2654435761) modulo 2^32 >> 13) as u8` has the following independently recomputed digests:

- Source-byte SHA-256: `fe28e4a1813ea87aa55d9e4aad3e8ac028a2142890e39b3ab8e5eb40ba4ef05e`
- Little-endian `u16` glyph-stream SHA-256: `9593d266e75e5494172c62f4ceaebc2c4bd9a9cb9476f528f9d71543f313d4ab`
- `[1,2,3,4,5]` maps to `[4,32,193,5]` in 10-bit glyph values.
- `[255]` maps to `[1020]` with two zero padding bits.

The native acceptance uses these bytes and externally frozen original source evidence rather than an invented neural-output oracle.

## Evidence archive: HBI → HBP → SHA → SH → HASH

The original source is pinned in `policy/PRISM_SOURCE_PIN.hbi`, with its exact commit and Git blob. The native `prism-attest` command checks the checked-out source Git identity and its current source SHA-256. It emits **15 case HBP receipts**, HBI row offsets, byte lengths, full SHA-256 of each original input and its u16 glyph representation, 16-character SH prefixes, sequential SHA-256 linked HASH, and an anchor containing the full source/Git, HBP and HBI digests.

```text
native/target/release/pnp-native prism-attest source-prism evidence
native/target/release/pnp-native prism-verify source-prism evidence
```

Artifacts: `PNP_PRISM.hbp`, `PNP_PRISM.hbi`, `PNP_PRISM.anchor`. Both hosted certification jobs check the published outputs. Deliberately altered original source, HBI, HBP and anchor must be rejected. Repeated generation must be byte-for-byte identical.

## Scope and next step

This gate certifies reversible matrix *address/representation translation* and its provenance. A 10-bit symbol may address an alphabet/glyph without constituting an executed agent. The 60D named-axis, GAC authorization and full BEHCS glyph-semantics parity remain separate.

The next acceptance slice is **non-destructive GC archive transfer with crash/restart replay**. The archive must preserve complete original bytes and SHA/HBI/HBP identities, keep GULP mistake/genius feedback addressable, and reject overwritten or incomplete archives. It must remain operator-held for live rotation until a separate live authority gate passes. All trained GNN/FNN forward-parity work remains an independent HOLD.
