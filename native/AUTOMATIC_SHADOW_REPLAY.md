# Automatic shadow-address discovery and conditional component replay (2026-11-26)

Prior primitive-shadow native acceptance PASS: run 45883347051 at 113178b4b4fb08b59319c31caf3ab8c80f8eba71. Matrix Law Essentials PR #19 previous head also passed run 45883354643. New local integer-only audits below; native test awaits hosted acceptance.

## Exact conditional decomposition identity

For any Boolean residual CNF F on remaining variables V, partition its active clauses by connected components of the variable-clause incidence graph. Let the disjoint active variable sets be V_i, with residuals F_i, and let U be the variables absent from every residual clause. Then #F=2^|U| product_i #F_i. Conditioning on one active variable v gives #F=#(F|v=0)+#(F|v=1), with counts over the same remaining variables. Contradictory empty clauses contribute zero; an empty conjunction counts all free assignments. These identities follow from disjoint union of conditioned solutions and Cartesian product of independent components. Cache exact canonical residual functions or (within one fixed graph) induced remaining-vertex masks, with any free-variable scale separate. Never reuse a mask across differing source graphs without source identity. A positive branch or all positive independent components suffice for a witness; replay and check original clauses. This is standard exact component-caching / AND-OR search, expressed as conditional joint-shadow replay, not a priority claim or a new universal SAT complexity bound.

For the independent-set CNF (not x_u OR not x_v) per graph edge, the count recurrence is I(S)=I(S minus v)+I(S minus closed-neighborhood(v)), with I(empty)=1. An induced graph that becomes disconnected factors into component counts. Unlike an ordered diagram, one can choose a new variable and component split at each residual state. The address order is a deterministic function of the residual source graph, not a hidden generator hint. This preserves counts even if the initial graph is connected. Arbitrary general CNF remains NP-complete; exact search may create exponentially many residual states. Independent-set decision itself is trivial (the empty set is feasible); its counting is the substantive graph experiment.

## Local graph audit

30 connected instances: 4 path sizes 12/24/48/72; 4 linked-clique configurations (4x4,8x4,12x4,12x6); 4 rectangular grids 3x4,4x6,5x6,6x6; 3 random cycle-plus-chord sizes 12/18/24; 2 independent relabelings each. All input to solvers is the labeled source edge list (equivalent to the negative binary clauses); all generator labels are reserved for separate reference recurrences. Independent oracles: path Fibonacci recurrence, clique-port transfer, grid row transfer and random-graph meet-in-the-middle subset zeta transform. All <=18-variable cases also checked against exhaustive original-clause enumeration. All completed counts agree. Every completed witness reconstructed from cached positive branches and checked against original graph; no additional count search during replay.

| Policy | PASS | HOLD | Sum cached states | Recursive calls | Planning probes | Component vertex scans |
|---|---:|---:|---:|---:|---:|---:|
| Fixed natural |26|4|278638|557401|0|0|
| Fixed reverse |26|4|278236|556583|0|0|
| Graph-neighborhood |30|0|21425|42820|12018|0|
| Greedy frontier |30|0|27659|55288|21416|0|
| Dynamic one-vertex separator |30|0|5550|11341|255546|39796|
| Dynamic degree + split |30|0|5119|10767|21046|41944|

State sums include the partial computations of capped HOLDs for fixed orders. Cap 60000 memo entries checked at recursive state entry; in-flight completions can exceed this threshold slightly. All other policies completed; 172 completed runs out of 180, each with a valid witness. Planning counters are heterogeneous: neighborhood/frontier count candidate probes (frontier probes scan assigned vertices); separator counts vertex visits in trial induced components; cheap degree counts candidate vertices. Do not aggregate unlike units into runtime. Cheap degree + split uses 76.1% fewer cached states than neighborhood; neighborhood wins on paths (608 vs 1324), cheap split wins on linked cliques (4412 vs 484), grids (15440 vs 2796), random graphs (965 vs 515). Dynamic cheap split used 2212 exact component decompositions and 352 witness replay visits. No timing, byte, or advantage over optimized component-caching solvers claimed.

## General CNF controls

24 additional cases: 20 random/planted 3-CNF (n=8,12,16,20,24, 2 seeds each for random and planted, 5n distinct clauses), and 4 pigeonhole instances 3/2,4/3,5/4,4/4 (at least one hole per pigeon; at most one pigeon per hole). Fixed natural residual replay versus adaptive occurrence-count branching plus exact component split, both using unit propagation, canonical residual clause tuples, memoization, and separately accounted free-variable powers of two. Independent reference is shortest-clause DPLL with unit propagation, no caching/splitting. 24/24 exact counts in each arm, 16 SAT and 8 UNSAT, every SAT witness checked against the original source. Fixed replay: 434 cache states, 892 calls, 2462 unit propagations. Adaptive: 224 states, 472 calls, 1558 units, one component split, 28288 component-literal scans and 28282 planning-literal scans. Reference DPLL uses 590 recursive calls. These are small synthetic tests, no optimized external solver comparison, no worst-case bound. The graph and general CNF tests are separate counting workloads.

## Native and evidence scope

New native test src/automatic_shadow_replay.rs compares neighborhood and dynamic degree/split methods on 30 independently shuffled graph fixtures, with path/clique/grid references and independent subset-zeta counts on small graphs. It does not implement the separate generic-CNF audit or the fixed-order cap comparison. Local compiler unavailable; hosted acceptance is pending. The test uses the existing native shuffle, exact integer arithmetic and no new dependencies. Original Brown-Hilbert/Fischer and full matrix freeze machinery were not invoked.

Local executable sources and outputs (SHA-256):
- pnp_auto_shadow_audit.py 80162078704e0ea05583c9ec5ef2946a5f0dbd2343540bbc699e66320d152db5
- pnp_auto_shadow_results.txt 2ab07e744c4a58168215c85e31f9f8f81ba2d850e93c5617cede939869877d81
- pnp_auto_shadow_general.py 7d4bdb66c37c0ae0cfa6eea8529666c53a84e7b5867acfe153dd383b280ea4e5
- pnp_auto_shadow_general_results.txt 18a70946c68386c9cb2fcb9870e50bdab2be1e9ee4ee9fa6d2d55e3d4941c646
- complete local evidence ZIP 924d997a638a38c83e8a330358b075e50d7850f16f02800e1b6441be8fd38859

Next: independent held-out benchmark including increasing-width expander/Tseitin and random 3-SAT at threshold; compare against conventional component-caching model counters and optimized SAT solvers. Test adaptive selection using only charged source features, not hindsight winner selection; report discovery, canonicalization, search, replay, verification and memory separately. Investigate reuse across isomorphic residuals with certified address maps and exact source checks. P=NP remains open. Main unchanged.
