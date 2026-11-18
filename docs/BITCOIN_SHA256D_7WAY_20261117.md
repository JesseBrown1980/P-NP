# Bitcoin genesis SHA256d SAT stress test — seven competitors

**Accepted on November 17, 2026.** [Merged PR #17](https://github.com/JesseBrown1980/P-NP/pull/17) at `990815f500758fd1bf250b50bb7e8976af4b67a1`; [final hosted run #44213748169](https://github.com/JesseBrown1980/P-NP/actions/runs/44213748169): all generator, seven competitor, independent-referee and independently downloaded archive checks PASS.

Original block header is the actual 80-byte Bitcoin genesis header: version 1, zero previous hash, public genesis Merkle root, timestamp 1231006505, nBits=0x1d00ffff, public reference nonce 2083236893. SHA256d digest in displayed byte order is `000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f`. Bitcoin Core source: https://github.com/bitcoin/bitcoin/blob/master/src/kernel/chainparams.cpp

## Four exact SHA256d preimage CNFs

Every source encodes the full two-pass SHA-256 circuit, including Bitcoin's actual 80-byte block header, padding, 32-bit modular additions, 64 SHA rounds per compression block and equality to the entire known 256-bit digest. Independent Python hashlib cross-checked the logic on the genesis reference and three other headers per source; the reference assignment satisfies every single emitted CNF clause. Only free source bits become existential variables, and no known nonce is explicitly supplied to the SAT solvers.

| Input | Free header bits | CNF variables | CNF clauses | Original source SHA-256 |
|---|---:|---:|---:|---|
| nonce4 | 4 | 236,344 | 867,665 | `204e9e08c5fb1a5f2555217c87bceb3385f9dc70405790015fd04393cb95d2aa` |
| nonce8 | 8 | 236,931 | 869,855 | `9968b60351e29037d9c7183f8c9b59a48f2da0be7b9c393d5013a503034ffa9a` |
| nonce32 | 32 | 239,112 | 877,879 | `6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3` |
| merkle_nonce288 | 288 (256 Merkle-root + 32 nonce) | 361,188 | 1,325,357 | `601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b` |

All four sources have at least one known mathematically valid solution, verified by direct SHA256d and complete CNF-evaluation independent checks. In the 288-bit case the Merkle-root field is unconstrained beyond the original header/hash; it is a Bitcoin-format header-hash preimage problem. Satisfying a 256-bit Merkle-root field does not by itself construct a matching transaction Merkle tree.

## Seven-competitor SAT outcome table

| Solver | nonce4 | nonce8 | nonce32 | merkle_nonce288 | Completed |
|---|---|---|---|---|---:|
| New fourth structural HXF-1 (original fallback) | 20s HOLD | 20s HOLD | 20s HOLD | 20s HOLD | 0/4 |
| Frozen Yang–Mills Fischer ordering + common watched-DPLL | SAT | 20s HOLD | 20s HOLD | 20s HOLD | 1/4 |
| Modern Rime PH ordering + common watched-DPLL | SAT | SAT (near 20s cap) | 20s HOLD | 20s HOLD | 2/4 |
| Byte-original historical ASOLARIA Fischer ordering + common watched-DPLL | SAT | 20s HOLD | 20s HOLD | 20s HOLD | 1/4 |
| MiniSat 2.2.1 | SAT | SAT | 12s HOLD | 12s HOLD | 2/4 |
| CaDiCaL 1.7.4 | SAT | SAT | 12s HOLD | 12s HOLD | 2/4 |
| PicoSAT 965 | SAT | SAT | 12s HOLD | 12s HOLD | 2/4 |

All successful SAT responses recovered the actual published genesis nonce **2083236893** and Merkle-root bytes `3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a`. Both their complete 80-byte header SHA256d and the original CNF model were checked before acceptance. Any UNSAT result would have failed the hosted test because reference SAT witnesses are known.

## Individual single-run wall-clock observations

These durations include solver launch, source loading and validation, and are not comparable as normalized algorithmic CPU costs. Research arms were given 20 seconds per source; external established SAT solvers 12 seconds.

| Solver | nonce4 elapsed | nonce8 elapsed |
|---|---:|---:|
| HXF-1 | 20.01s HOLD | 20.01s HOLD |
| Yang–Mills adapter | 5.05s SAT | 20.03s HOLD |
| New Rime adapter | 3.35s SAT | 19.54s SAT |
| Historical Rime adapter | 4.10s SAT | 20.01s HOLD |
| MiniSat | 1.16s SAT | 3.00s SAT |
| CaDiCaL | 0.67s SAT | 8.22s SAT |
| PicoSAT | 0.64s SAT | 6.90s SAT |

All seven arms were tested on exactly identical raw CNF bytes. Research Fischer arms retained their original ordering implementations, but their original tools do not directly process 240,000-variable SHA2 bitblasts; their branch-order results were therefore plugged into a *new shared exact watched-literal DPLL SAT kernel*, with identical unknown-input variable hints. The fourth HXF-1 was run as-is and fell back to its slower original general-CNF search. Thus the results identify an actionable engineering bottleneck but do not support equal-kernel or universal speedup conclusions.

## Independent acceptance

- **28/28** original-CNF/receipt checks passed; 4/4 source SHA-256 hashes and original SHA256d generator reference-model checks passed.
- **10 SAT solutions** with fully checked source models and Bitcoin header digests; **18 bounded time-limit HOLDs**.
- **20/20** deliberately altered nonce and Merkle-header hash claims rejected by independent SHA256d.
- **Final published archive**: GitHub Actions artifact ID `14387337879`, ZIP SHA-256 `663ee060c8070d4acb21fbb0eb829677950359133f272504f31c8539d5c73472`; independently downloaded and manifest-checked.

The reproducible tools are `experiments/bitcoin/genesis.py`, `watched.py`, `arm.py`, `runner.py`, `verify.py`, and `.github/workflows/bitcoin-genesis-7.yml`.

## Next research gate

Build a new fifth-generation structural engine that preserves Hall and GF(2) certificate recognition but upgrades the fallback with watched unit propagation and learned conflict clauses, retaining exact original-CNF witnesses. Evaluate on fresh blinded SHA256d and broader NP-complete constraint instances under matched limits, with repeated trials. Crypto preimage difficulty is not established by a finite SAT timeout, and the mathematical P versus NP question remains OPEN.