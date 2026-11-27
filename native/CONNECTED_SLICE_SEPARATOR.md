# Connected finite slices — exact bounded-width counting and reverse reconstruction

## Theorem

After exact GF(2) recognition and elimination of complete parity definitions in CNF, let the remaining free-coordinate predicates be factors f_i: {0,1}^{S_i} -> {0,1}. For a variable v, multiply every current factor containing v and sum over v. The resulting factor on the union of those supports minus v preserves the total sum of the product. An empty bucket contributes 2, preserving unconstrained coordinates. Applying this identity repeatedly gives the exact number of satisfying assignments; affine pivot reconstruction gives the corresponding full original-variable assignment. For a nonzero count, reverse the elimination order and choose a value with positive conditional bucket weight. Induction proves that each chosen value can extend to a satisfying assignment. Check the reconstructed assignment against the original CNF.

For an elimination order with largest bucket w (including the eliminated variable), the stored sum tables have at most 2^(w-1) entries apiece; for n variables and m original factors, the total number of table entries evaluated is at most n*2^w after initial factor construction. Charging all bucket products yields O((m+n)*2^w) integer arithmetic operations, since each factor is consumed at most once. This excludes preprocessing, planning, addresses and integer bit costs; counts need at most n+1 bits. For fixed w, bounded input factor supports and polynomial-time discovery of an order, the complete method is polynomial. It is a standard exact variable-elimination identity applied to our finite slice architecture, not a new general SAT complexity theorem. Width may grow linearly in arbitrary inputs; the min-degree heuristic need not discover an optimal order.

## Local executed audit

Prior DIMACS parity fixtures reused, original and auxiliary variable names and clauses shuffled. 5 sizes (6,12,24,48,60 original variables with equally many auxiliaries), 2 invertible parity families, 2 pattern seeds, 2 connected topologies (path and cycle of coupled triples) = 40 cases. Solver receives DIMACS alone; source labels only used by generator and separate transfer recurrence. All 40 exact counts matched the independent block transfer oracle, 8 smallest also matched independent original-coordinate DPLL. All 40 reconstructed full assignments passed original CNF; 40/40 bit-flip controls failed. The full clique on recovered coordinates gives 8 HOLDs above cap 6 (2 families at each of 12,24,48,60). The cap is an admission policy, not an UNSAT result. Largest successful bucket width 3 for every case.

| Original variables | Chain bucket entries (4 cases) | Cycle bucket entries (4 cases) |
|---:|---:|---:|
|6|120|152|
|12|248|344|
|24|564|728|
|48|1236|1496|
|60|1592|1880|

Dense greedy scope-scan plan totals: 8360 bucket entries, 7920 initial table entries, 4180 emitted count entries, 8160 factor products, 4180 additions; 944165 scope tests, 58910 elimination pivot tests, 4036 row XORs, 17240 input literal reads, 14160 parity pattern probes, 22240 table literal tests. Forward fixed address order admitted 39/40 (15474 entries across admitted cases); reverse order 40/40 (15916 entries). Orders are simple controls, not the original Fischer engine. Incremental graph minimum-degree planner also verified 40/40 with 8124 entries, width 3: 2780 initial edge attempts, 1651 heap pops, 2011 edge removals, 851 fill attempts, 2011 heap pushes. Different operation types are not summed into a speedup; no wall-time, peak-byte or advantage over optimized conventional model counters claimed. Neither method invokes trained STEER or signed leaves. Local independent Python research audit SHA-256 56a3f7ae955cb8c54576996b0e2b14fc34c63410476b83ff4454272a036f5930; output SHA-256 d141bc5464a435d97236d6f99b294818d8a3aeb93343d748; result stream SHA-256 16c7ba880c5a65973b5c01c17b4a3db783371cdae2ef6861827c5cbd352fb7b9. Local research audit depends on prior pnp_blind_cnf_audit.py; native test uses the earlier independent Rust fixture generator (different shuffles). The native implementation uses BTreeSet for the incremental planner, and checks count and witness in the original CNF. Rust compiler absent locally; native acceptance pending.

## Next gate

Test implicit XOR recovery, independent public DIMACS fixtures and harder graphs, including planted hidden low-width and high-width controls. Compare against a conventional XOR-aware model counter and an equally optimized bucket-elimination baseline; charge discovery, transformation, solving, validation, stored message bits and fallback. Reversible address mapping alone does not establish polynomial width. No P vs NP conclusion, no merge authorization.

Related: [swarm collective-elimination bridge](SWARM_COLLECTIVE_ELIMINATION_BRIDGE.md). Prior native run 45518351163 passed tests but failed strict Clippy suspicious-else formatting in blind projected reference; correction 140b30d66162d16f11d121234b50c1962e3934e1 leaves arithmetic unchanged. Earlier finite-slice run 45519048593 was in progress at inspection.
