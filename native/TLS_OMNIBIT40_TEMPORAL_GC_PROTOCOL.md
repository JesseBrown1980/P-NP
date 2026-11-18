# Temporal TLS traces into the original 40 Omnibit operators, with post-GULP garbage collection

This is the next Asolaria experiment after [PR #43's stochastic held-out temporal-memory acceptance](TLS_STOCHASTIC_HELDOUT_PROTOCOL.md), the [original Omnibit 40-operator integration](OMNIBIT_40_OPERATOR_GATE.md), the [reversible binary–ternary floor bridge](BINARY_TERNARY_BRIDGE_PROTOCOL.md), and the original GULP/GC source controls. It directly consumes the **real archived event bytes** from the preceding PR, not a new generator or approximate replay.

Scientific source: Gosling and collaborators, *Probing the Memory of a Superconducting Qubit Environment*, [PRL DOI 10.1103/j1r2-5ph7](https://journals.aps.org/prl/accepted/10.1103/j1r2-5ph7). The present experiment uses **synthetic classical** event traces, not measured qubit jumps, observed TLS lifetimes, recovered quantum coherence, or a fit to device data.

## Source and accepted event population

Before performing any candidate selection, the native gate re-verifies:
- The earlier 196-record classical TLS state-pair archive and source-bound DOI fixture.
- The accepted 117-record stochastic TLS prediction archive, including complete fixed training, validation, held-out and shuffled input bytes, original source HBI/HBP root and exact trace SHA-256.
- The 20,710-record original Omnibit source-catalog archive and byte-for-byte provenance of the **8 RNQ, 12 TRI and 20 PI** reversible Rust functions, at original repository commit `dad4647ad19a1b08ee90990c34aab1255f9a6046`.

From each of the **nine** frozen scenarios (three source regimes × three seed specifications), two 8,000-event streams are selected: held-out and **event-count-preserving shuffled held-out**. This gives **18 distinct source streams, 144,000 raw event bytes**, with each source event equal to 0 or 1. Training and validation bytes stay in the original prior archive for audit but do not enter the compression comparison. Each of the 18 streams is encoded independently under three routes, for 54 separate source-comparable encoded frames.

## Three reversible routes, every original operator

All three use the already certified `−1/0/+1` trit alphabet, `binary_ternary_bridge::encode_route/decode_route`, and the untouched original `omega_operator_catalog::apply_symbols/invert_symbols`:

| Route | Floor representation | Source events per 20-symbol block | Blocks per 8,000-event stream |
|---|---|---:|---:|
| BINARY64 | 1 event as 1 signed trit in a canonical padded 64-value floor word | 20 | 400 |
| TERNARY256 | 5 signed event trits per 256-floor code; exactly 243 valid data codes | 100 | 80 |
| MIXED256+64 | 8 signed event trits as 256-floor five-trit + 64-floor three-trit banks | 160 | 50 |

All 40 source operator IDs are tried on **every** 20-symbol block, with both banks transformed under the same selector in mixed mode. No reserved floor pattern is ever promoted to a valid ternary data code: transformed symbols are inverted by the original source operator first, then validated by the reversible floor decoder.

Counts: `18 × (400 + 80 + 50) = 9,540` selected data blocks and **381,600 original-operator candidate evaluations**. Under the precommitted objective, the smallest complete stored block wins, with operator ID breaking ties. A compact, native exact `u16` literal/RLE codec chooses the shorter payload **separately for each transmitted bank**; every selected block stores the operator ID (one byte), each bank's coder mode (one byte), encoded byte length (two bytes), and the complete literal/RLE residual. Each stream frame stores scenario identity, seed, held-out/shuffled indicator, route, event count and block-payload byte length. Model-generated TLS hidden labels, source pseudo-random generator seeds and trained predictor weights are **not required** by the decoder.

The experiment reports:
- Selected stored bytes **per source stream and route**, with the exact original-operator identity-only baseline and within-pair held-out vs shuffled comparison.
- Totals over all 54 separately encoded outputs, number of selected nonidentity original operators and true stored byte costs, source event bytes, a one-bit-per-event reference, and the **full on-disk Rust executable size** charged once as a conservative self-contained decoder artifact.
- Actual hosted elapsed time (attest + source-bound reverify) and peak resident memory, from `/usr/bin/time`, retained separately from deterministic HBI/HBP receipts. This time includes full source provenance checks and is *not* a pure throughput comparison.
- A full decode of all 54 routes, reconstructing **exactly** the original 8,000 events per stream with the source generator and learned model state absent.

A raw 144,000-byte event dataset, a hypothetical 18,000-byte bit-packed reference and the three-fold, 54-stream coded archive are **different denominators**. Numerical savings must be compared within the same route and source-stream set, not by summing three encodings and claiming that sum is compression.

## Hosted, reproducible results — November 18, 2026

The accepted [PR-head hosted run #44322846648](https://github.com/JesseBrown1980/P-NP/actions/runs/44322846648) passed Rust 1.81/Clippy, source-prerequisite checks, full decoder comparisons, actual owned scratch reclamation and independent HBI/HBP evidence verification. The exact computed values printed by the frozen release CLI:

| Item | Source-bound result |
|---|---:|
| Source streams | 18 × 8,000 events |
| Independent route frames | 54 |
| Selected 20-symbol blocks | 9,540 |
| Original operator candidates evaluated | 381,600 |
| Nonidentity operators chosen | 8,266 of 9,540 (86.65%) |
| Actual selected encoded archive | **294,615 bytes** |
| Same-route, same-frame identity-only reference | **357,338 bytes** |
| Bytes saved versus identity-only | **62,723 (17.553%)** |
| All-route reconstructed event archive | 432,461 bytes including metadata |
| Full compiled P-NP native release executable | **1,449,920 bytes** |
| Selected archive plus full decoder charge | **1,744,535 bytes** |
| Identity-only archive plus same decoder | 1,807,258 bytes |
| Hypothetical minimal 1-bit-per-event storage for all 54 routes | **54,000 bytes** |
| Original distinct event scratch actually removed after GULP | **144,000 bytes** |

**Interpretation:** The original 40-operator selection beats the same codec's **identity-only** candidate on this fixed synthetic corpus. This represents a real, measured advantage in the selected family of reversible transforms and literals/RLE, **not** a compression advantage over bit-packed binary streams or a full-standalone executable benchmark. The complete P-NP binary is deliberately charged as the decoder, so its large fixed program cost is visible rather than hidden. The transcript separately records hosted wall time and resident memory in `PNP_TLS_OMNI40_RESOURCE.hbp`; those measures include upstream attestations and repeated verification.

## Physical GC after complete GULP acceptance

No live human operator approval occurs in any event or selection loop. The native `tls-omni40-gulp-discard` command performs an **actual owned scratch file deletion**, not merely a policy verdict:

1. Re-verify the accepted original PR #43 source evidence, all 40 original operators and the new HBI/HBP/anchor/encoded/restored archives.
2. Create exclusively `PNP_TLS_OMNI40_GULP_OWNED.tmp` under the destination, containing **144,000 actual projected held-out/shuffled event bytes**; write and sync it, read it back, and verify byte equality.
3. Independently decode the retained encoded archive and compare its reconstructed event archive with the already committed full decoder witness. Sync each selected, restored, HBI, HBP and anchor file.
4. **Remove the owned scratch file**, verify it no longer exists, and verify the immutable PR #43 producer HBP remains intact. Record the scratch bytes removed, selected archive digest, restored digest, source HBP digest and anchor digest in an independently hashed `PNP_TLS_OMNI40_GC.hbp` receipt, then sync that file.

This is genuine removal of owned disposable scratch **after** GULP. The certified PR #43 original source archive and every event distinction needed for exact reconstruction remain preserved. Atomic crash recovery through disk/fsync directory boundaries and concurrent-reader safety on **protected original exact-replay state** remain separate HOLD gates.

## HBI → HBP → SHA → SH → HASH acceptance

```text
pnp-native tls-omni40-attest source-omnibit evidence evidence
pnp-native tls-omni40-verify source-omnibit evidence evidence
pnp-native tls-omni40-decode evidence/PNP_TLS_OMNI40_SELECTED.bin restored.bin
pnp-native tls-omni40-gulp-discard source-omnibit evidence evidence
```

The source-pinned deterministic archive contains **72 new indexed receipts** (1 source, 54 stream/route outputs, 9 held-out/shuffle comparisons, 3 source regimes, 4 GC policy cases, 1 summary), each with original PR #43 and Omnibit SHA roots, explicit HBI byte offsets and lengths, SHA-256 row hash, SH prefix and linked HASH state. Root `PNP_TLS_OMNI40.anchor` binds HBI, HBP, exact selected archive, reconstructed event archive, and full program SHA.

Artifacts: `PNP_TLS_OMNI40.hbp`, `PNP_TLS_OMNI40.hbi`, `PNP_TLS_OMNI40.anchor`, `PNP_TLS_OMNI40_SELECTED.bin`, `PNP_TLS_OMNI40_RESTORED.bin`, and a separate source-rooted `PNP_TLS_OMNI40_GC.hbp` scratch deletion action receipt. The hosted acceptance job repeats source-bound attest and compares bytes, performs a **standalone original-source-free inverse**, removes the owned scratch and verifies the original producer files, plants corruptions in the source HBP plus each of five new core artifacts, and publishes an independent SHA-256 manifest checked by a second hosted job.

All native logic is Rust 1.81, strict Clippy, exact integer/trit arithmetic, no third-party crates, no embedded Python/Node/JSON execution, no floats and no human approval in the inner loop.

## Follow-on evidence and scientific boundary

This is a bounded comparison of **lossless representation and memory-accounting behavior** for synthetic event traces from PR #43. The full 40-operator search and RLE source codec are explicit, reproducible implementation choices. The original Omnibit October tournament scoreboard and its private Acer/Liris/Relic seat-local verdicts remain `NOT_SCORED` and `NONE`; a coded PRL-inspired event corpus is not a new original-seat tournament result.

A real compression advantage requires the per-route full decoder charge amortized over a clearly specified corpus, plus larger held-out traces, alternative source-compatible coders and independent timing. Device data, cross-seed predictive uncertainty, full trained multi-GNN forward parity and exact-source concurrent crash-safe GC remain independent scientific/engineering acceptance gates.
