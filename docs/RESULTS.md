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
