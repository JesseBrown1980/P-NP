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


## First hosted results — reproducible pilot, November 17, 2026

[Run #44193491896](https://github.com/JesseBrown1980/P-NP/actions/runs/44193491896) **PASS** for both source/selector acceptance and a separately downloaded archived artifact. The checker independently reproduced all original-CNF exact counts and SHA proof/trace/receipt chains; **10 altered selector receipts were rejected**. Across 21 cases, the selected algorithm returned **20 EXACT** and **one HOLD_BUDGET**, exactly matching the already certified count/HOLD outcomes.

Committed representation outcomes: **6 XOR-affine** (three Fischer orders and three fixed), **14 raw CNF** (ten fixed, four Fischer), and **one HOLD**. Four raw-fixer pilot selections exhausted their full budgets and then succeeded via the measured raw-Fischer fallback. All failed searches are included in the final total. The SHA archive ID and full per-instance receipts are stored in the hosted run.

**Important falsifiable counterexample:** on the three mixed cases, the short-probe heuristic selected affine-fixed, requiring **61, 1,081 and 10,075 search nodes** for n=12/24/36, while the upstream affine-Fischer ordering needed **51, 645 and 8,591** nodes. The selector produced the correct exact counts **14, 200, 1,824**, but did not choose the least-node option in these cases.

On pure XOR n=24/48/96, the selector returned exact counts **4/1/4**, each in **one full-search node** after independent parity reconstruction. Total counted work including all candidate probes was **25,307 / 71,327 / 251,582** units. The 96-node counterfactual probes consumed **34.3% of the combined XOR-family work metric**, even though the committed algebraic search itself was trivial. This is a specific reason to improve selector overhead on recognizable structured inputs.

Across all 21 cases the selector recorded **31,978,925** total defined work units, of which **666,153** came from pilot probes and **30,994,544** from attempted full search paths. These instrumented units are defined counters, not processor instructions or a hardware-matched solver speedup measurement. The remaining work includes input bytes, discovery, watchers and metering. The full case-level evidence, including counterexamples and HOLD, is archived.

**Next selector objective:** preregister a cheaper decision rule that recognizes when a completed algebraic certificate dominates unnecessary trials, and test whether calibrated score estimates beat simple fixed ordering on genuinely independent held-out families while including all choice costs. The current output is an exact, independently checked algorithmic pilot; the general complexity question remains OPEN.
