# P-NP — exact complexity research laboratory

**Status (2026-11-17): OPEN RESEARCH.** This repository tests whether source-frozen, independently verified representation discovery and move selection can reduce the total cost of difficult constraint problems. The mathematical outcome is undecided. A finite set of successful experiments is evidence about algorithms and instances, while a statement about the complexity classes P and NP requires a mathematical theorem.

## Purpose

Build one reusable **freeze → propose → score → verify → commit** engine. We will evaluate both successes and counterexamples, including recognition time, heuristic ranking, verifier cost, full search, proof length, peak memory, and archived inputs and outputs.

- **MTP** proposes exact representations and corrective reductions.
- **Bobby Fischer-inspired scoring** ranks possible transformations by *total* predicted cost, subject to independent verification. Existing SAT-specific scores are heuristics; their value must be measured against alternatives.
- **N-Nest watchers** independently reconstruct every accepted transformation and reject altered proofs or witnesses.
- **SAT / #SAT** remain distinct tasks: deciding satisfiability versus counting all satisfying assignments.

## Provenance and continuity

This is a clean research home for the SAT work previously developed in [Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C](https://github.com/JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C). Its [integrated solver PR #299](https://github.com/JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C/pull/299) and [held-out benchmark PR #302](https://github.com/JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C/pull/302) provide the initial implementation and independent acceptance evidence. The original scientific record and Hamiltonian stay in their own repository.

The accepted starting evidence consists of (a) a source-frozen 11-case integrated SAT test with **7 exact counts and 4 budget HOLDs** and (b) a separate 21-case deterministic held-out pilot with **20 exact counts and 1 HOLD**, independently reconstructed source inputs and receipts, plus agreement between MiniSat and PicoSAT on SAT decisions. All these cases are finite, source-generated pilot problems; they do not settle the general complexity question.

## Evidence standards

Every accepted result has immutable input identity (SHA-256), explicit generator and version, exact or explicitly bounded output, an independently replayable checker, adversarial corruption tests, complete metering, and an archived receipt. HOLD and negative results remain visible. SAT witnesses are checked against the original CNF; an UNSAT assertion needs its own valid exhaustive oracle or independently checkable unsatisfiability certificate. Benchmarks cannot be substituted for worst-case proof.

## Roadmap

1. Transfer the original exact SAT code, clean-room verifiers, negative controls and hosted source/receipt evidence into this repo.
2. Implement a verifiable representation selector with multiple exact choices (XOR basis, bounded dependency tables, conventional CNF search) and *measured* move costs.
3. Expand to independent non-generator-held-out SAT and #SAT benchmark corpora, with pinned baseline versions and the same hardware limits.
4. Plot scaling of end-to-end computation, including discovery, scoring, watcher and archive overhead, rather than the final search stage alone.
5. Separately develop and referee any mathematical claim; track lemmas, assumptions, counterexamples and independently checkable proofs.

See [docs/RESEARCH_PROTOCOL.md](docs/RESEARCH_PROTOCOL.md) for the acceptance contract once the first port is complete.

**Research direction:** aggressively test the possibility of new structural reductions while keeping all mathematical conclusions open to evidence.


## Run the exact research pilots

The core scripts use Python 3 and exact integer arithmetic; the archived GitHub Actions workflows run the pinned external SAT tools and re-download their evidence. The local commands for the original 11-case regression are:

```bash
python3 -I -B sat/engine.py > integrated.hbp
python3 -I -B sat/verify.py integrated.hbp
```

The first adaptive selector is separately executable, including raw source and independent exact-count verification:

```bash
mkdir -p build/selector/cnf
python3 -s -B sat/selector.py build/selector/cnf > build/selector/SELECTOR.hbp
python3 -s -B sat/verify_selector.py build/selector/SELECTOR.hbp build/selector/cnf
```

See [architecture](docs/ARCHITECTURE.md), [provenance](docs/PROVENANCE.md), [evidence ledger](docs/RESULTS.md), [research protocol](docs/RESEARCH_PROTOCOL.md) and [first selector analysis](docs/SELECTOR_V0.md).

## Active mathematical and engineering questions

[Exact representation selection](https://github.com/JesseBrown1980/P-NP/issues/2), [independent SAT/#SAT benchmarking](https://github.com/JesseBrown1980/P-NP/issues/3), [formal complexity proof obligations](https://github.com/JesseBrown1980/P-NP/issues/4), and [MTP/Fischer/N-Nest watcher architecture](https://github.com/JesseBrown1980/P-NP/issues/5) are tracked independently. We preserve the original questions and experiments as input evidence, while treating new P-NP work as its own reproducible program.

## Original Fischer source preserved exactly

The [historical ASOLARIA Rime Fischer implementation](historical/asolaria-2026-07-21/rime_fischer.py), its [original sphere dependency](historical/asolaria-2026-07-21/rime_sphere.py), and the original cluster and 27-Fischer variants are preserved byte for byte from the [original ASOLARIA universe simulator](https://github.com/JesseBrown1980/ASOLARIA-UNIVERSE-SIMULATOR-with-gravity-light-time-space-color-rime-winding-drills). Their exact upstream Git blobs are checked by hosted CI. The original code runs unchanged, and 21 discrete-log test cases match the new reimplementation and an independent full-order BSGS referee. See the [historical-source acceptance](experiments/historical/README.md) and [results ledger](docs/RESULTS.md). The historical source computes discrete logarithms; the separate SAT competition uses a verified PH-guided branch-order adaptation rather than treating discrete logs themselves as SAT solutions.

## Hard SAT challenge: six competitors, four original CNFs

The [source-frozen six-contender hard SAT benchmark](docs/HARD_SAT_SIX_WAY_20261117.md) tested the original Yang–Mills Fischer strategy, newly implemented Rime, byte-identical historical ASOLARIA Fischer, MiniSat, CaDiCaL and PicoSAT on PHP(21,20), a 128-vertex odd-charge Tseitin contradiction and two controls. The original SAT witness, exact XOR and DRAT claims were independently checked; 100 adversarial verdict flips were rejected. The largest PHP challenge exhausted all declared budgets, while all three structural approaches certified the Tseitin contradiction at zero search nodes. The mathematical P versus NP question remains open.


## 84-case six-competitor SAT array

The [accepted 14-input × six-arm research array](docs/SAT_ARRAY_84_RESULTS_20261117.md) reproduces 84 source-identical SAT runs across original Yang–Mills Fischer, new Pohlig–Hellman-guided Rime, byte-original historical ASOLARIA Rime, MiniSat, CaDiCaL and PicoSAT. All six arms completed 11/14 decisions, with three declared HOLDs each. The modern and historical PH variants used fewer search nodes on medium UNSAT pigeonhole tests; Yang–Mills used fewer on larger satisfiable controls. The parity-aware research engines certified four Tseitin contradictions at zero search nodes. Independent acceptance verified all 84 original sources, SAT witnesses, eight CaDiCaL DRAT proofs and 20 corrupted-witness controls. [Hosted final run #44208774230](https://github.com/JesseBrown1980/P-NP/actions/runs/44208774230).

## Fourth engine: structure-aware Hall + XOR + fallback

A [new fourth SAT research contender (HXF-1)](experiments/fourth/README.md) now independently recognizes mutually-exclusive slot/requirement structures and certifies Hall UNSAT or constructs SAT matching witnesses, recognizes parity groups and produces exact source-row XOR contradiction certificates, and falls back to bounded general-CNF DPLL.

In a source-identical 14-case head-to-head with all three previously archived Fischer research engines, HXF-1 completed **14/14** with zero search nodes; each original engine completed **11/14** at its 2,000-node ceiling. Independent tests accepted 42 blind variable-and-clause-reordered variants and five near-miss fallback controls. The three original research engines and historical code are unchanged. [Hosted accepted run #44210600049](https://github.com/JesseBrown1980/P-NP/actions/runs/44210600049). General P versus NP remains open.