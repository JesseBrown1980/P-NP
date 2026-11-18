# P-NP — exact complexity research laboratory

## Mandatory native software acceptance (November 17–18, 2026)

**The Yang–Mills engineering standard now governs every P-NP implementation, retrospectively and prospectively.** Rust **1.81.0**, exact integers/trits, zero floating point, **zero Node.js / JSON / V8 runtime**, offline locked builds, Clippy `-D warnings`, original-source SHA pins, separate independent checks, tamper rejection, exact HBP/HBI receipts and independently rechecked archives.

**All 44 historical Python executable files and all 10 prior CI workflows are `FAIL_NATIVE_ACCEPTANCE` as software under this new standard**, including earlier Fischer/HXF, SAT-array, Bitcoin, RIME, MTP and N-Nest prototypes. Their original source blobs and independently replayable research results remain preserved. Historical GREEN benchmark receipts mean the archived tests passed their *original* requirements; they are not Yang–Mills-grade software approval. Pending Python-first drafts also require Rust-native recertification.

The [binding engineering contract](policy/NATIVE_ACCEPTANCE.md) and [exactly pinned legacy inventory](policy/LEGACY_PINNED.hbp) define the cutover. The native Rust `native/` acceptance pilot covers universal 27-cell matrix addressing, 2D shadow freeze/play and the matrix-as-third-fabric-witness cascade. The complete trained GNN/FNN, Omni/Host-8, hotel, tournament, and SAT-solving architecture is **HOLD pending its own native implementation and tests**. [Native CI workflow](.github/workflows/pnp-native-certification.yml).

---

## Historical GNN/FNN source authentication — HBI/HBP/SHA/SH/HASH (November 18, 2026)

The next native phase [PR #30](https://github.com/JesseBrown1980/P-NP/pull/30) establishes the original trained-model provenance gate using the Asolaria seal **HBI → HBP → SHA → SH → HASH**. The [source-pin HBI](policy/TRAINED_SOURCE_PINS.hbi) identifies **23 pinned original files** in the trained-GNN and Shannon-stage repositories, including all **nine published .pt checkpoints**. The Rust 1.81 [native source protocol](native/TRAINED_SOURCE_PROTOCOL.md) builds reproducible HBI/HBP receipt bundles, computes SHA-256 over the actual original bytes, validates Git blob/source identities, carries compact SH fingerprints and links every row into a verifiable HASH chain.

The initial hosted test, [run #44279556309](https://github.com/JesseBrown1980/P-NP/actions/runs/44279556309), passed native Rust tests, zero-warning Clippy, the original v1/v2 GSLGNN checkpoint SHA-256 checks, repeated byte-identical HBI/HBP receipts, four adversarial corruption controls, and independent archive verification. The native inference equivalence and unavailable private training material retain their source-defined HOLD statuses until separately verified; raw checkpoints are never executed in the accepted runtime.

---

## Exact ternary matrix witness (November 18, 2026)

[Phase-2 PR #29](https://github.com/JesseBrown1980/P-NP/pull/29) extends the native matrix third seat from aggregate comparison to **ordered, exact per-address reconstruction** using nine reversible `(S,d0,d1)` witness triples per 27-cell room. It preserves the 3/9/27/81 nested cascade and validates **1,080 triples** across 120 rooms. The new negative controls reject **42 balanced, sum-preserving ±1 faults** and **14 coordinated worker/reflection report faults**, alongside the original **42 single-report tests**. This redundancy is intentional verification storage, with original-source hashes and archived receipts supplying provenance.

The [first hosted Rust 1.81 certification and independent archive run](https://github.com/JesseBrown1980/P-NP/actions/runs/44277700671) passed both jobs. Historical trained-network weights and the full Omni/Host-8 restoration remain assigned to [issue #28](https://github.com/JesseBrown1980/P-NP/issues/28).

---

## Native ternary engine rebuild (November 18, 2026)

The [Rust 1.81 native ternary engine protocol](native/TERNARY_ENGINE_PROTOCOL.md) now uses [THE RELIC REDISCOVERY](https://github.com/JesseBrown1980/THE-RELIC-REDISCOVERY) as the exact reference for invariant free-center semantics, ternary prefixes, intermediate ×3 waves, nonuniform structural landings, and 27-triple center/difference reconstruction. The 3/27/81 n-ary structures are real source-derived dispatch lanes and matrix fabric widths. Balanced `−1,0,+1` trits represent false, unassigned, true in the SAT core.

**First native pass:** nine Rust-only source-derived strategies share one exact full SAT DPLL implementation and were verified on small SAT/UNSAT/PHP tests, with 2,304 exhaustive crosschecks; full [Rust 1.81 Clippy, byte-exact replay and independent archive](https://github.com/JesseBrown1980/P-NP/actions/runs/44275667495) PASSED. This is a fresh native correctness baseline; **historical trained GNN/FNN weights, full Omni/Host-8 architecture, source-equivalent Fischer/HXF algorithms, general cryptographic benchmarks, and a P=NP theorem remain pending**. The 54 legacy artifacts are still nonaccepted under the new software contract.

---


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
## Bitcoin SHA-256 challenge — seven SAT engines

The [source-frozen real Bitcoin genesis SHA256d SAT benchmark](docs/BITCOIN_SHA256D_7WAY_20261117.md) now tests the fourth structural HXF-1, original Yang–Mills Fischer, newly implemented Rime, byte-exact original ASOLARIA Fischer, MiniSat, CaDiCaL and PicoSAT on full Bitcoin double-SHA-256 Boolean circuits with 4, 8, 32 and **288 free header bits**. The largest CNF has **361,188 Boolean variables and 1,325,357 clauses**. The HXF-1 recognizers are inapplicable to SHA256d; its current general fallback reached HOLD on all four. The modern Rime adapter and three external solvers recovered the independently verified real nonce on the 4- and 8-bit cases; all seven reached HOLD on both 32- and 288-bit cases. **All 28 original-input results, reference gate models, SHA256d witnesses and 20 wrong-header controls passed independent acceptance**, with a re-downloaded SHA archive. [Accepted hosted run #44213748169](https://github.com/JesseBrown1980/P-NP/actions/runs/44213748169). The general P versus NP theorem remains open.

## RIME winding drills across all SAT and Bitcoin tests

The [378-result RIME wind/unwind SAT competition](docs/RIME_WIND_UNWIND_SAT_378_20261117.md) tested all 14 original SAT-array and four Bitcoin SHA256d cases under plain original-CNF representation, 27-point RIME/Fischer winding, and matched random signed-variable relabeling, for all seven engines. [Full hosted independent acceptance #44216855484](https://github.com/JesseBrown1980/P-NP/actions/runs/44216855484) passed: 89 SAT, 144 UNSAT, 145 bounded HOLDs; all 378 inverse clause mappings and SAT models checked. RIME winding did not improve completed counts over randomized controls in this first experiment. HXF-1's Hall recognition was disrupted by arbitrary signed winding, reducing 14/18 plain decisions to 11/18; its XOR path survived. The **N-nested infinite-agent** architecture and **MTP** forward/backward scoring are **not included** in this controlled representation experiment and remain separate research arms to evaluate.


## Nested-agent + MTP + Fischer SAT research

The new [MTP × N-Nest × Fischer 270-run ablation](docs/MTP_NNEST_FISCHER_270_20261117.md) tests 15 configurations of source-only forward/backward order-two scoring, the original N-Nest independent watcher law (3, 9, 27, 81 virtual leaves), and exact RIME/Fischer wind/unwind branch priorities. All 15 use the same HXF Hall/XOR fast paths and watched-literal fallback on **all 14 exact SAT-array and four Bitcoin double-SHA256 problems**. [Hosted run #44219809886](https://github.com/JesseBrown1980/P-NP/actions/runs/44219809886) accepted **270/270 receipts**: 45 SAT, 165 UNSAT, 60 bounded HOLDs, 10/10 planted N-Nest faults rejected, full source identity, independent SAT model verification and downloaded archive checksum PASS. All 15 configurations preserved the 14/14 exact structural decisions but held on the same four cryptographic challenges. Source-pinned original Fischer/Yang–Mills implementations unchanged. These are finite verified virtual agents and CNF-adapted frozen MTP scoring—not unlimited agent processes or the original text-trained MTP model.
