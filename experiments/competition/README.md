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