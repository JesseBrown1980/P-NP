# RIME wind/unwind SAT competition — 378 accepted observations

**Accepted 2026-11-17 (São Paulo time) / 2026-11-18 UTC.** [Source PR #20](https://github.com/JesseBrown1980/P-NP/pull/20) merged at `072936fa9c65bfdcbca03bea73acaf1b625cc0af`; [complete independent hosted run #44216855484](https://github.com/JesseBrown1980/P-NP/actions/runs/44216855484) fully green, including independent referee and re-downloaded SHA manifest check.

## Scope: 18 existing CNF problems × three representations × seven solvers

The existing 14-case SAT array, plus Bitcoin's genuine SHA256d genesis preimage cases with 4, 8, 32 and 288 unknown header bits, were each tested in (1) original CNF form, (2) RIME-wound signed-variable form, and (3) same-size randomized signed-variable gauge form. The RIME transform uses the archived ASOLARIA 27-point number-theoretic transform and Fischer spherical wind/unwind constants to choose a **reversible signed permutation within 27-variable blocks**. Random control matches the number of sign flips within every block. Each arm ran at a 12-second wall limit; the four research adapters also retained the declared 2,000-node cap. Winding is a CNF representation transform, not direct inversion of SHA256.

## Completed outcomes out of 18 (SAT or UNSAT, otherwise HOLD)

| Engine | Original | RIME wound | Matched random gauge |
|---|---:|---:|---:|
| Fourth structural HXF-1 | **14** | 11 | 11 |
| Yang–Mills Fischer ordering + common watched adapter | 9 | 9 | 9 |
| New Rime Fischer ordering + common watched adapter | 9 | 9 | 9 |
| Historical original-source Fischer ordering + common watched adapter | 9 | 9 | 9 |
| MiniSat | 13 | 12 | 13 |
| CaDiCaL | 13 | 13 | 13 |
| PicoSAT | 13 | 13 | 13 |

**All 378 observations accepted:** 89 SAT with checked original-CNF witnesses where applicable, 144 mathematical-class-consistent UNSAT decisions, 145 budget or wall-time HOLDs. Source hashes and inverse clauses verified on all 378. The independent referee reconstructed mappings without importing the winding implementation and rejected a deliberate tampered signed-map checksum. The published complete evidence archive is GitHub Actions artifact ID `14389575931`, ZIP SHA-256 `19637e16d9372da743aa3898deb2ad824b6a58918e923eb86fa07bc8de1e135a`.

## Detailed outcome

Four connected odd-charge Tseitin contradictions continued to be exactly certified by HXF-1's XOR recognizer with **zero search nodes** in each of three representations. By contrast, HXF-1's signed RIME and random transformations erased its direct positive-clause Hall recognizable syntax. For PHP(8,7), PHP(12,11), PHP(21,20), HXF-1 changed from zero-node Hall UNSAT to a 2,000-node HOLD under either gauge; smaller PHP instances finished using general DPLL with positive node counts. Both signed transforms therefore completed 11/18 compared with 14/18 original.

Across the three Fischer ordering adapters, completed counts stayed at 9/18 under each representation, although search-node counts changed. Both new and historical Rime produce the same SAT behavior under the shared watched kernel here. Under Bitcoin, no engine solved nonce32 or Merkle+nonce288 within 12 seconds in any representation. CaDiCaL and PicoSAT solved both nonce4 and nonce8 in all three forms. MiniSat solved nonce8 in plain and random gauge but reached wall-time HOLD with RIME-wound gauge.

**Conclusion for this particular signed-permutation version:** RIME winding did not produce a repeatable advantage over the matched random control. A repeated-time randomized-order test is needed before claiming stable performance effects. More importantly, HXF-1 should normalize signed constraints or preserve recognizable Hall structure rather than blindly winding literals.

## What's in the experiment, and what is not yet integrated

**Included:** 27-point RIME prism, Fischer smooth-sphere winding and unwinding, reversible signed-CNF relabeling, all seven existing SAT engine/adapters, independently checked inverse mappings, and all 18 earlier benchmark inputs.

**Not included:** the **N-nested infinite-agent system** or **MTP** (the original ASOLARIA forward/backward frozen order-two scoring model). The historical `players/rime_windunwind.py` measured Fischer wind/unwind and MTP separately, but the 378-run SAT contest intentionally isolated the RIME representation variable. Future experiment arms can compare: base solver; RIME only; nested-agent routing only; MTP scoring only; RIME+MTP; and combined RIME+MTP+N-agent routing. Keep identical SHA source, verified model/proof rules and matched budgets. The phrase 'infinite agents' requires a concrete finite execution and resource policy before a reproducible benchmark.

See [experiment implementation](../experiments/rime_sat/README.md), [ASOLARIA historical winding drill](https://github.com/JesseBrown1980/ASOLARIA-UNIVERSE-SIMULATOR-with-gravity-light-time-space-color-rime-winding-drills/blob/main/players/rime_windunwind.py), and [ASOLARIA historical measurement receipt](https://github.com/JesseBrown1980/ASOLARIA-UNIVERSE-SIMULATOR-with-gravity-light-time-space-color-rime-winding-drills/blob/main/receipts/operator/rime_windunwind.txt).

General P versus NP remains an open mathematical problem. No preimage complexity claim is established by finite timeouts.