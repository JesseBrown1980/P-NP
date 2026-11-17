# PHP(21,20): exact CNF, adversarial answer flips and independent DRAT

Status: engineering pilot pending hosted acceptance. The experiment is a general-CNF source/decision/proof test. The existing SAT selector requires three literals per clause, so its results are not silently applied to this PHP family.

## Original variables and encodings
For 0 <= i < P pigeons and 0 <= h < H holes, Boolean x(i,h) is DIMACS variable i*H+h+1. No auxiliary variables.

* P-ALO: a positive H-literal clause for every pigeon; P clauses.
* H-AMO: for each hole and pair of distinct pigeons, a negative binary clause; H*C(P,2) clauses.
* Basic PHP total: P + H*C(P,2) clauses on P*H variables. Satisfiable iff P <= H.
* Functional variant: add P*C(H,2) negative binary clauses ensuring one hole per pigeon.

PHP(21,20) therefore has 420 variables, 21 P-ALO clauses, 4200 H-AMO clauses, and 4221 basic clauses. The optional functional encoding adds 3990 pigeon-AMO clauses, totaling 8211 clauses.

The large instance is generated and independently counted, but this first pilot does not claim to solve PHP(21,20).

## Five matched decisions
Check PHP(2,1) UNSAT, PHP(2,2) SAT, PHP(3,2) UNSAT, PHP(3,3) SAT and PHP(4,3) UNSAT. Compare the bounded general-CNF Python DPLL, MiniSat and CaDiCaL on byte-identical DIMACS. Verify both external SAT assignments on the original CNF. For all three UNSAT cases, CaDiCaL emits textual DRAT (--no-binary); independently pinned drat-trim checks the input/proof pair and the clean-room verifier repeats the check.

## One hundred answer flips
Twenty rounds on five instances. Flip SAT/UNSAT and alternate bare altered verdicts with forged original-source evidence: all-false SAT assignments or a DRAT proof borrowed from a different input. Require both independently valid SAT witnesses or a DRAT proof checked against the exact source. All 100 flipped verdicts must be rejected.

## Timings and evidence
Record generator bytes/time, CNF parse, native preprocessing, native search and witness validation separately. Record MiniSat/CaDiCaL process time and peak memory, proof generation and drat-trim validation, solver versions and source pins. Archive exact CNFs, models, solver logs, DRAT proofs, proof-checker logs and the SHA manifest, and re-download and recheck in a separate hosted job. Any missing file, mismatch, incorrect flip acceptance or unverified UNSAT proof fails the gate.

Before making a direct performance claim for PHP(21,20), run it under a separately declared time/memory budget with an independently checked UNSAT certificate.