# Starting evidence ledger (2026-11-17)

Source of record: [P-NP provenance](PROVENANCE.md). All experiments in this table are finite algorithmic tests.

| Source test | Positive acceptance | HOLD / counterexample |
|---|---|---|
| Integrated source-frozen 11-case SAT | 7 independently reproduced exact model counts | 4 search budget HOLDs; original Fischer static ordering used more branch nodes than fixed order on three mixed cases |
| 21-case held-out deterministic DIMACS | 20 independently replayed exact #SAT counts; 21/21 MiniSat/PicoSAT SAT decision agreement; 17 SAT and 4 UNSAT | Planted n=30 count HOLD at 20,000 nodes; two ordering counterexamples |
| Independent acceptance hardening | All three SHA chains, rank and recognized/free/residual fields; 13 altered receipt fields rejected; external SAT assignments checked including planted-30 HOLD | No general asymptotic bound claimed |
| Archive acceptance | Second hosted job downloaded the GitHub artifact and verified SHA manifest and 21 case transcripts | Host/installer and external comparator cost must remain separately measured |

## Selected numerical receipts

- Complete XOR n=24,48,96: exact counts 4,1,4; residual search at one node; defined metered work 11,121 / 43,182 / 187,890.
- Mixed n=12,24,36: exact counts 14,200,1,824; Fischer-inspired search nodes 51/645/8,591 versus fixed-order 61/1,081/10,075 on this particular held-out suite.
- Generic random 3-CNF n=12,18,24,30: exact counts 2,7,241,0; exact independent count replay passed.
- Hub-connected n=12,18,24,30: exact counts 10,42,144,128.
- Explicit contradictory n=12,24,36: exact counts zero.

These metrics preserve the negative controls and counterexamples. The next milestone is to run the transferred source and all independently hosted gates in this **new** repository. No result on this page should be interpreted as a proof about the general P versus NP question.


## First adaptive selector — exact verified, choice heuristic under study

[P-NP PR #6](https://github.com/JesseBrown1980/P-NP/pull/6) / [hosted run #44193491896](https://github.com/JesseBrown1980/P-NP/actions/runs/44193491896) PASS: 21 source-frozen cases, 20 exact model counts and one budget HOLD, original-CNF independent verification, ten forged selector receipts rejected, and complete SHA-256 artifact re-download check.

- The selector compares raw CNF with verified XOR-affine reduction and two variable orders; every 96-node probe, unsuccessful full attempt, fallback and watcher is charged.
- Selected outputs: 6 algebraic, 14 raw-CNF, 1 HOLD; four fixed-order runs ran out of budget and were rescued by the recorded Fischer-inspired fallback.
- **Counterexample to optimal selection:** on mixed n=12/24/36, affine-fixed needed 61/1,081/10,075 nodes versus affine-Fischer 51/645/8,591. The selector achieved exact correct counts but its probe heuristic made the more expensive choice.
- Pure XOR n=24/48/96 exact counts 4/1/4 required one final search node each, though probing all candidate representations increased the total reported work to 25,307/71,327/251,582 defined units.

Full source, proof and work receipts in the hosted archive; methodology in [SELECTOR_V0.md](SELECTOR_V0.md). Generalized scaling and strong external #SAT competitiveness remain OPEN.

## Frozen Yang–Mills vs independently implemented Rime Fischer competition

[Research PR #9](https://github.com/JesseBrown1980/P-NP/pull/9), [initial hosted acceptance #44198991467](https://github.com/JesseBrown1980/P-NP/actions/runs/44198991467): **PASS** for 21 same-source paired 3-CNF cases, independent SHA/affine-rank/counted-model witness referee, 11 corrupted result rejections and independently downloaded artifact acceptance. Both contestants gave 20 exact counts and one budget HOLD. On the 20 complete cases, the experimental Rime-inspired branch order won 6, lost 7, tied 7 by *search nodes*. Sum over all 21: frozen 75,388 nodes, Rime 72,414 nodes (both include 20,000 on the common HOLD).

Each arm was isolated with alternating process order. Observed aggregate subprocess elapsed: frozen 7.542 s; challenger 2.874 s. A separate fixed-order baseline is executed by the frozen solver, so this raw elapsed ratio is **not a matched-kernel performance ratio**. Median peak RSS: 18,636 versus 18,724 KiB. The original discrete-logarithm Law-14 PH prime-power lifting and CRT were independently reimplemented and passed all 21 math cases against full-order Shanks BSGS; these are **separate mathematical problem classes**, not general 3-SAT complexity evidence.

Detailed family table, source pins and acceptance limitations: [Competition protocol](../experiments/competition/README.md). Next: common-cost single-order comparison, repeated runs and unseen SAT classes. All counterexamples retained.
## Byte-identical historical ASOLARIA Rime Fischer replay (2026-11-17)

The original historical four source scripts from ASOLARIA Universe Simulator commit `3f9f42dab6247c40b30921b253e3f73c9e942ca5` are now archived under `historical/asolaria-2026-07-21/` with exact Git blob SHA checks. In particular, original `players/rime_fischer.py` is Git blob `cbdf20564f1b38f3ddd08062ead8174ecc3d236a` and original `players/rime_sphere.py` is `449ac82b82223349ca8fc41d8e9b005fd3beda06`.

[Hosted run #44202756746](https://github.com/JesseBrown1980/P-NP/actions/runs/44202756746): **PASS** for historical direct 5-target replay, original multiprocessing cluster, original 27-Fischer script, 21 independent original-source exact discrete logs against full-order BSGS, and 21/21 agreement on the *same targets* with the newly reimplemented Rime PH kernel. Altered exponent/SHA negative controls rejected. Both source-identity and the separately downloaded artifact manifest/required four Git blobs passed. Published artifact ID `14382716405`, SHA-256 `42c19481d933e5af05e354bde81db1e1619604c507a42430d368c7d3b7cae136`.

The 21 historical-versus-new elapsed samples sum to 508,575 ns and 437,453 ns within this single hosted run, respectively. These sub-millisecond aggregates have not been repeated or calibrated; no timing advantage is accepted. This is a mathematical/source equivalence test of classical Pohlig–Hellman + Shanks + CRT, distinct from the previously verified 21-case SAT/#SAT Yang–Mills-versus-Rime heuristic competition. The original source was never edited. See [historical provenance and test contract](../experiments/historical/README.md).

## Hard SAT six-arm comparison accepted (November 17, 2026)

[Full original-source 24-case outcome table](HARD_SAT_SIX_WAY_20261117.md) and [hosted run #44206197812](https://github.com/JesseBrown1980/P-NP/actions/runs/44206197812): six contender jobs, exact source/receipt referee, real 100-flip corruption rejection and published-archive download PASS. PHP(21,20), 420 variables and 4,221 clauses: all three native adapters HOLD at 2,000 search nodes; MiniSat, CaDiCaL, PicoSAT HOLD at 50 seconds. A connected 128-vertex odd-charge Tseitin 3-CNF: three native exact XOR contradiction certificates at rank 127, zero search nodes; all three external solvers HOLD at 50 seconds. PHP(8,7): native HOLD, all three external UNSAT, with CaDiCaL emitting a separately verified DRAT proof. PHP(4,4): all six SAT, original-CNF witnesses checked. Distinct search-node and wall-clock budgets prevent equal-cost speed conclusions; P=NP OPEN.


## 84-case SAT array accepted, November 17, 2026

[Full 14-input × six-competitor result table](SAT_ARRAY_84_RESULTS_20261117.md); [hosted acceptance run #44208774230](https://github.com/JesseBrown1980/P-NP/actions/runs/44208774230): 84/84 original-CNF and receipt verifications, 66 completed decisions, 18 budget HOLDs, eight CaDiCaL DRAT certificate replays and 20 forged SAT-witness rejections. On the seven completed PHP tests, matched common DPLL kernel search nodes total 899 (Yang–Mills) versus 516 (new/historical Rime), 42.6% fewer for the Rime ordering. On four odd-charge Tseitin contradictions, all three research models certified exact rank inconsistencies with zero search nodes. P versus NP remains open.

## Fourth structural engine HXF-1: 14/14, source-authenticated (November 17, 2026)

[Independent method, 61-case results and original-source proofs](../experiments/fourth/README.md); [Actions run #44210600049](https://github.com/JesseBrown1980/P-NP/actions/runs/44210600049). Fourth Hall/XOR/fallback engine obtained 14/14 complete exact decisions using **zero search nodes**, compared with 11/14 decisions and 6,899 (Yang–Mills), 6,516 (new Rime), 6,516 (historical Rime) total search nodes under matched 2,000-node caps. An independent referee replayed 56 Hall or explicit GF(2) contradiction certificates from the original CNFs, including 42 blinded variable/clauses permutations; five structurally altered controls went to bounded generic DPLL and passed independently. Full source/receipt archive hashes independently re-downloaded and accepted. All original Fischer sources unchanged; class-specific structural improvement, not a general P=NP theorem.
## Bitcoin genesis SHA256d seven-way cryptographic SAT benchmark (November 17, 2026)

[Full results and independent original-CNF evidence](BITCOIN_SHA256D_7WAY_20261117.md); [hosted run #44213748169](https://github.com/JesseBrown1980/P-NP/actions/runs/44213748169). Bit-exact SHA256d SAT circuits with authentic genesis header and published 256-bit target: nonce4 (236,344 vars/867,665 clauses), nonce8 (236,931/869,855), nonce32 (239,112/877,879), Merkle-root-256 + nonce-32 (361,188/1,325,357). All original gate models were independently checked against hashlib and original full CNF, including alternate header inputs. In 28 source-frozen original-CNF solver tests, HXF-1 completed 0/4 (4 HOLD), Yang–Mills adapter 1/4, modern Rime PH adapter 2/4, historical byte-original Rime 1/4, MiniSat 2/4, CaDiCaL 2/4 and PicoSAT 2/4. Every SAT output recovered nonce 2083236893 and verified the complete genesis header hash. Every arm held at 32 and 288 free bits. Referee PASS: 28/28 receipts, 10 verified SAT models, 18 explicit time HOLDs, 20 wrong-header rejections, independent published checksum archive PASS. The research algorithms used distinct SAT fallbacks/time limits, so no equal-cost speed ranking is inferred; P=NP remains OPEN.