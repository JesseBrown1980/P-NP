# RIME wind/unwind SAT representation contest

This hosted experiment tests the original 14-case SAT-array and four exact Bitcoin genesis SHA256d SAT preimage CNFs using a **reversible representation transform** keyed by the archived ASOLARIA 27-point RIME prism and Fischer wind/unwind sphere.

## Design

Each original CNF generates three independently verified logical representations:

- **plain**: the unmodified original clause literals in canonical DIMACS.
- **rime_wound**: for each 27-variable block, source-SHA seeded bytes pass a 27-point NTT with original constants `p=1000081`, `g=7`, `w=951846`, then the original Fischer smooth-sphere winding `p=103681`, `N=20736`. Point/glyph order defines a variable permutation. The glyph parity defines exact reversible sign flips.
- **random_gauge**: seeded random within-block variable permutation and random sign flips using **exactly the same number of sign flips in every 27-variable block** as the RIME arm.

Every mapping is a signed bijection; every transformed clause is **exactly** mapped back to the original clause, in original order. Original CNF SAT witnesses are independently checked after inverse mapping. On Bitcoin, every successful witness is separately checked against public genesis double SHA-256. No source solves or expected answers are available to mapping construction.

This is a controlled first-stage test of **RIME-derived signed-variable relabeling**. It measures whether a reversible coordinate choice changes search complexity, separately from an equally local randomized gauge. It is not a new nonlinear SAT algebra or inversion of SHA-256.

## Run array

14 original SAT-array problems (7 UNSAT PHP, 3 SAT PHP, 4 odd-charge Tseitin) + 4 Bitcoin preimage SAT circuits (4-bit nonce, 8-bit nonce, full 32-bit nonce, 256-bit Merkle root + 32-bit nonce) = 18 inputs; × 3 representations × 7 solver arms (HXF-1, old Yang–Mills Fischer ordering, modern Rime, historical source-original Fischer, MiniSat, CaDiCaL, PicoSAT) = **378 bounded runs**.

The three Fischer research contenders use their source-pinned branch-order algorithms through the **same existing watched-literal DPLL adapter**, while HXF-1 uses its unchanged Hall/XOR/generic implementation. Every solver receives a 12-second per-case wall limit; the four research arms additionally retain 2,000 search-node limits. Source conversion and inverse validation times are recorded separately from timed solver execution. The 27-point transform is invertible and may be input-sensitive; no outcome is presumed.

GitHub hosted acceptance: `.github/workflows/rime-wind-sat-378.yml`. The independent verifier re-derives all 18×3 mappings without importing transform or solver source, checks the SHA of all transformed clause streams, checks every exact inverse clause, validates mathematical UNSAT classes and original-CNF SAT models, and validates Bitcoin preimage hashes. The complete evidence is re-downloaded and SHA-256 validated in a final hosted job.

## Historical sources

- [ASOLARIA 27-point RIME/UNRIME](https://github.com/JesseBrown1980/ASOLARIA-UNIVERSE-SIMULATOR-with-gravity-light-time-space-color-rime-winding-drills/blob/main/measurements/rime_unrime_three.py)
- [ASOLARIA Fischer wind/unwind drill](https://github.com/JesseBrown1980/ASOLARIA-UNIVERSE-SIMULATOR-with-gravity-light-time-space-color-rime-winding-drills/blob/main/players/rime_windunwind.py)
- [Original Fischer kernel](https://github.com/JesseBrown1980/P-NP/blob/main/historical/asolaria-2026-07-21/rime_fischer.py)

All original competitor source hashes are pinned in the workflow; the original files remain unmodified. No timing or speedup claim is made before fully hosted accepted receipts are available.