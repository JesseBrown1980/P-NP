# Research protocol — falsifiable P versus NP investigation

**Version:** 2026-11-17. **Question:** can a bounded and independently certified representation-discovery hierarchy solve progressively difficult constraint instances with improved *total* computational cost?

## Precise mathematical target

For a decision problem such as 3-SAT, the P=NP claim would need a mathematically justified algorithm whose worst-case running time is polynomial in the encoded input length, with every discovery, scoring, checking, search and memory operation accounted for. Successful performance on finite benchmarks establishes empirical behavior for those inputs; the mathematical theorem is a separate acceptance track.

The default status of a conjecture is OPEN. Supportive measurements, counterexamples to a heuristic, and unresolved instances are archived with equal care. A solver returning a model count and a solver deciding SAT/UNSAT are evaluated as separate algorithms; #SAT model counting has its own proof requirements.

## Source-to-acceptance protocol

1. **FREEZE:** represent the *actual* clauses or input bytes, machine/environment versions, deterministic seeds, input length, source SHA-256, and algorithm commit. Never use a generator's planted assignment as a solver input.
2. **PROPOSE (MTP):** propose an explicit identity, equivalent CNF, parity constraint, low-rank relation, dependency table or search reduction. Attach a reconstruction recipe and its declared domain of validity.
3. **SCORE (Fischer):** choose moves from costs of *all* candidates, including failed proposals, analysis, verifier work, data conversion and memory. Log the fixed-order alternative. All heuristic gains remain empirical.
4. **WATCH (N-Nest):** check from immutable original clauses with an independently written derivation or alternate algorithm. Reject altered SHA, forged status, corrupted witness, missing clause and invalid representation. Watchers do not authorize their own results.
5. **COMMIT:** only after source identity, exact transformation and outcome checks, emit a reproducible HBP receipt (kind, counts, SHA chain, status, work, peak memory and time). Store the independent checker and result next to the source.
6. **ARCHIVE:** re-download the published artifact in a different CI job and validate all files and hashes; green job completion alone is insufficient to guarantee the evidence archive is readable.

## Acceptance boundaries

- **SAT:** a reported SAT instance needs a witness checked against the original clauses. An UNSAT finding needs full independent enumeration for a small instance or a separately checkable UNSAT proof for a large one. Agreement among SAT solvers provides useful independent corroboration.
- **Exact #SAT:** a claimed count needs independently implemented exact enumeration, an algebraic certificate, or another exact trusted counter. A SAT decision alone never proves the model count.
- **HOLD:** budget exhaustion returns an explicit unknown count; a separate SAT solver can still establish a decision or witness. Never convert a HOLD into a proof of hardness, success, UNSAT, or a numerical model count.
- **Scaling:** include input bytes and generation, discovery, candidate scores, watchers, searches, other exact baselines, proof verification, persistence and archive overhead. Report peak RSS, elapsed integer nanoseconds, operation proxies, factor width and verifier cost separately. Record every timeout and threshold.
- **Reproducibility:** fixed versions; controlled CPU/RAM; raw DIMACS; complete case-level transcripts; SAT/UNSAT decisions and #SAT counts separately; negative tests and corrupted-receipt trials.
- **Generalization:** source-generated known fixture families are development sets. Later holdouts should come from independent published families or preregistered randomized generators with previously unrevealed seeds, compared under matched budgets with established solvers.

## Research claims ledger

Use the labels **PASS** (specific tested gate), **HOLD** (incomplete acceptance), **OPEN** (unproved hypothesis), and **REFUTED** (specific counterexample). Do not infer worst-case complexity from polynomial-looking plots. Each theorem candidate gets a precise statement, proof obligations, search for counterexamples, and an independent mathematical referee.

A mathematically accepted P=NP result must give a uniform polynomial bound for general 3-SAT input size under a well-specified classical model of computation. For P≠NP, a valid unconditional separation proof would be required. Keep both possible outcomes live.
