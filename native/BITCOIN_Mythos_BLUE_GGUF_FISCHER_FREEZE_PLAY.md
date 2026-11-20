# Original Bitcoin SHA256d → GGUF v3 → Bobby Fischer freeze/play

Status: draft PR #57. Independent hosted Rust 1.81 and source-frozen
acceptance required before claiming an accepted GGUF playback result.

## Original source inspiration

The exact source to reuse is the 50,752-byte
`JesseBrown1980/Asolaria/MYTHOS-BLUE-STAR/ACER-MYTHOS-BLUE.gguf`
(SHA-256 `3dd92a42160a38c121980957ce9eabbea216678892801d7ea3ed4e99e96c3aae`).
Its 64×64×3 slice data and radial tensors were read in the original
`mythos-as-the-trits-themselves` Rust 1.81 research, passed through 27
parallel shifted trit copies and reversible gated movement, then recovered
exactly after 72 forward and 72 reverse steps. The standard GGUF v3 parser
in `src/gguf.rs` identifies magic, version, typed metadata, tensor
definitions and aligned storage.

The original RIME Law 15 in `Asolaria/laws/RIME-27-LAWS-operator-original-2026-07-23`
specifies freeze/slice/save and CPU-addressed Bobby Fischer playback.
The historical `rime_fischer.py` and the existing P-NP native Rust
`freeze.rs` plus `fischer_prime_tower.rs` contain the canonical
generator-7 modular address and verified prime-tower inverse.

## New standard GGUF archive

The new native encoder `native/src/bitcoin_gguf.rs` writes genuine GGUF
**v3**, two one-dimensional **GGML_TYPE_I8** tensors, typed string metadata,
canonical 32-byte alignment, and zero padding:

- `bitcoin.original_dimacs`: **all original full-SHA256d CNF source bytes**
  including clauses, unmodified and SHA-256 authenticated.
- `bitcoin.sat_trits`: one signed ternary byte for **every original SAT
  variable**: 255 means false (−1), 0 means unknown, 1 means true (+1).
  This is an actual search-state representation, not the known genesis nonce.

Metadata includes architecture, archive name, original-CNF SHA256, full
ternary-state SHA256, and the source-frozen Fischer playback scheme.
The Rust GGUF parser strictly authenticates both tensors, lengths, offsets,
metadata types, version, padding, original bytes and canonical re-encoding.

Full original source cases:

| Case | Original free bits | CNF variables | Clauses |
| --- | ---: | ---: | ---: |
| nonce32 | 32 | 239,112 | 877,879 |
| merkle_nonce288 | 288 | 361,188 | 1,325,357 |

## Actual Fischer playback

`native/src/bin/bitcoin_gguf_fischer.rs` supports:

`freeze <case> <original.cnf> <initial.gguf>`

The initial snapshot explicitly contains all-unknown ternary SAT states,
and includes the byte-pinned original CNF. This program has no external
runtime/model or GPU requirement.

`play <case> <original.cnf> <frozen.gguf> <receipt.hbp>`

Playback independently loads the GGUF file, verifies original source SHA
and parsed clause counts, replays addresses spanning 27-cell room boundaries
and distant variables using `fischer_address` and the original
`reverse_fischer_address`, and publishes metered SHA256 and HBP evidence.
The operator's original exact mod-p Fischer address `7^(27*room+cell)
mod 1,000,081` remains unchanged.

`branch <case> <original.cnf> <before.gguf> <after.gguf> <variable> <+|->`

Branch takes an unknown variable through its Fischer playback address,
sets one trial Boolean polarity, and freezes a **new** GGUF snapshot while
leaving the original archive unchanged. It refuses already assigned or
out-of-range variables and does not claim the branch satisfies the whole CNF.
The two hosted branches touch variable 28 (the second 27-cell room)
and the final variable in the original CNF.

The current freeze/play milestone is independently measured as
`solver_search=NOT_RUN`. The independent reviewer verifies the file's actual
GGUF layout and all source/snapshot bytes using separate Python standard
library logic, exact modular Fischer address witnesses, source-frozen
Bitcoin header SHA256d, eight corrupted-header controls, file manifests,
mutated-GGUF rejection, and separate archived readback.

## Follow-on coupled solver experiment

The existing P-NP `solve_with_matrix_guidance` consumes real live partial
assignments, hashes each 27-cell read, and performs exact CNF-guided branch
selection. The next link will initialize its search from a **loaded
GGUF snapshot**, so the source of truth becomes freeze → Fischer play →
original clauses and DPLL → changed live state → new GGUF freeze.

This coupled execution must explicitly charge GGUF file I/O and size,
original CNF decoding, Fischer addressing, state re-freezes, branch and
propagation counts, selector costs, and independent verifier cost against
matching direct SAT controls with an honest five-minute wall budget.

A verified lossless GGUF representation and exact Fischer playback establish
reversible computation. Any cryptographic preimage search improvement is an
additional measurement, requiring actual solver completion and original
full-SHA256d validation.

Workflow: `.github/workflows/pnp-bitcoin-fischer-frozen-gguf.yml`.
Independent referee: `research/bitcoin_gguf_fischer_referee.py`.
Original Mythos and RIME source projects remain byte-unchanged.
