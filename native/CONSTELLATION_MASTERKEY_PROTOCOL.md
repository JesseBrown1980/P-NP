# Constellation kernel + master table: original 256-level disk playback gate

**Pinned upstream:** \`JesseBrown1980/Quant-anything-0-loss@e979ebe114f32c5b6ab9c2325e252c4b8ce28ddd\`
([published manifest](https://github.com/JesseBrown1980/Quant-anything-0-loss/blob/e979ebe114f32c5b6ab9c2325e252c4b8ce28ddd/PUBLISHED-20261115.md),
[original Rust range-coder implementation](https://github.com/JesseBrown1980/Quant-anything-0-loss/blob/e979ebe114f32c5b6ab9c2325e252c4b8ce28ddd/rust/src/main.rs),
[original playbook](https://github.com/JesseBrown1980/Quant-anything-0-loss/blob/e979ebe114f32c5b6ab9c2325e252c4b8ce28ddd/kernel-and-key/PLAYBOOK.md)).

## Why these are three distinct inputs

| Original immutable input | Bytes | SHA-256 | Accepted role |
|---|---:|---|---|
| kernel-and-key/ASOLARIA-KERNEL-3174.bin | 3,174 | 202ff2f06af54788958a03f60245593f89a88b0a3a4fc76435ad8468f151dc98 | Constellation source-kernel description |
| kernel-and-key/ASOLARIA-KERNEL-3174.kernel-pump-copy.bin | 3,174 | f075f5c64d656ae3afa91c793ab2c74b5425e41de141d6b7dc5cf4605663b0ca | A distinct original kernel-pump description |
| kernel-and-key/ASOLARIA-CONSTELLATION-KEY-3174.bin | 3,174 | cce4f1e8ae407f108b5c122e764f0b4c3882ce059cf900a0cc1b2a0e7588c1ea | Constellation/frozen prior key |
| master-key/masterkey.xz | 549,112 | 567d937b8942aaaf5fc95a752ddf1ecc40a51031867834dfc4515467b7702f77 | Compressed order-2 byte probability function table |
| master-key/order.txt | 1 | d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35 | Required decoder order 2 |
| master-key/stream.bin | 1,711,250 | e3290c1052b0cbd7583bc89ca4af5c5f3eb7db081cc9ef46e1cfcf8ce6b29d1b | Original bitstream for first 4,194,304 enwik8 bytes |
| ggufs/gguf__axes__ASOLARIA-FOUR-AXES-RIME.gguf | 4,908 | 9fe92da2b064e39ad52b389fb92d0c59739e1fd7d9db5b82c6d25c0ebf8fa735 | Small genuine disk-frozen GGUF to validate identity and format |

The two kernels differ **by bytes and by SHA** despite sharing an old
3174-byte filename. The first is a textual generative reference, not
CPU machine code to execute. The Constellation prior key is separate
from the 549,112-byte compressed master function table. Both are
public published data, not private authorization secrets.

## The two distinct meanings of "read the 256"

1. **The existing native P-NP 256 floor:** a 256-symbol / eight-bit
   container for **five balanced ternary digits**. Of the 256 codes,
   243 are valid base-3 data words and 13 are reserved. Its pure
   integer radix transformation is already source-verified in
   \`native/src/binary_ternary_bridge.rs\`. No model table is needed to
   invert this *mathematical* representation.

2. **The original 256-outcome stream coder:** the published
   \`asolaria_codex dec\` consumes the original 33,554,432-byte
   decompressed order-2 table and the original 1,711,250-byte stream.
   Neither 3,174-byte Constellation description alone can invert
   that separately coded stream. The expected 4,194,304 decoded
   bytes have SHA-256
   \`5ba647fec2ba51b0383cbe7147e15a478d85613245b7131c41764dbdd5062f77\`.

Thus the Constellation/kernel/master-table loader is essential for
recovering the original **key-dependent coded data and learned
context**, while the local 256 floor alone is reversible without it.
The adapter must also distinguish source-dependent BEHCS-256 semantic
glyphs from their plain eight-bit ordinal representation.

## Executable first gate

The separate GitHub Actions
\`.github/workflows/constellation-masterkey-preflight.yml\` downloads
all seven original inputs and Rust decoder from the fixed upstream
Git commit, checks each published SHA-256 and length, and verifies
the small GGUF magic. It decompresses the original master table,
builds the original Rust 1.81 integer-only decoder, and decodes
4,194,304 original source bytes. It refuses a missing key, wrong
order, or any false positive from a truncated bitstream.

The job publishes only fingerprints and its decode receipt,
**never copied key data, model bodies, source stream, or raw corpus**.
The verified inputs remain immutable; temporary decoding scratch
is disposable after proof generation.

This is the **source and key acceptance prerequisite**, not yet a
claim that the stored GGUF can serve arbitrary Rime coordinates
through HTTP-0. Next, a native read-only GGUF reader must parse the
actual source tensor/metadata tables, bind them to the matching
generative kernel and exact source hash, and implement
\`HTTP-0 -> store address -> frozen model/key -> reconstructed cell
-> source witness\`. Path-1 direct store lookup, Path-2
sufficient-shadow recovery, and Path-3 n-dimensional witness have
separate required payload/capacity proofs. Do not use a generic
llama.cpp loader as an Asolaria-specific semantics oracle.

No GPU is required by this first acceptance gate. Whole-system
performance must charge original model-table load, disk/page-cache
read, decoder work, HTTP transport, and independent verification.
CPU-only fast address lookup should be compared against
an uncached source decoder and ordinary SAT baselines.

**Acceptance status: PENDING hosted run for this new gate.**
