# Signed residual equivalence and addressed generator reuse (2026-11-27)

Previous native automatic shadow replay acceptance PASS: run 45886348707, commit d2792ded04be320e0ee753326e153a010f26a2b8. Previous Matrix Law Essentials PR #19 PASS: run 45886374618. This new gate is research-only; native hosted acceptance pending.

## Exact identity

For finite Boolean CNFs F and G, a signed permutation of active variables maps every source literal to a target literal, with opposite literals mapped to opposites. Require a bijection on underlying variables and exact equality of the transformed normalized source clause set with the normalized target clause set. This proves a bijection of satisfying assignments, preserving the exact count; target witness bits are mapped back by the inverse signed correspondence. Count free variables separately by powers of two. The proof does not require identifying all semantically equivalent CNFs. It applies only to certified syntactic signed isomorphisms after normalization (duplicate clauses/literals and tautologies removed in the local implementation).

Search for maps using clause-size/sign occurrence profiles, then backtrack over compatible target literals, enforcing injectivity and fully mapped clause membership. Accept only complete mapped-clause equality. The profile is a filter, never a certificate. A fixed per-comparison probe cap prevents unbounded backtracking inside this recognizer; a failed search or cap means no reuse, not non-equivalence or UNSAT. Count or solve the source independently on a miss. Discovery and bank traversal costs remain separate from solving. Neither the number of residual classes nor the fallback search is proved polynomial.

## Local executed audit

Six templates of sizes 6,6,8,10,8,10: triangular prism, K3,3, two seeded 3-CNFs, an explicit contradictory three-variable core with extra clauses, and a cycle. Independent exhaustive assignment counts: 13,15,4,8,0,123. For 3/9/27/81 copies per template, each gets fresh disjoint physical variable labels and random literal polarity flips, and the bank is reset between sizes. No prototype metadata is provided to the matcher. Reference counts for all 720 copies are independently derived by bijection from those six exhaustive counts and checked with direct uncached DPLL. Literal addressed cache has 720 distinct keys. Reuse solves 25 generators (one extra due to a cap), certifies 695 cache hits, checks all 600 positive witnesses against their source clauses. No combined formula is allocated or solved. All 720 counts agree.

|Copies per template|Instances|Direct DPLL calls|Reuse counting calls|Successful maps|Generators computed|
|---:|---:|---:|---:|---:|---:|
|3|18|320|92|12|6|
|9|54|964|122|48|6|
|27|162|2990|118|156|6|
|81|486|9022|133|479|7|
|Total|720|13296|465|695|25|

Separate discovery ledger: 816 candidate bank comparisons, 135735 attempted mapping assignments, 155726 completed-clause membership checks, 695 complete equality checks, one 4000-probe mapping cap. Direct DPLL uses 14671 unit propagations; reuse counting uses 508. These are distinct operation types, not a runtime speedup. Feature computation and witness replay additional. An eight-round literal/negation/clause incidence refinement filter was tested against the same inputs: 135013 probes, 154969 clause checks, identical hits and cap. The extra refinement barely reduces probes (~0.53%) and is not established as useful. No wall-time or memory advantage claimed.

## Connected and unstructured controls

16 connected selector CNFs, n=6,8,10,12 source variables per arm and four independently seeded templates each. A selector gates two differently signed/readdressed copies on disjoint variable sets. With selector fixed, the inactive arm contributes 2^n and the active arm has the template count. Exact reference is 2^(n+1) times the independently bit-parallel exhaustive template count. Both fixed literal residual caching and signed residual reuse agree on all counts; all positive witnesses checked. Counting calls: 628 vs 334; stored residual states: 306 vs 159; units: 1004 vs 511; hits: 2 vs 20. Signed refinement arm: 20 bank comparisons, 152 map probes, 437 mapped-clause checks. The branching policy is identical in the two arms. This is a deliberately reusable structured family.

24 unstructured random 3-CNFs, sizes 8/12/16/20, six seeds each and 4n distinct clauses. All counts agree with independent exhaustive bit-parallel enumeration (up to 2^20 assignments represented as integer bit masks). Literal cache: 608 calls, 292 stored states, 1 hit. Signed cache: 604 calls, 290 states, 3 hits. 3 bank comparisons, 6 mapping probes, 4 mapped-clause membership checks; feature calculations at 293 lookups are additional. Little benefit on this suite; no general efficiency claim.

24 randomized prism-versus-K3,3 pairs have identical occurrence profiles and identical tested incidence-refinement color histograms, but distinct counts (13 and 15) and no signed isomorphism. All 24 rejected; 24 positive relabeled prism maps verified; all 24 one-sign corruption controls rejected by exact mapped-clause equality. The collision controls use 9564 probes and 12574 mapped-clause checks in aggregate (positive and negative). Degree profiles and color histograms cannot authorize reuse by themselves.

## Native scope and source evidence

New test src/signed_residual_reuse.rs exercises bounded signed mapping on four small templates, 96 positive relabelings, DPLL/exhaustive count comparisons, witness reconstruction, zero-cap controls and 24 same-profile collision/corruption controls. Independent fixture generation differs from local Python. The native test does not implement the complete bank, connected selector benchmark, or refinement filter. Native compiler unavailable locally; hosted acceptance required. The original Brown-Hilbert/Fischer engines are not invoked. Main unchanged.

Local evidence SHA-256:
- pnp_signed_residual_reuse.py 7eadd2f9bd4be4a0c88cfa6444edf4519f62812099df310b41fb53199f413bc6
- pnp_signed_residual_reuse_results.txt 7627fb3a84dad1679f18aa1c89453b2e36d2c8feaefa7faf45fe97a2bce9a674
- pnp_signed_residual_refinement.py bb9de3bf6547a7ce7fd9b10320efe9421478f905ad24d7cd52ccc69939d25498
- pnp_signed_residual_refinement_results.txt b602b279c2febf8d5bc6646d3b0158e25c867f6901ebd38a1fb2fa0e9c798cdf
- pnp_signed_residual_controls.py 585aa719507b898ea70d878236bacb18c6beeb1256ac96b8e2be1e8d739d9fe0
- pnp_signed_residual_controls_results.txt 1889994588d3627976702b7643bc3811eedbff7f4e8cb1aae3d9ad6d2bb3d5d2
- evidence ZIP 193850714df5bf9988b7ca875c75af20aa32979acd63f177b39a955f6f687104

Next: charge source-feature and address-discovery costs; test efficient canonicalization of bounded-width incidence graphs, repeated substructures within independently generated high-width formulas, and comparison with conventional symmetry-aware component counters. Distinguish signed syntactic symmetry from semantic equivalence (which itself is computationally difficult). P=NP remains open.
