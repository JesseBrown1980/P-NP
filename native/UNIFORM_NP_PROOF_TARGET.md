# Uniform NP computation — nonlinear relation discovery and stress boundary (2026-11-26)

Prior overlapping signed AMO full native acceptance PASS, run 45827821985 at 3846b6da8bbbf17e6456c1f8bac618393fc07896. This follow-up is a research gate, not a solution to P versus NP.

## Uniform proof obligation

A deterministic solver for every encoded SAT formula F with time C |F|^k, for fixed constants C,k independent of F and its length, implies P=NP by Cook–Levin. Finite tables with size dependent on a fixed physical cutoff do not prove this bound. A sufficient architecture theorem would give polynomial-time discovery of a polynomial-size, equisatisfiable finite-domain factor representation (bounded domain q), a polynomial-time computable elimination order of bucket width O(log |F|), and polynomial-time reconstruction. Then exact Boolean elimination takes polynomial time. Exact integer counting is a stronger target than Boolean decision; a polynomial exact model counter would imply FP=#P for #SAT, while P=NP alone does not immediately give a polynomial exact model counter.

The discovery stage for explicit full-support gate encodings of arity at most 4 is polynomial: group clauses by sorted support, record their forbidden truth rows, evaluate all at most 16 rows, and check each possible output for a unique value on every input. For fixed arity the truth-table tests are constant work per support; sorting and input processing are polynomial. A fixed library of signed/permuted gate signatures classifies AND, XOR, majority, multiplexer and OR. Multiple possible output directions are retained in local research replay. Every support table is an exact conjunction of the clauses on that support, regardless of recognition. Missing defining clauses must not be silently restored. This test does not infer compact, distributed, or implicit gate encodings in arbitrary CNF.

## Reduction showing the missing obligation

Given arbitrary 3-CNF F, introduce one output per clause for the OR of its three signed literals, then a binary AND tree over those outputs, and require the root to be true. Encode each gate by its complete forbidden truth rows. For each assignment of source variables, all outputs are uniquely determined in topological order. A full assignment satisfies the new CNF exactly when the source assignment satisfies F. The encoding has linear size in the number of clauses and preserves the source model count. Every gate is explicitly recognizable by the above algorithm. Nevertheless decision for this family is NP-complete because it contains a polynomial reduction from 3-SAT. Thus complete gate recognition alone cannot establish uniform polynomial SAT computation unless P=NP. This is a barrier for the proposed inference, not a proof that P!=NP or that every possible coordinate transformation must fail.

Exact forced-value propagation through the discovered tables can remove the AND tree and fix all clause-output variables to true, recovering the source 3-CNF. Each restriction preserves model count since the removed values are forced. Remaining SAT search must still be charged. If a compiler can efficiently reduce all such residual formulas to low-width or some other polynomially solvable language, it would meet the missing general proof obligation. A high width from a heuristic order only establishes a limitation of that order, not a lower bound on optimal width.

## Executed local audit (integer Python research code)

32 signed gate chain controls: 4 lengths (4,8,16,32 gates), 4 seeds, complete vs one missing truth row. Input DIMACS with randomized variable names and literal/clause order. Complete chains recognize all 240 gates; missing-row chains recognize 224 and retain each incomplete relation. All 32 exact counts matched independent two-state forward recurrence, and all 32 reconstructed witnesses satisfy original CNF. Max 85 variables and bucket width 4. Each family uses 5608 evaluated bucket entries; each has 19365 planner vertex scans. Complete family: 5796 literal reads, 8658 table/recognition probes; incomplete: 5738 reads, 8296 probes. Different operation categories are not combined into speedup ratios.

15 randomized 3-SAT sources, n=8,12,16,20,24 with 3 seeds each, exactly 4n distinct clauses. Each encoded into a circuit with 2m-1 explicit gates. Input to discovery/propagation is DIMACS alone. All 1905 gates recognized and all 1905 forced auxiliary variables removed by exact truth-table propagation; original source generator only used by independent DPLL counter. 15/15 recovered counts agree (13 SAT, 2 UNSAT). The source DPLL reference uses 517 recursive nodes in total; recovered DPLL uses 537, due to ordering differences. Both are exponential worst-case procedures. External system Z3 4.13.3.0: 45/45 satisfiability decisions agree, across original, encoded and recovered CNFs (39 SAT, 6 UNSAT). External solver confirms decisions, not counts. No timings claimed. Direct scope planner for the gate circuit exceeds cap 6 in every case, all 15 HOLD; after propagation reduced widths still exceed cap. All decisions and counts for these controls use conventional DPLL fallback, not a new polynomial algorithm.

| Source variables | Max gate-circuit greedy width | Max recovered greedy width |
|---:|---:|---:|
|8|14|8|
|12|18|10|
|16|20|13|
|20|25|15|
|24|30|18|

General cases: 42075 source literal reads, 57464 initial table/recognition probes, 173880 planner vertex scans, 306126 propagation probes; max encoded variables 215. These synthetic random instances are not representative benchmarks of arbitrary SAT. No uniform complexity or performance gain over a conventional solver follows.

Local source SHA-256 85021d284c00f30ffd83ca344737e259a94942f0c552e0a9b82f861264a54f42; output 9bbce5d29f89cda87d025c302de604ebb1574522ab501ec86e05111579fc12d4; external check source 07d4a91bb3940c9613ccea06a7bfa53cb586d41f5aa26780ebcc9ce62972f406; output 4a890e7cb3ce8f04d0efd86f1b56665d482d17f8e43e3e3a855959593778bcb8. Dependency pnp_blind_cnf_audit.py SHA 1f2b37cac2418c577949777148041a3b945b96b5c85dff930eea7fce42f7e42a. Native test [uniform_gate_boundary.rs](src/uniform_gate_boundary.rs) uses distinct deterministic fixture generation: 64 signed library gate recognition/missing-row pairs and 15 random circuit recovery cases; reference DPLL count comparisons. Native module does not reproduce the local chain planner metrics; hosted acceptance pending.

## Next proof-directed gate

Use a blind holdout of independently generated 3-CNF and unsatisfiable families, plus bounded-width positive controls. Investigate certificates for Boolean-specific reversible rewrites that reduce residual search width or provide a different efficiently decidable language; charge discovery, description, transformation, evaluation, fallback and witness verification. Include a conventional solver baseline and avoid interpreting heuristic HOLD as UNSAT. A polynomial-size representation alone is insufficient (the known circuit reduction above is already linear). A proof must address all inputs and show a fixed exponent, not merely extend a sequence of finite successes. Keep formal arithmetic lower-bound claims separate, as in VP_VNP_LOWER_BOUND_BRIDGE.md. No main merge authorization.
