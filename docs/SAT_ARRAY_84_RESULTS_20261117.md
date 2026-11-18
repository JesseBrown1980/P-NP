# P-NP 84-run SAT benchmark array — accepted 2026-11-17

**Research:** [merged PR #14](https://github.com/JesseBrown1980/P-NP/pull/14), source merge `c1953fd123db2c2bd7f30f7271d5aee1569c15a6`.
**Hosted acceptance:** [Actions run #44208774230](https://github.com/JesseBrown1980/P-NP/actions/runs/44208774230). Six competitor jobs, 84 original-CNF entries, clean-room mathematical referee and independently downloaded summary archive: **PASS**.
**Final independently verified result artifact:** GitHub artifact ID `14383774834`, ZIP SHA-256 `020f846d11f62fdc606029679221ee46ca559ca5f5ea467996f99d2aca0a7958`.

## Array of 14 frozen inputs

Seven unsatisfiable pigeonhole formulas PHP(3,2), PHP(4,3), PHP(5,4), PHP(6,5), PHP(8,7), PHP(12,11), and PHP(21,20); three satisfiable pigeonhole controls PHP(3,3), PHP(4,4), PHP(6,6); and four connected 3-regular, odd-charge Tseitin formulas with 16, 32, 64 and 128 vertices. Exact original DIMACS bytes and 14 per-case SHA-256 values are included in all six separate runner archives.

## Research-model exact SAT search-node results

| Case | Frozen Yang–Mills Fischer | New PH-guided Rime | Historical original-source Rime |
|---|---:|---:|---:|
| PHP(3,2) UNSAT | 3 | 3 | 3 |
| PHP(4,3) UNSAT | 17 | **15** | **15** |
| PHP(5,4) UNSAT | 103 | **61** | **61** |
| PHP(6,5) UNSAT | 749 | **399** | **399** |
| PHP(8,7) UNSAT | HOLD at 2,000 | HOLD at 2,000 | HOLD at 2,000 |
| PHP(12,11) UNSAT | HOLD at 2,000 | HOLD at 2,000 | HOLD at 2,000 |
| PHP(21,20) UNSAT | HOLD at 2,000 | HOLD at 2,000 | HOLD at 2,000 |
| PHP(3,3) SAT | 4 | 4 | 4 |
| PHP(4,4) SAT | **7** | 10 | 10 |
| PHP(6,6) SAT | **16** | 24 | 24 |
| Tseitin 16 UNSAT | 0 | 0 | 0 |
| Tseitin 32 UNSAT | 0 | 0 | 0 |
| Tseitin 64 UNSAT | 0 | 0 | 0 |
| Tseitin 128 UNSAT | 0 | 0 | 0 |

All three completed 11 of 14 inputs, with the same three budget HOLDs. Across the seven completed pigeonhole formulas, the Yang–Mills arm used 899 search nodes total and each PH-guided Rime arm used 516: 42.6% fewer search nodes. On the 14-case array if HOLDs are counted at their full 2,000-node cap, totals are 6,899 versus 6,516. New and original Rime generated identical SAT search paths and literal-check counts on every pigeonhole formula. Original historical PH often had lower measured Python overhead than the reconstructed PH, but timing was single-run and not controlled for variance.

Each Tseitin result used the common original watched XOR extraction, producing an exact rank-(V-1) contradiction without decision-tree search. The historical source's Pohlig–Hellman operation was authenticated and run separately and serves as the branch-order tie-break in the pigeonhole tests.

## External SAT solver result array

| Case group | MiniSat 2.2.1 | CaDiCaL 1.7.4 | PicoSAT 965 |
|---|---|---|---|
| PHP(3,2), PHP(4,3), PHP(5,4), PHP(6,5), PHP(8,7) | UNSAT 5/5 | **DRAT-certified UNSAT 5/5** | UNSAT 5/5 |
| PHP(12,11), PHP(21,20) | Time-limit HOLD 2/2 | Time-limit HOLD 2/2 | Time-limit HOLD 2/2 |
| PHP(3,3), PHP(4,4), PHP(6,6) | SAT 3/3 | SAT 3/3 | SAT 3/3 |
| Tseitin(16), (32), (64) | UNSAT 3/3 | **DRAT-certified UNSAT 3/3** | UNSAT 3/3 |
| Tseitin(128) | Time-limit HOLD | Time-limit HOLD | Time-limit HOLD |

Each external solver completed 11 cases, reached its 12-second wall limit on three, and had all reported SAT assignments checked against the original CNF. CaDiCaL published eight exact-input DRAT certificates, each independently accepted by a second source-pinned drat-trim rebuild. MiniSat and PicoSAT UNSAT claims were checked against independent mathematical truth of the original PHP/Tseitin cases, with no separate portable proof file produced by these programs.

## Acceptance and scope

- **84/84** original source/answer receipts accepted by the clean-room verifier, with source SHA identical across all six contenders per formula.
- **66/84** final SAT or UNSAT decisions; **18/84** bounded HOLDs. Each arm: 11 complete, three HOLD.
- **8/8** CaDiCaL independent completed UNSAT DRAT proof replays passed.
- **20/20** original-CNF forged SAT assignments rejected.
- Source identity: original Yang–Mills SAT engine Git blob `0f67b6a32ad830188c1b8194ed212ef4950c3e7b`; original historical ASOLARIA Fischer Git blob `cbdf20564f1b38f3ddd08062ead8174ecc3d236a`.

Research engines used an identical general-CNF DPLL decision kernel on PHP, differing in branch order; their limit was 2,000 search nodes and a 20-second process ceiling. External solvers used a 12-second per-case deadline. These different resource metrics are recorded explicitly. The source-frozen test array and result acceptance are reproducible with `.github/workflows/sat-array-84.yml` and `experiments/array/`.

**Next engineering target:** repeated ABBA runs with matched per-instance time budgets, resource-calibrated kernel search, unseen SAT/#SAT families and additional proof-checked CDCL comparators. P versus NP remains an open theorem question.