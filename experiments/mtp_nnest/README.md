# MTP × N-Nest × Fischer: exact-source SAT ablation

Third-stage follow-up to the [378-result RIME winding comparison](../../docs/RIME_WIND_UNWIND_SAT_378_20261117.md). Unlike RIME-only signed literal relabeling, this version preserves the original Hall/XOR constraint structure and uses Fischer, frozen MTP and virtual N-Nest agents **only to guide branch ordering on the original exact source**. Search is performed by the same original `experiments/bitcoin/watched.py` kernel for every arm after the unchanged `experiments/fourth/engine.py` Hall/XOR structural path. Recovered SAT witnesses are checked against the entire original CNF; Bitcoin additionally checks full double SHA256.

## Historical implementation provenance

- [N-Nest-Prime original source](https://github.com/JesseBrown1980/N-Nest-Prime-INFINITE-SELF-REFLECT-AGENTS-NESTED) at source tree `307903651f56e8a28ef98ca36aef22ee8cda935c`: `node_green = (reported == independent_watcher_truth) AND all child_green`, agent and watcher pair per node, tampered descendants fail upward. [Depth-3 verification](https://github.com/JesseBrown1980/N-Nest-Prime-INFINITE-SELF-REFLECT-AGENTS-NESTED/blob/main/nest-depth3-verify.cjs) and [prime-depth verification](https://github.com/JesseBrown1980/N-Nest-Prime-INFINITE-SELF-REFLECT-AGENTS-NESTED/blob/main/nest-depthN-prime-verify.cjs).
- [ASOLARIA frozen four-agent model](https://github.com/JesseBrown1980/ASOLARIA-UNIVERSE-SIMULATOR-with-gravity-light-time-space-color-rime-winding-drills/blob/main/players/rime_agents.py): MTP model frozen from forward order-two and reverse-time order-two data, with measured x1/x3/x27 lookahead; [winding drill](https://github.com/JesseBrown1980/ASOLARIA-UNIVERSE-SIMULATOR-with-gravity-light-time-space-color-rime-winding-drills/blob/main/players/rime_windunwind.py) used 27 sectors and exact Fischer wind/unwind.
- This test is an exact-integer **CNF-native adaptation** of those rules. It freezes a new local, unlabeled order-two bank on the input clause-token stream (even sampled rows) and scores branch candidates from the disjoint odd sampled rows. The original text-trained model weights and GNN are not loaded; there is no hidden result label, Bitcoin nonce or independently supplied answer in the scoring input.
- N-Nest trees have 3, 9, 27, or 81 **virtual leaf agent/watcher pairs** (depths 1,2,3,4); each agent computes exact integer shard scores, each watcher independently scans its shard, and parent node reports are recomputed with fail-closed child propagation. All 10 planted faulty reports spanning every nest depth are blocked. This is bounded execution demonstrating nested corrective gates—not an assertion of physically infinite, autonomous concurrent processes.
- Fischer source-sha-seeded 27-point NTT glyphs and exact sphere wind/unwind provide deterministic branch priors. All input variables retain their original meanings, preserving HXF-1's Hall and XOR recognizers.

## 15 matched-kernel arms

| Arm | MTP | Fischer RIME | N-Nest leaves |
|---|:---:|:---:|---:|
| baseline | — | — | — |
| mtp | yes | — | — |
| fischer | — | yes | — |
| nest3, nest9, nest27, nest81 | — | — | 3, 9, 27, 81 respectively |
| mtp_nest3, mtp_nest9, mtp_nest27, mtp_nest81 | yes | — | 3, 9, 27, 81 respectively |
| all3, all9, all27, all81 | yes | yes | 3, 9, 27, 81 respectively |

Each arm processes the same 14 archived SAT-array cases and four complete Bitcoin genesis double-SHA256 CNFs (including the 32-bit nonce and 288-bit Merkle-plus-nonce challenge), giving 15 × 18 = 270 original-CNF observations. Each receives 2,000 DPLL search nodes or 12 seconds end-to-end per input, whichever comes first. Structural fast-path recognition and fallback solver are identical across all 15 arms; only deterministic candidate branch-priority preparation differs. Score, MTP freeze, N-Nest watcher, Fischer wind/inverse and process times are individually recorded.

## Verification and controls

Hosted `.github/workflows/mtp-nnest-fischer.yml`: original Yang–Mills and historical Fischer Git blob check; exact-arithmetic AST audit; 3/9/27/81 corrective trees green and 10/10 planted faults fail apex; 18 frozen source CNFs; independent full original-CNF SAT witness and Bitcoin SHA256d verification; mathematical truth of pigeonhole and connected odd-charge Tseitin UNSAT checked from source; all HONEST HOLDs; all 270 source and timing receipts; final independent re-download and SHA-256 manifest check.

Interpretation: when every arm completes an instance structurally, differences can only be reported as **coordination overhead**. For formulas requiring search, compare actual search nodes and verified decisions at matched budgets. An ablation improvement is evidence of that tested configuration, not a P=NP result.