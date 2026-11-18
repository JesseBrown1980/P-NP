# Six-competitor hard SAT stress test — accepted November 17, 2026

**Original research source:** [PR #12](https://github.com/JesseBrown1980/P-NP/pull/12), merged to main at `baaa06df9c21439653b3995268a29152a35f1b1d`.
**Hosted acceptance:** [run #44206197812](https://github.com/JesseBrown1980/P-NP/actions/runs/44206197812): six competitor jobs, independent referee, and re-downloaded published evidence archive all PASS.
**Archive:** artifact ID `14384416253`, ZIP SHA-256 `9875ce90f86aca474418ffa52d25341dd7bbb8d9b7f7ae66adee089b3d74eabb`.

## Source-frozen instances

| Instance | Variables | Clauses | Mathematical source condition |
|---|---:|---:|---|
| PHP(21,20) | 420 | 4,221 | UNSAT by pigeonhole principle; 21 at-least-one clauses plus 4,200 hole-pair exclusions |
| PHP(8,7) | 56 | 204 | UNSAT by pigeonhole principle; separately proof-checkable test |
| 128-vertex random-matchings Tseitin | 192 | 512 | UNSAT by connected 3-regular graph with odd parity charge; exact rank-127 XOR contradiction |
| PHP(4,4) | 16 | 28 | SAT; original-CNF witness control |

The deterministic generator and original-CNF-independent referee reproduce the clause types and source SHA-256 for all six arms. The Tseitin input uses three random perfect matchings under fixed seed `20261117`; it is connected and degree three. Its spectral expansion was not independently established. Full raw CNF files are archived.

SHA-256 of original DIMACS bytes:

| Input | SHA-256 |
|---|---|
| PHP(21,20) | `b2f4062f20c75098afe491682145cc448ea8bb69e95083a31091e92aa05f3fbc` |
| PHP(8,7) | `b418a91aca6e6d8232fc5ab8e3a592f0f95fbc0b021c09f27cc83699b76487d1` |
| Tseitin(128) | `994eee83dbe437bcd6a27ab34405746d15fc3028af2b234aab0c04210c8f3fb0` |
| PHP(4,4) | `4a3b203dce0ccf991860ec881d8abfe5699b6a1fcd83f31c4d3cffa934fba7f4` |

## All six competitor outcomes

| Contender | PHP(21,20) | PHP(8,7) | Tseitin(128) | PHP(4,4) |
|---|---|---|---|---|
| Frozen Yang–Mills Fischer approach | HOLD, 2,000 nodes | HOLD, 2,000 nodes | UNSAT exact XOR, 0 search nodes | SAT, 7 nodes |
| New Rime PH/BSGS/CRT ordering | HOLD, 2,000 nodes | HOLD, 2,000 nodes | UNSAT exact XOR, 0 search nodes | SAT, 10 nodes |
| Byte-original ASOLARIA Rime Fischer ordering | HOLD, 2,000 nodes | HOLD, 2,000 nodes | UNSAT exact XOR, 0 search nodes | SAT, 10 nodes |
| MiniSat 2.2.1 | HOLD, 50 s | UNSAT reported, no independent solver proof | HOLD, 50 s | SAT witness verified |
| CaDiCaL 1.7.4 | HOLD, 50 s | **UNSAT: DRAT independently checked** | HOLD, 50 s | SAT witness verified |
| PicoSAT 965 | HOLD, 50 s | UNSAT reported, no independent solver proof | HOLD, 50 s | SAT witness verified |

Research engines have a common **2,000-search-node** budget for the general-CNF DPLL adapters. The external SAT decision solvers have a **50-second per-case** subprocess limit. These are deliberately different forms of resource limitation, so the table establishes outcomes under declared limits, **not an equal-budget or statistically validated speed comparison**.

On the Tseitin input, the native approaches share exact independently watched XOR recognition and Gauss elimination; this accounts for their zero-search certification. In particular, the historical Pohlig–Hellman source is executed as a verified *branch-order adapter* on general CNF; the original historical discrete-log program by itself does not solve SAT. The three-literal Yang–Mills engine remains byte-identical; on wide-clause PHP, its byte-identical branch-order score is used with the published shared general-CNF DPLL adapter.

The three external solvers each exhausted 50 seconds on both PHP(21,20) and Tseitin(128). Their PHP(8,7) UNSAT decisions agree; only CaDiCaL emitted a checked DRAT certificate for that specific input (470,751 proof bytes). Proof generation, checking, parser timing, native branch effort, and maximum resident memory are preserved in per-case receipts. CaDiCaL's timed-out partial proof may be large; no timed-out proof is represented as a completed certificate.

## Independent acceptance and corruption controls

The hosted independent referee verified six identical source hashes per case, source clause structure, the algebraic 128-vertex XOR contradiction, every reported SAT assignment against its original CNF, any claimed CaDiCaL UNSAT certificate with a separately built source-pinned `drat-trim`, and the complete evidence checksum chain. `100/100` actual adversarial flips were rejected: **75 forged assignments on original UNSAT CNFs**, and **25 attempts to apply the real, correct PHP(8,7) DRAT proof to the satisfiable PHP(4,4) input**. These are direct checks of the original archived bytes, not a general soundness theorem.

## Next experiments

1. Introduce one matched wall-clock/cpu and memory budget for all six arms, and compare common single-order kernels. Run repeated trials to estimate variability.
2. Expand to unseen published SAT/UNSAT and #SAT families with source identities and full certificate checks.
3. Add strong contemporary CDCL baselines, verify complete UNSAT certificates, and compare preprocessing, search, proof and memory costs separately.
4. Establish and validate any proposed SAT-to-discrete-log representation before promoting a historical Fischer group operation to an exact SAT reduction.

**Mathematical status:** individual input checks PASS; empirical limitations recorded; general polynomial-time SAT algorithm and P=NP remain OPEN.