# HXF-1: fourth structural SAT research engine

**Role:** new source-blind solver that chooses certificate-producing representations from the actual clauses, rather than tuning each of the three Fischer engines separately. The original Yang–Mills, new Rime and byte-identical historical ASOLARIA Fischer code are unchanged.

## Architecture

**Hall recognizer:** From the CNF itself, recognize disjoint all-positive requirement groups and negative binary conflicts whose connected components are complete cliques. This gives a verified partition of mutually exclusive slots. A bipartite matching over requirements and slots constructs a true satisfying assignment, or an explicit Hall-deficient group subset with fewer neighboring slots certifies UNSAT. Every original clause must appear in the reconstructed structure.

**Parity recognizer:** Group 3-CNF clauses by triples; recognize four-clause exact parity constraints only after exhaustive local truth-table verification. Perform exact-integer GF(2) elimination with an accumulated source-row provenance bitset; an inconsistent system yields explicit row indices whose XOR is `0=1`. Consistent affine systems yield exact original-CNF SAT assignments.

**Fallback:** If structural recognition fails, bounded original-CNF unit-propagating DPLL provides exact SAT witnesses or fully completed decision searches. A 2,000-node cap gives an explicit HOLD.

## Source-frozen hosted acceptance

[GitHub Actions run #44210600049](https://github.com/JesseBrown1980/P-NP/actions/runs/44210600049): both test/replay job and separately re-downloaded archive verification PASS.

Test inputs: 14 unchanged cases from `experiments/array/`, 42 deterministic variable and clause permutations (three per case), and five deliberate near-match/overlap/unit-conflict/broken-XOR controls. The engine receives only anonymous CNF and no test names or expected answers.

| Measure | HXF-1 | Yang–Mills Fischer | Modern Rime | Historical Rime |
|---|---:|---:|---:|---:|
| Completed original 14 | **14** | 11 | 11 | 11 |
| Original HOLDs | **0** | 3 | 3 | 3 |
| Original SAT search nodes | **0** | 6,899 | 6,516 | 6,516 |
| Independent source-blind renumbered/reordered exact receipts | **42/42** | not in this round | not in this round | not in this round |

The fourth engine's 14 original cases were completely certified from clauses, with no search nodes: 10 via Hall slots/matching and 4 via exact parity. It also passed 42 variable-renamed and reordered instances and routed all five broken-structure controls to generic DPLL. Independent referee accepted all **56 exact structural certificates** and exhaustively checked small fallback controls. Exact source SHA and proof/receipt manifests were published and downloaded for a second check.

One hosted timing sample for PHP(21,20): HXF-1 solver-internal `7,514,507 ns` (~7.5 ms), with a completed and independently replayed Hall UNSAT certificate; all three earlier approaches reached their search cap on that case. Timing is a single-run observation on a recognized structured family; process-launch and instrumentation differences prevent a general speedup claim.

## Reproduce

`python3 -s -B experiments/fourth/sweep.py build/fourth`

`python3 -I -B experiments/fourth/verify.py build/fourth`

`python3 -I -B experiments/fourth/compare.py build/fourth`

Run workflow `.github/workflows/fourth-engine.yml` for the original Git blob gates, source SHA checks and published artifact acceptance.

**Research interpretation:** HXF-1 outperformed all three incumbents on this 14-case fixture array by exploiting explicit, independently checkable structure. Hall recognition and exact linear parity elimination apply to well-defined constraint families. Other CNFs proceed through the bounded fallback; the general P versus NP problem remains OPEN.