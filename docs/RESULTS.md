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
