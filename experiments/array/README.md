# Repeatedly runnable SAT benchmark array

The original three research contenders and three external SAT solvers receive 14 exact original-CNF inputs each, 84 observations total. No pre-known SAT or UNSAT status is passed to a solver. The test sources are generated and SHA-256 checked on each of six independent runners.

| Family | Cases | Expected |
|---|---|---|
| Pigeonhole UNSAT | PHP(3,2), (4,3), (5,4), (6,5), (8,7), (12,11), (21,20) | Seven UNSAT |
| Pigeonhole SAT | PHP(3,3), (4,4), (6,6) | Three SAT |
| Tseitin on three-regular graphs | 16, 32, 64, 128 vertices, each one odd parity charge | Four UNSAT |

Research contenders: frozen Yang–Mills Fischer, new Rime Pohlig–Hellman ordering, and byte-identical historical ASOLARIA Fischer PH ordering. On wide PHP, all share the same exact general-CNF unit-propagating DPLL decision kernel with different original-source ordering. On Tseitin, all use the same original independently watched XOR extraction and rank contradiction; the historical group operation is also run separately but is not a SAT algorithm. The original Yang–Mills and ASOLARIA scripts remain unchanged.

External controls are release-pinned MiniSat, CaDiCaL, and PicoSAT. The three research arms stop at 2,000 search nodes or a 20-second process ceiling. External SAT search is limited to 12 seconds per case, and DRAT certificate checking receives a separate 12-second budget. These are explicit, nonidentical budget dimensions, so elapsed times are empirical comparisons and not standardized speedup claims.

Independent referee reconstructs all PHP clauses, verifies 3-regular connected Tseitin graphs and their odd-charge GF(2) contradiction, checks original SAT assignments, independently checks completed CaDiCaL DRAT proofs, confirms source SHA equality across all arms, compares new and historical PH branch-search behavior, and rejects 20 forged SAT witnesses.

**Workflow:** `.github/workflows/sat-array-84.yml`. Outputs include raw original CNF files, per-case solver stdout/stderr, status and timing receipts, solver proof material where generated, source SHA manifests, a complete 84-row independently checked result table and a separately re-downloaded SHA-checked summary.

Results are to be accepted only when both referee and final published-archive jobs succeed. The P versus NP hypothesis remains open.