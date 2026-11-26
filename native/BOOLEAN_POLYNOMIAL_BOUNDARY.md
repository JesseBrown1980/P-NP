# Boolean polynomial boundary — exact cardinality recognition (2026-11-26)

Related: VP_VNP_LOWER_BOUND_BRIDGE.md, CONNECTED_SLICE_SEPARATOR.md. Previous connected-separator native acceptance run 45519688416 passed. The later literature-documentation run 45826304532 was in progress when inspected. External VP-VNP claims remain separate from the finite experiments; no external Lean replay performed.

## Boolean theorem and formal counterexample

For binary x_1,...,x_n, product A = product_{i<j}(1-x_i*x_j) equals the indicator that sum x_i <= 1. B = product_i(1-x_i) + sum_i x_i product_{j!=i}(1-x_j) is the same indicator on the Boolean cube. Hence A=B modulo the Boolean relations x_i^2=x_i. They are not equal as formal polynomials for n>=3: the coefficient of x_1^(n-1)*x_2*...*x_n in A is (-1)^(n-1), from the unique star edge set, while B is multilinear. The coefficient remains nonzero over every field. At (2,2,2), A=-27 and B=5 over the integers. Exact local audit: 504 Boolean points for n=3..8, six coefficient witnesses; independent edge-subset enumeration for n<=5. The Boolean rewrite does not imply a small formal arithmetic representation of the permanent or Hamiltonian cycle. Cardinality recognition is standard SAT preprocessing; this is its integration into the existing affine slice pipeline, not an algorithmic novelty claim.

## Executed local blind-CNF control

The earlier synthetic parity families were reused (randomized original/auxiliary names, shuffled source clauses). After parity recognition and affine elimination, verify complete coverage of the n(n-1)/2 negative singleton-pair clauses. Replace exactly the pairwise exclusions with the candidate language consisting of zero and all n singleton assignments. Retain every other clause. Evaluate all n+1 candidates, reconstruct all original/auxiliary coordinates and check each candidate against the source CNF. Complete coverage proves all omitted assignments violate at least one original pair clause. Count is exact even when residual constraints cause UNSAT. Each free assignment has a unique full extension. If coverage is incomplete, decline this rewrite (not an UNSAT verdict).

40 cases: sizes 6,12,24,48,60 free variables, 2 parity families, 4 forbidden triple-pattern seeds. Largest original CNF has 120 variables. All 40 counts matched independent block-pattern oracle; 8 n=6 cases also checked via independent original CNF DPLL. All 1240 candidate membership checks matched original CNF. Of the 40 cases, 32 have complete-graph width >6 and were refused by the previous scope planner. These include the earlier eight width-control configurations at seed 0 (redundant bridge clauses omitted). All 10 near-clique controls (one missing edge, no residual block constraints) declined rewrite, and each missing-pair assignment satisfied original CNF while violating the proposed AMO. No arbitrary clique discovery or nonlinear basis search is claimed; whole free-domain AMO only.

| n | cases | candidate evaluations | edge-coverage probes | source literal reads | independent total count |
|---:|---:|---:|---:|---:|---:|
|6|8|56|120|848|30|
|12|8|104|528|2336|54|
|24|8|200|2208|7040|102|
|48|8|392|9024|23360|198|
|60|8|488|14160|34976|246|

Total 26040 edge probes, 68560 literal reads, 59458 GF(2) pivot tests, 3826 row XORs, 116800 pattern probes, 36320 residual clause tests if all evaluated. Source checks additionally scan original CNF (implementation short circuits). Costs are different operation types; no runtime or advantage over optimized cardinality-aware SAT/model counters asserted. For this explicit family the candidate count is n+1; edge recognition requires quadratic coverage and residual verification is polynomial. The native test uses the independent Rust fixture generator (different shuffles), tests exact counts, complete coverage and missing-pair witnesses. Native acceptance pending.

## Separate weighted arithmetic control

40 deterministic integer matrices (orders 2..6, eight seeds, entries -3..3): permanent permutation sum vs independent Ryser 40/40; directed Hamiltonian cycle anchored at vertex 0 permutation sum vs subset DP 40/40. Permanent and determinant agree modulo 2 in all 40; differ over Z in 39 and modulo 3 in 19. This is a finite numeric test, not a symbolic identity certificate for arbitrary polynomials. The formal characteristic-two equality follows from signs becoming 1 in the determinant expansion. For characteristic 3 the coefficient of x_01*x_10 differs by 2 in the 2x2 permanent and determinant. Neither finite weighted tests nor Boolean simplification contradict the reported arithmetic bounds. Weighted comparison executed locally in Python, not part of current native test.

## Evidence and next gate

Local integer-only research source SHA256 c2bad1e86f0b9e83ae816e4d774afca44b3fc3532ab7018002c7696d78fc9881, output 8c7cb824f613b5000fd9f25b0a83115d48a654a8e2bf34f5e80135c7f2b9e927. Dependencies connected separator audit 56a3f7ae955cb8c54576996b0e2b14fc34c63410476b83ff4454272a036f5930, blind CNF audit 1f2b37cac2418c577949777148041a3b945b96b5c85dff930eea7fce42f7e42a. Local compiler unavailable; native code registered in policy-approved src and hosted verification required. No change to main or production solver.

Next: independent held-out overlapping AMO/cardinality classes, mixed signs and missing edges; discover multiple local classes and certify each rewrite, use sequential counter or conditional separators for interactions. Compare with conventional cardinality-aware and XOR-aware solvers and model counters; explicitly charge source recognition, transformation, counting, validation and fallback. Separately benchmark arithmetic path/tree/grid witness families with formal coefficient identities. No P vs NP conclusion.
