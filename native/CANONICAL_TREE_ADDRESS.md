# Canonical signed tree generator addressing and Fischer replay (2026-11-27)

Previous signed residual reuse native acceptance PASS, run 46009108394, commit d5e9446a619b08cb54af0f07e3b230b49587109c. P-NP is the primary research repo. No changes to Matrix Law Essentials in this stage.

## Exact canonical tree theorem

Domain: normalized finite CNF with nonempty clauses, distinct underlying variables within a clause, and a connected acyclic variable-clause incidence graph. All active variables represented. Disconnected formulas may be handled by earlier exact component factorization; this gate only handles connected trees. The graph has two node types and each incidence edge has a literal sign. A signed variable permutation preserves node types and clause membership, and flips all signs incident to a complemented variable.

Find the one or two unrooted tree centers by leaf removal. At a rooted variable, use the sign of its parent clause edge as its polarity orientation (so the parent incidence becomes positive). Encode every child clause by its sign relative to that orientation and its recursively canonical subtree. At a root variable, try both orientations and choose the smaller code. At a clause, encode the sorted multiset of child variable codes. Include node types, child counts and sign tags so the flattened code is uniquely parseable. Sort children by complete codes; tied subtrees may be traversed in either order because the codes agree. Choose the least full code across center roots and root orientations. Canonical preorder assigns consecutive variable addresses and records each chosen sign. The resulting signed map must reproduce the full normalized canonical clause image before any cached count or witness is reused.

Induction on rooted subtree size proves invariance under signed permutations, and equality of codes yields an explicit isomorphism by matching corresponding child codes. Tree centers are invariant under graph isomorphism, so the minimum is an unrooted canonical form for this domain. There are at most four candidate rooted encodings, each visits every tree node exactly once; naive repeated serialization, copying and lexicographic sorting can cost polynomial more than the visit count (e.g. a conservative O(L^3) bound with straightforward token vectors for L incidence-tree encoding length). No general graph canonicalization or semantic equivalence claimed. This is typed signed-tree canonicalization, not a new general complexity result.

For an acyclic incidence graph, exact variable-to-clause messages are products of incoming messages, and a clause-to-variable message equals the total of child products minus the product of all falsifying child values if the parent literal is false. Each message has two entries. This is the usual sum-product tree algorithm with integer counts of at most n+1 bits; a positive witness can be reconstructed with conditional messages. The generic factor-elimination method in the local audit is separate from this reference. Tree-shaped CNF is already polynomially solvable; the new reuse experiment measures repeated computation, not a complexity-class separation.

## Local audit results

Five sizes n=8,16,32,64,96, six independently seeded mixed-sign tree templates per size, 12 signed random relabelings per template. One template per size has contradictory units; additional contradictions may arise naturally. The recognizer receives clauses and variable count, not the template ID. 360/360 exact counts checked against independent tree sum-product; all 12 smallest templates additionally exhaustively counted. For each relabeling, generic exact bucket elimination on the source serves as direct comparator, while one canonical image is solved per new code; every full canonical clause image checked for equality, every positive witness mapped back and checked against original CNF.

| Variables | Cases | Hits | Direct bucket rows | Generator bucket rows | Tree-node visits |
|---:|---:|---:|---:|---:|---:|
|8|72|66|3504|292|2256|
|16|72|66|8532|716|3948|
|32|72|66|17048|1460|9888|
|64|72|66|34376|2916|16272|
|96|72|66|52604|4648|28116|
|Total|360|330|116064|10032|60480|

30 distinct keys and 288 source-verified positive witnesses. Canonicalization is direct, no pairwise signed-map backtracking. Discovery visits count each candidate root pass, not sorting/comparison/serialization costs. Bucket rows are not equivalent to tree visits or bit operations. No wall-time or optimized external count comparison. Same-shape sign control: (a) AND (a OR b) has count 2, (a) AND (NOT a OR b) has count 1; canonical keys differ. 24 random 3-CNF cyclic controls rejected by specialized compiler (not UNSAT). External Z3 4.13.3.0 agrees on all 30 template decisions, solves cyclic controls (4 SAT,20 UNSAT); 12 smallest cyclic decisions independently exhaustively checked.

## Actual Fischer interface and search control

The existing native freeze::fischer_address uses 7^(27*position+glyph) modulo 1,000,081. New native generator bank hashes its complete canonical token vector with original sha256::hash_bytes, reduces the first 64 digest bits modulo p-1, converts the exponent into (position,glyph), and calls the existing Fischer function. Modular addresses are routing buckets, not equivalence proofs. Even a forced routing collision must compare full canonical keys and mapped clause images before reuse. Local replay of the same modular formula with the Python tuple serialization found 30 buckets for 30 keys and 330 hits; serialization differs from native. No claim of universal injectivity or compact storage beyond the actual keys.

Independent Python replay of the existing native solver's Base and Fischer guidance (occurrence count x1024, Fischer priority p-address, three lanes, lane sort and round-robin) compared forward and reverse Fischer order on the 24 cyclic controls, 8/16/24/32 variables, six seeds each, nominal 5n random clauses before deduplication. All 72 decisions agree with Z3, SAT witnesses validated. Budget 100000 recursive nodes, none held. Same exact DPLL propagation and branching implementation for every arm.

|Policy|Nodes|Literal checks|Assignments|
|---|---:|---:|---:|
|Base|1153|620721|7081|
|Fischer|1100|607404|6682|
|Reverse Fischer|1135|569729|7249|

Forward Fischer reduces nodes by 4.6%, reverse reduces literal checks by 8.2% versus Base. No timing claim; guidance and reverse-order construction cost additional. Reverse order here means reversing the generated variable-order vector, not a full bidirectional proof search or original nine/twenty-step Fischer methodology. Z3 is an external decision oracle, not a timed comparator. No optimized external solver benchmark yet.

Native test src/canonical_tree_address.rs: exact signed tree code, bank using original Fischer routing and SHA, independent native count recurrence, inverse signed witness map, forced address-bucket collision test, cycle rejection. Native tree fixtures use different deterministic shuffles. A second native test calls existing Guidance::for_source for Base/Fischer/Hybrid3 and reversed Fischer order, then the actual solver::solve and exhaustive checker on twelve cyclic random controls of sizes 8/12/16. Hybrid3 uses existing FrozenMatrix and three-seat wave interface. Local native compiler unavailable; these new native tests are committed but not yet accepted. Full original Brown-Hilbert engine and genuine bidirectional playback not integrated in this stage.

## Evidence

SHA256 of local executable scripts and results:
- pnp_canonical_tree_audit.py 4db4d71ecf10e4e73ab4da5532d3ad2487fa1c549bf1b852b3e9f36614f34953
- pnp_canonical_tree_results.txt 0cd8a97b394b6772bffd085fe5a5b264b841694f442b9aa9609e46985841a78d
- pnp_canonical_tree_z3.py 4ac9a410069c39e43cdf8ac06a09cc703f647e8a24ed1b3b18d6dd2ada710ad8
- pnp_canonical_tree_z3_results.txt d7276795376338ef0c0dcc61bbd47bb5c9e6e1eee907ab059af40fb56ac99940
- pnp_fischer_order_control.py 73ee812f5f4238611bdd901c7c16a02d67935f990812672b109c2f1b6f6523a7
- pnp_fischer_order_results.txt b34c03927de6bc3490ada9dc002aaac2892660eadec097d1c6f43b2583649798
- local ZIP e41e61c243e1f20fe77231f8dfdfaabb555025bb687ae56e8e92fa0c09b6a698

Next chosen target: certified decomposition of cyclic residuals into small-boundary blocks (block-cut tree, then separators with explicitly bounded ports). Canonicalize each block while retaining named boundary ports and signed compatibility, not merely unconditioned counts. Compare against ordinary block-cut dynamic programming and optimized conventional model counters; charge discovery, state count and memory. General high-connectivity controls must fail closed or use conventional fallback. P=NP remains open; no merge authorization.
