# Frozen Yang–Mills versus Rime Fischer: two-track competition

## Exact source and task parity

**SAT track:** Compare two code paths on the same 21 independently specified 3-CNF inputs, with the same 20,000 search-node ceiling. The incumbent, `sat/engine.py`, is copied byte-for-byte from the independently accepted Yang–Mills source at Git blob `0f67b6a32ad830188c1b8194ed212ef4950c3e7b`. The challenger, `sat/competition.py`, uses the same original-CNF identity, exact XOR recognition and opposite-pivot watcher, but orders free variables by original residual-clause influence with a Pohlig–Hellman-discovered coordinate as tie-breaker. It independently checks each digit target and output before search. The original solver is not edited.

Each arm runs in its own subprocess; order alternates A/B and B/A case-by-case. Charge both input generation and decision/count time, record wall clock including interpreter launch, per-process peak RSS, baseline's native metered work and challenger's own metered work, search nodes, witnesses, SHA proof chain, exact counts or explicit budget HOLD. Original A and new B use *different defined internal operation metrics*: those unit totals are NOT directly equated. Equal-node search comparisons, full wall time, memory, and exact correctness are the interpretable comparisons. Later hardware-matched repeated runs and confidence intervals should check noisy timing.

All 21 sources are rebuilt by the independent `sat/verify_heldout.py` oracle; a standalone competition referee rechecks original clauses, parity rank, solution counts, witnesses, source SHA and evidence fields for both contestants, then reconstructs the Rime prime-power digit receipts and rejects corrupted reports. Every different verdict or model count is a FAIL or HOLD until explained. Count of exact solutions and number of search nodes are separate from SAT-vs-UNSAT decisions.

## Distinct mathematics track

**Discrete-logarithm track:** The earlier Rime Bobby Fischer formulation is classical Pohlig–Hellman in a cyclic group: factor the exact group order into coprime prime powers; remove already-recovered digits; solve each order-q digit with Shanks baby-step/giant-step; reconstruct the exponent with exact CRT. This is not the later antiblunder CPL/routing scorer. The historical description is [Asolaria, Law 14, commit 1c03dca](https://github.com/JesseBrown1980/Asolaria/blob/1c03dca255cd374ff7e6db19790c2317c1eb2c85/laws/RIME-27-LAWS-operator-original-2026-07-23/STZ9.md). A searchable copy of its standalone `rime_fischer.py` implementation was not present at that path, so `algebra/rime_fischer.py` is an **independent new implementation from the documented mathematics**, not a byte-exact copy.

Benchmark this mathematical candidate on 21 known group inputs across primes 97, 193, 1,000,081, including prime-power towers and a factor 463. The independent referee solves those *same* targets using separate **full-order Shanks BSGS** and checks that both exponents reproduce the target modulo p; it also independently replays every digit and CRT modulus. These results are about discrete logarithms, NOT evidence of SAT polynomial complexity.

## Critical controls

- The PH-derived SAT ordering is a *heuristic adapter*. Applying modular coordinates to variables neither preserves a speed advantage nor gives a structural reduction automatically. Only measurements determine whether this adapter helps.
- The incumbent and challenger must return the exact same independently verified model counts whenever both complete. Any HOLD remains explicit; all failure cases remain in the artifact.
- The published archive must contain all 21 original DIMACS, all 42 contestant receipt files, full independent referee transcript, 21 math inputs/outputs, SHA256 manifest, scripts, and a separate successful downloader verification job.
- The general PHP family uses clauses wider than three literals; it belongs to its independent general-CNF proof and corruption gate, not this three-literal SAT competition. Equating performance there would require a frozen and independently verified common encoding.

## Acceptance and next iteration

Status: **OPEN until hosted checks**. Compare exact correctness, completed instances under the fixed node budget, node counts, launch+solve nanoseconds, and peak RSS. Publish win/tie/loss and counterexamples by family. Then preregister independent benchmark families and multiple repetitions with strong CDCL SAT and exact #SAT external controls. No improvement, regression, or reduction to general P=NP is presumed.
## Hosted acceptance and first measured result — 2026-11-17

[GitHub run #44198991467](https://github.com/JesseBrown1980/P-NP/actions/runs/44198991467) **PASS**: both paired SAT/referee and downloaded-archive jobs. Archive ID **14379359268**; published ZIP SHA-256 **a8bcc35fb68de96cfdf185049332d9875b6e55dfd36f93e6be39f7fece398b9b**. The independent referee reconstructed all original 21 inputs, all model counts, both source/proof hashes, Rime digit/CRT receipts, and rejected 11 forged competitor records. The independent full-order Shanks checker agreed with all 21 original Law-14 Pohlig–Hellman digit-lifting answers.

| Family | Frozen Yang–Mills search nodes | Rime-guided search nodes |
|---|---:|---:|
| XOR | 3 | 3 |
| Mixed | 9,287 | 9,139 |
| Planted | 29,361 | 29,639 |
| Random | 24,196 | 23,018 |
| Hub | 12,448 | 10,490 |
| Explicit UNSAT controls | 93 | 125 |
| **All 21** | **75,388** | **72,414** |

Both returned **20 independently verified exact #SAT counts and one explicit HOLD**, on planted n=30, counting the 20,000 budget nodes in both totals. Among **20 jointly completed** cases: Rime wins in branch nodes on **6**, incumbent on **7**, with **7 ties**. Both have correct exact counts when completed. The 11 corrupted competitor receipts were rejected.

Single-hosted-pass *total subprocess launch + execution time* was **7.542 seconds** for the incumbent versus **2.874 seconds** for the Rime candidate summed over all 21 cases (21 isolated processes each, alternating launch order). Median peak RSS was **18,636 KiB** incumbent and **18,724 KiB** Rime, with maxima **18,808 KiB** versus **18,780 KiB**. These are observed values, not calibrated speedup estimates. Importantly, the incumbent performs its extra fixed-order baseline search and associated witness checks; the challenger runs one order and has a different metric definition. Thus **the 7.54/2.87 elapsed ratio is not evidence that the PH algorithm itself speeds up SAT**. The Rime method's verified PH-digit work is actually extra computation for an experimental tie-break.

Next acceptance gate: repeated hardware-matched trials under controlled CPU frequency and fixed cost definitions, a shared single-order baseline for apples-to-apples kernel comparison, independent external SAT/#SAT solvers, and unseen SAT instance families. Preserve the original 21 development fixtures and the mixed wins, planted/UNSAT losses, and all time differences without rewriting them. P versus NP remains OPEN.