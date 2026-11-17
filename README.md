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
