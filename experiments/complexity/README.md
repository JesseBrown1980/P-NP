# Completion versus complexity: 22 SAT research and standard solvers

Follow-on to accepted [MTP × N-Nest × Fischer 270-run ablation](../../docs/MTP_NNEST_FISCHER_270_20261117.md). This benchmark separates the mathematical notion of polynomial worst-case complexity from an arbitrary 12-second demonstration deadline.

## Two evaluation tracks

**Completion track (7 original CNFs):** planted witnessed 3-SAT with 12,18,24 Boolean variables; complete graph 3-color K4 and K5 UNSAT (verified from the full graph and three-color constraints); original PHP(4,3) and odd-charge Tseitin(16) UNSAT. All solvers run with **no per-case deadline and no search-node cutoff**. The GitHub Actions job has a finite 18-minute machine safety limit, so the execution remains bounded by available computing resources. This does not mean an abstract infinite computation is allowed.

**Scaling track (9 original CNFs):** planted witnessed 3-SAT with 36,54,81,121 variables; original PHP(8,7) UNSAT; and exact Bitcoin genesis SHA256d SAT at 4, 8, 32 and 288 free header bits. No SAT search-node cap is applied. This exploratory track uses a **30-second safeguard** on planted/PHP scaling formulas, **60 seconds** for 4- and 8-bit Bitcoin nonce problems, and **45 seconds** for 32-bit nonce / 288-bit Merkle-plus-nonce problems. Exhausting these safety budgets is reported as censored `HOLD_RESOURCE`, not algorithmic failure. All timing, preprocessing and search operation counts are retained.

## All 22 arms

Original seven: HXF-1 structural solver; original Yang–Mills Fischer ordering; new Rime Pohlig–Hellman ordering; byte-original historical ASOLARIA Fischer ordering; release-pinned MiniSat, CaDiCaL, PicoSAT.

Fifteen comparable CNF-native ablations: baseline, MTP only, Fischer only, N-Nest 3/9/27/81, MTP+N-Nest 3/9/27/81 and MTP+Fischer+N-Nest 3/9/27/81. The fifteen share identical HXF-1 Hall/XOR exact structural paths and the same watched-literal DPLL fallback. The original three Fischer arms also share that watched fallback, but HXF-1 uses its original own DPLL implementation. All research arms remove the earlier 2,000-node cutoff. The standard three CDCL SAT solvers use their installed original binaries.

## Correctness and fairness

Each arm receives the same source SHA-256 for each of 16 original CNFs. The separate clean-room referee checks the source clauses, independently establishes SAT existence for planted formulas from a source-derived assignment, reconstructs graph-color K4/K5 UNSAT constraints, checks the original mathematical PHP/Tseitin contradictions, and independently checks original-CNF witnesses for every SAT result. Bitcoin SAT witnesses must reproduce the public genesis double-SHA256 hash. Sixteen source-byte tamper controls are rejected by SHA checks. The complete archive is downloaded and independently SHA-256 checked again.

All 22 × 16 = **352 observations** are hosted in `.github/workflows/complexity-22-engines.yml`, with separate completion and scaling labels. Source and metrics are in `experiments/complexity/`.

### Complexity interpretation

Completed finite tests can show that a method is correct on a tested class and reveal scaling of measured operations. **P = NP would require a proof of a polynomial worst-case bound for every instance of an NP-complete problem**, not completion before any arbitrary clock deadline. No extrapolation from this finite suite establishes a universal polynomial algorithm. External resource limits and censored results are preserved explicitly.