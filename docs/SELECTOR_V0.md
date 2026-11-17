# Adaptive representation selector v0 — exact candidate accounting

**Status:** experimental, subject to hosted acceptance. Base source: source-certified P-NP SAT engine on `main`.

This experiment applies one reusable **freeze → propose → score → watcher → commit** sequence to the 21 deterministic held-out SAT cases.

## Candidate spaces

- **XOR-affine + Fischer order:** independently authenticated parity gadgets are eliminated over GF(2); the remaining 3-CNF constraints are tested over free variables with SAT-specific variable scores.
- **XOR-affine + fixed order:** the same exact reduction, but a fixed residual branch order.
- **Raw CNF + fixed order:** original clauses without parity compression.
- When there are no recognized XOR motifs, replace the two affine candidates with **raw CNF + Fischer order**, alongside raw CNF + fixed order.

Only complete, independently recognizable XOR four-clause motifs are committed. Each candidate's exactness is determined by its original-CNF witness or complete exhaustive counting. The accepted parity basis also undergoes opposite-direction Gaussian elimination and expression replay.

## Selector logic

All candidates are probed for up to **96 nodes**, including proposal, scoring and watcher work. Completed exact probes outrank incomplete probes. Within each class, candidates are ranked by their **measured probe cost** in exact-integer operation units, then by stable name. The selected candidate is run up to **20,000 nodes**, and a bounded exact fallback is attempted if it holds.

The reported total work includes source bytes and clause writes, discovery, Gaussian elimination, every watcher, all bounded probes, every attempted full search, and witness verification. Timings and high-water RSS are reported separately. This rule is intentionally simple and falsifiable: an inexpensive short probe may lead to an expensive full search. Report those counterexamples rather than concealing them.

## Independent acceptance

`sat/verify_selector.py` reads raw source files and reconstructs all 21 fixture cases without importing the selector or engine. It verifies the SHA source/proof/trace/receipt chain, independent Gaussian rank and residual counts, deterministic candidate order, probe and execution work arithmetic, original-CNF witnesses, and exact counts using a separate exact counter. Ten mutation controls modify receipt hashes, counts, work, or the selected representation and require rejection.

The hosted workflow stores source files, full case receipts and checker output. A separate job downloads the published archive and checks SHA-256 of every retained source and artifact.

**Acceptance target:** a reliable, exact selector with fully measured costs, positive and negative empirical results on the finite development set. This is a first choice between **raw CNF and XOR-affine** representations. Dependency tables, learned decompositions and independent unseen datasets are separately tracked in [issue #2](https://github.com/JesseBrown1980/P-NP/issues/2). P=NP remains OPEN.
