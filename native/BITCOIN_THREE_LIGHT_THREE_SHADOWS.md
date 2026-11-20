# Three lights, three 2D shadows: exact 3D RimeSphere extraction

**Stage:** native-source implementation, awaiting Rust 1.81 and independently
verified original Bitcoin archive acceptance. Stacked on already accepted
RimeSphere streaming draft PR #58, independently of its branch; original
P-NP PR #57 and main are unchanged.

## Geometry: three distinct lights, not three duplicate views

Each signed 3×3×3 ternary room has 27 original trits. Project it with
three computationally orthogonal lights (along X, along Y, along Z) onto
three distinct **3×3 signed line-sum shadows**. These are line-integral
projections of signed occupancy, not binary silhouettes or a photographic
rendering. The three 2D planes retain different information.

A single shadow has nine signed pixels, each from three original cells.
All three have 27 pixel values, but only **19 independent linear
constraints**. An exact rational 27×27 rank calculation on the three
orthogonal projection operators gives rank 19, nullity 8. This is
reproduced in the independent Python referee using exact
`fractions.Fraction` Gaussian elimination.

So the actual reversible format includes **eight independent source trits**
from the cube with coordinates `x,y,z in {0,1}`. They provide the missing
information; the third planes are then used as independent consistency
checks. With these eight values, reconstruct the 19 remaining outer
face/edge/corner cells in increasing boundary order using the appropriate
shadow line sum. Verify **all 27 signed shadow pixels** against the
reconstructed cube afterward.

A concrete ambiguity test sets an alternating ±1 checkerboard in the
interior 2×2×2 cube and all other cells to zero: it has the **same three
shadows as the all-zero cube**, but different 3D data. The eight witnesses
distinguish the two exact sources. The Rust tests also enumerate all
`3^8 = 6,561` interior configurations with zero outer cells and check
1,024 deterministic pseudorandom complete 27-cell trit cubes.

## Original Bitcoin source flow

The hosted result from [streaming run #44652775916](https://github.com/JesseBrown1980/P-NP/actions/runs/44652775916)
has already passed for both original complete SHA256d CNFs and stores
the original source byte-for-byte:

| Case | Verified source bytes | Verified 27-cell rooms | Rime slices |
| --- | ---: | ---: | ---: |
| `nonce32` | 19,169,294 | 4,259,844 | 26,296 |
| `merkle_nonce288` | 29,552,992 | 6,567,332 | 40,540 |

These exact figures come from the independent accepted referee logs,
not estimates. The original SAT source fingerprints remain pinned:

- `nonce32`: `6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3`.
- `merkle_nonce288`: `601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b`.

The new `native/src/three_shadow.rs` consumes the original
`BITCOIN.rime` binary one 729-byte source gulp at a time. Each 27-cell
room becomes 35 exact signed bytes:

- 9 X-projected shadow values
- 9 Y-projected shadow values
- 9 Z-projected shadow values
- 8 retained source trits for the exact inverse

The `BITCOIN.three_shadows.bin` output is a room-aligned binary stream.
`BITCOIN.three_shadows.hbi` stores each original slice ID, original byte
offset/count and SHA256, parent RIME root, shadow payload SHA256 and a
source-bound rolling hash chain. Multiple distant rooms remain distinct
because the original (slice ID, room index, Fischer cell address) scope is
preserved.

`native/src/bin/bitcoin_three_shadows.rs` then replays **every room**
from the 35-byte representation and writes
`BITCOIN.three_shadows.replayed.cnf`. It must equal the original Bitcoin
CNF **byte for byte**, including every clause. It builds a new standard
GGUF v3 from the reconstructed original CNF and an all-unknown ternary
SAT root. The new GGUF must be **byte-identical** to the already accepted
`BITCOIN.rimesphere.gguf`, including its SHA256 and original-source tensor.

The independent Python referee `research/bitcoin_three_shadow_referee.py`
uses only the standard library. It separately reconstructs every 35-byte
room; checks the exact rational projector rank, every shadow pixel,
eight witnesses, all source bytes, HBI offsets and rolling roots; compares
the original accepted RIME stream; verifies the standard GGUF and all
zero SAT root; and runs malformed-record and incorrect-Bitcoin-header
negative controls. The Rust and Python readers share neither decoder
nor reconstruction implementation.

## Expected storage, runtime and interpretation

The raw 27 source trits are 27 signed bytes in the original RIME stream;
the three-shadow representation consumes **35 signed bytes per room**.
The approximate raw payload is therefore 29.6% larger than the 27-byte
room before per-slice receipts and indexes. This is a **reversible
measurement decomposition**, and no compression or preimage-search
speedup is assumed.

The original complete Bitcoin CNF remains the only input to SAT search.
Three shadows are a representation/verification layer that can potentially
supply distinct constraint views to Bobby Fischer's selection logic,
but that downstream experiment must compare measured original-CNF search
decisions, cost, and verification against matched baselines.

This stage has **no five-minute search timer**, because it is streaming
and reconstruction, not a SAT search. GitHub hosted jobs still have a
finite platform run limit. The existing GGUF DPLL can start from a verified
assignment; uninterrupted logical search across jobs requires persistence
of its **complete backtracking stack/frontier** in addition to the
GGUF root, and is a later acceptance gate.

Workflow: `.github/workflows/pnp-bitcoin-three-shadows.yml`.
It reuses the pinned successful full-Bitcoin RIME artifacts from
run #44652775916 to avoid redoing prior expensive processing and to
measure exactly what the three-shadow layer adds.
