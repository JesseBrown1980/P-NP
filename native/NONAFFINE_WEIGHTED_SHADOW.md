# Non-affine weighted shadow discovery (2026-11-26)

Prior native shadow gate passed run 45878688559, head 15de24648c91ccf9b2e9b642dc73bed1021b99a5. Related SHADOW_RESIDUAL_PROJECTION.md and UNIFORM_NP_PROOF_TARGET.md. New research-only native test module: src/nonaffine_weighted_shadow.rs, using existing planner and FrozenMatrix. Native hosted verification pending; local Python audit executed, Rust compiler unavailable locally.

## Exact theorem and boundary

For any finite nonnegative integer factor family and internal group I, collect *all* incident factors. Define W(b)=sum_i product of incident factors(b,i), retaining full joint dependence on the boundary. Substitute W for the bucket and eliminate I. The total sum of products is unchanged by distributivity. A positive bucket row stores a positive internal choice; reverse the sequence to reconstruct one witness, then check original CNF. Unlike the prior affine-only commit rule, W can have arbitrary support and positive weights larger than one. For counts, multiplicity must be preserved. For decision, support suffices if subsequent operations are existential, but a witness still requires compatible joint choices. For fixed cap c on incident-union size, each admitted projection uses at most 2^c trial rows. This is exact bounded variable elimination, not a proof that every SAT instance admits an inexpensive sequence. A single witness is not an enumeration of all possible extensions.

## Executed blind local test

40 randomized DIMACS fixtures: ring of k=4,8,12,24,36 boundary variables, two internal selector variables per edge, four modes and two seeds. Source variables relabeled and literals/clauses shuffled. Modes: optional distinct selector targets; mixed three-of-four Boolean support; inconsistent equality ring with one inequality; optional identical selector targets (multiplicity three). Discovery uses only the source graph, selecting disjoint adjacent degree-three pairs with identical two external neighbors. Each bucket uses all incident source factors. Every group is independently re-enumerated in reverse union-address order, and positive witness checked. Independent oracle derives each edge's count matrix directly from generator semantics then multiplies the matrices and takes the trace. Original CNF ordinary min-degree elimination was also actually executed on every case. Eight smallest cases additionally verified with independent DPLL.

40/40 exact counts, 30 SAT witnesses checked against source CNF, 10 UNSAT by exact count zero. Largest 108 variables. 672 groups discovered, 446 weighted groups (at least one entry >1), 58 non-affine Boolean supports (three of four points), 481 rank-two 2x2 count matrices. 2688 boundary entries, 2294 positive witness rows; 2688 signed-difference triple arithmetic reconstructions and same-sum negative comparisons. Local triple arithmetic checks are not the native matrix API; native test separately uses FrozenMatrix for joint count control.

| Operation | Total |
|---|---:|
| Source graph edge insertion attempts |3476|
| Group projection rows |10752|
| Projection factor reads |51904|
| Independent verification rows |10752|
| Residual table evaluation rows |4976|
| Ordinary elimination rows (executed) |23752|
| Residual planner vertex scans |8720|
| Ordinary planner vertex scans |76464|

Projection plus residual =15728 table rows, 33.78% fewer than ordinary elimination under the same minimum-degree policy; independent re-enumeration brings the total to 26480, exceeding the ordinary baseline's solving rows. This is not a wall-time speedup; factor reads, storage, graph construction, scope scanning, bit costs and verification are additional, and the conventional baseline can also use group elimination. Maximum ordinary bucket width 4; residual width 3 after group projections (group union width 4). Native code has independent shuffles and per-clause initial factors rather than grouped source support, and no native performance claim.

## Joint-shadow negative control

Two count matrices A=[[2,1],[1,2]] and B=[[1,2],[2,1]] have identical row and column sums (3,3). Yet sum A_ij*A_ij=10 and sum A_ij*B_ij=8. Local marginals cannot certify global compatibility. Native test freezes these joint tables into three 27-cell rooms, reconstructs all occupied triples and checks both joins; it does not claim physical compression. Exact arithmetic check on the full joint tables is essential.

## Unstructured controls

12 independently seeded random/planted 3-CNF fixtures (8,12,16 variables) and four pigeonhole instances (6,9,12,16 variables), all independently counted by DPLL. Two admitted at cap six and both agree; 14 HOLD, including satisfiable and unsatisfiable cases. None has the selected twin-group structure. No assertion of UNSAT from HOLD. No universal width bound established; the new recognizer does not solve the general residual proof obligation.

## Evidence

Local script SHA-256 5bc1ff0a062238c36b0b4ad62a1da437f7de171d3891ef6e0d01757dfc615183; output 7d83e368b434b985a869c0ab3032438a032f95fa741bc05179c9f0cbc1dda0f2. Dependencies pnp_uniform_gate_audit.py 85021d284c00f30ffd83ca344737e259a94942f0c552e0a9b82f861264a54f42; pnp_residual_affine_audit.py be05e6c0eb2ba8b07695e6f5b3b60f1ed5ed45fa2b31936bd56bddfeff88f31d; pnp_blind_cnf_audit.py 1f2b37cac2418c577949777148041a3b945b96b5c85dff930eea7fce42f7e42a. Evidence zip SHA-256 2ddef8931c306601bc6fe722ea6eee5cfe54b6a220b6c02ab39fada45d7bfefc. Main unchanged.

Next: investigate compact exact representations of joint weighted boundary functions without requiring all boundary rows to be enumerated; benchmark against optimized decision diagrams and conventional model counters. Include unstructured residual controls and charge representation discovery, compilation, evaluation and reconstruction. Exact equivalence must hold for all boundary assignments. Universal polynomial representation plus efficient evaluation is still unproved. No P vs NP resolution.
