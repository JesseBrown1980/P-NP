# Collective elimination research bridge — 2026-11-24

## External source and status

[P = NP agent swarm](https://pequalnp.com/), strategy update retrieved 2026-11-24; [public Lean proofs](https://github.com/brilliant-hq/pequalnp-proofs). The swarm reports a circuit family with a shared helper where removing the entire dependent class avoids circuit growth observed when removing individual members. Its report explicitly marks this as computational evidence pending formal checking. Record the lead as EXTERNAL_CONJECTURE, with no imported proof or general complexity claim.

## Source-linked QNN application

[QNN PR #40](https://github.com/JesseBrown1980/GGUF-files-operating-as-trained-quantum-neural-network-models/pull/40), stacked on [radiated slice PR #39](https://github.com/JesseBrown1980/GGUF-files-operating-as-trained-quantum-neural-network-models/pull/39), whose hosted native and radiated-factorization checks passed. [Asolaria Radiated Slice](https://github.com/JesseBrown1980/Asolaria/blob/main/docs/THE-RADIATED-SLICE.md) supplies the address/generator motivation. [Prior P-NP eight-square PR #70](https://github.com/JesseBrown1980/P-NP/pull/70) supplies the complementary signed route for separate investigation.

V7 dressing is P^-1 R P with invertible ternary SUM-ring address map P. For output x and y=(Px)_q, conjugated local numerator matrix M contributes sum_t M[y,t] psi(x+(t-y)P^-1 e_q), with ternary address arithmetic. A gate depends only on the union of the support of row q of P and column q of P^-1. Union the four initial pair blocks touched by every active gate. Since every gate acts inside one resulting component, the state is exactly a product of component tensors. This is an independent sufficient factorization proof, rather than an application of an unproved external Boolean claim.

Local exact integer replay of published V7 rotation receipts: 59,049/59,049 amplitude equalities, 9/9 component corruption rejections. Four inactive-layer checkpoints need 36 entries each; two active identity-seeded checkpoints need 738 each; three star checkpoints remain at 6,561 each. Total 21,303 vs 59,049 entries. The two newly reduced checkpoints use 2,430 vs 21,870 dressing coefficient products; preparation, addressing and verification additional. Native SHA-pinned Rust 1.81 acceptance passed in QNN PR #40 at corrected commit 6089874f00750da4550fbe2ca30e67b6671157aa (runs 45517041433 and 45517041377). These are state-entry and arithmetic counts, not wall time, and an optimized conventional tensor network may use the same exact factorization.

## P-NP follow-up gate (proposed; no native acceptance yet)

1. Identify complete dependency classes by certified supports, including overlapping classes and cycles. Preserve original source identities and all required helper semantics.
2. Compare collective and individual rewrites with a common exact verifier; charge helper reconstruction, planning, intermediate memory and product-tree costs, plus referee cost.
3. Reuse the native ternary matrix / third fabric witness only after independent result parity and complete HBI/HBP receipt checks; Rust 1.81, integer only, no unsafe, no Node/JSON/V8, offline locked Clippy.
4. Test conditional separators in the fully connected QNN star models; then test Boolean circuits with held-out shared-helper structures, external SAT solvers, and equal optimized direct baseline. Reject faulty class removal and altered helpers. Separate structural arithmetic from measured execution.
5. Generalize only when a formal identity or certified proof applies to the entire stated class. No SAT algorithm, polynomial bound, or P versus NP verdict follows from these finite QNN certificates.

Research record only. No change to production solver or historical acceptance records.

## Boolean coordinate recovery control (implemented, hosted acceptance pending)

[QNN PR #41](https://github.com/JesseBrown1980/GGUF-files-operating-as-trained-quantum-neural-network-models/pull/41) motivates the Boolean address-basis experiment: exact rank-27 forward-ring cuts in the three V7 star models.

Let y = P x over F_2 with P invertible. If F(x) is a conjunction of predicates on disjoint groups of y coordinates, its solution count is the product of group counts, including a factor of two for every unconstrained coordinate. The proof is the bijection x -> y and the Cartesian product of local solution sets. Support closure must merge groups touched by every transformed clause. For explicit linear parity forms, choose a basis by GF(2) elimination, express every form in that basis, and union the transformed clause supports. The test fixtures span the full ambient dimension; arbitrary rank-deficient input needs basis completion and free-coordinate handling.

Native integration test: [address_factorization.rs](tests/address_factorization.rs). Two invertible mask families (adjacent XOR chain and shared first-coordinate XOR), sizes 6, 9, 12, 15, three deterministic forbidden-pattern seeds and two variants with/without a bridge coupling neighboring triples. The factor algorithm receives explicit parity forms, not the basis or component labels. An independent reference expands each forbidden parity pattern into ordinary CNF clauses on original coordinates, enumerates all original assignments, verifies predicate equivalence and address bijectivity, and compares the exact count with component enumeration. Omission of the bridge must alter membership and increase the count. Synthetic parity-aware control, not a blinded CNF parser benchmark; explicit forms are privileged structural input.

Local independent Python research audit executed: 48/48 instances, 449280 original assignment comparisons, 2496 local assignments, 624 compiled CNF clauses across fixtures, 24/24 bridge omission controls. At n=15, unbridged cases use 40 local assignments versus 32768 original assignments; bridged cases use 88. The 180-fold aggregate ratio counts assignments only, not operations, timings, discovery, CNF construction, or verifier costs. No external SAT solver available in the local execution environment. The native Rust 1.81 test is committed for hosted mandatory acceptance; local Rust compiler unavailable. Local audit SHA-256 27236bffa335196c7a142240cd436bb82dce46a74ac1a54d5a599bd12389ef4e; output SHA-256 c8ac52f941e5f2c0ad6b7d420d1648bb43a2d27b206bd51646ff84c79bbe4bd5. No claim of accepted production software, arbitrary-CNF discovery, polynomial worst-case algorithm, or P versus NP resolution.

Next blind test: give the solver only ordinary DIMACS CNF with auxiliary variable names randomized; recover parity/helper classes without generator metadata, verify equivalence including projected counts for auxiliaries, and compare against equally optimized XOR-aware and component-aware baselines plus independent SAT solvers. Include arbitrary nonfactorable controls and charge all recognition, rewriting, verification, and fallback costs.
