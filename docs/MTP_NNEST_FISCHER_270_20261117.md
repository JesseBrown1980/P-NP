# MTP × N-Nest × Fischer — 270 accepted SAT/Bitcoin outcomes

**Accepted on November 17 (São Paulo) / November 18, 2026 (UTC).** [Research PR #22](https://github.com/JesseBrown1980/P-NP/pull/22) merged at `d7e8936b5b063ba77dcdb2db90c8b71c1e5a6602`. The exact-head [GitHub Actions run #44219809886](https://github.com/JesseBrown1980/P-NP/actions/runs/44219809886) passed the original-input source freeze, independent 3/9/27/81 watcher tamper controls, 15 hosted competitor jobs, independent all-270 receipt referee, and a separately downloaded checksum archive acceptance.

**Final evidence:** GitHub artifact `14389463433`, `mtp-nnest-fischer-270-final-accepted`, archive SHA-256 `3949fe42c41489efd57766b4cdb87faffd66f652ddf026bb34d0a3bc28dbbb88`.

## What ran

The benchmark reused all 14 existing exact SAT-array CNFs (7 UNSAT pigeonhole, 3 SAT pigeonhole, 4 exact odd-charge Tseitin) plus the Bitcoin genesis SHA256d SAT CNFs with 4, 8, 32 or 288 unknown header bits. All 15 arms used the **same** unmodified HXF-1 structural Hall/XOR recognition and the **same** watched-literal DPLL fallback. They differed only in deterministic branch priorities. Every arm received 12 seconds per original input and at most 2,000 search nodes where fallback was reached.

- **MTP:** CNF-native, integer-only, local frozen order-two models, with forward and reversed-time training from alternate sampled source rows and held-out alternate rows used for candidate scoring. No SAT labels, nonce solutions or external trained corpora enter the model. The archived ASOLARIA MTP trained on text is a distinct earlier implementation.
- **N-Nest:** Trinary trees with 3, 9, 27, 81 virtual leaf agent/watcher pairs; each worker computes source-scoped integer counts while an independent watcher rescans its assigned source rows. Every internal node recomputes child sums, and an invalid child verdict blocks the apex. Ten injected false reports at levels 1–4 were rejected, with no prompted intervention. Virtual trees are bounded, not unrestricted autonomous process spawning.
- **Fischer/RIME:** The historical 27-point RIME number-theoretic transform and smooth Fischer sphere derive deterministic variable-order priors. Exact NTT inverse and sphere wind/unwind are checked. CNF clauses remain in their original literal gauge, keeping Hall and XOR fast paths recognizable.

Historical anchor: [N-Nest-Prime-INFINITE-SELF-REFLECT-AGENTS-NESTED](https://github.com/JesseBrown1980/N-Nest-Prime-INFINITE-SELF-REFLECT-AGENTS-NESTED), tree `307903651f56e8a28ef98ca36aef22ee8cda935c`. Original invariant: `node_green = (reported == watcher_recomputed_truth) AND all_child_green`. Historical ASOLARIA sources: [`players/rime_agents.py`](https://github.com/JesseBrown1980/ASOLARIA-UNIVERSE-SIMULATOR-with-gravity-light-time-space-color-rime-winding-drills/blob/main/players/rime_agents.py), [`players/rime_windunwind.py`](https://github.com/JesseBrown1980/ASOLARIA-UNIVERSE-SIMULATOR-with-gravity-light-time-space-color-rime-winding-drills/blob/main/players/rime_windunwind.py). The original Yang–Mills and historical Fischer implementation blobs remained unchanged.

## Matched 15-arm results

| Ablation arm | Completed / 18 | HOLD | Recorded fallback nodes | Preparation ms¹ |
|---|---:|---:|---:|---:|
| `baseline` | 14 | 4 | 6,000 | ~64 |
| `mtp` | 14 | 4 | 6,000 | ~127 |
| `fischer` | 14 | 4 | 6,000 | ~179 |
| `nest3` | 14 | 4 | 6,000 | ~140 |
| `nest9` | 14 | 4 | 6,000 | ~102 |
| `nest27` | 14 | 4 | 6,000 | ~124 |
| `nest81` | 14 | 4 | 6,000 | ~127 |
| `mtp_nest3` | 14 | 4 | 6,000 | ~161 |
| `mtp_nest9` | 14 | 4 | 4,000 | ~134 |
| `mtp_nest27` | 14 | 4 | 6,000 | ~128 |
| `mtp_nest81` | 14 | 4 | 6,000 | ~165 |
| `all3` | 14 | 4 | 6,000 | ~226 |
| `all9` | 14 | 4 | 6,000 | ~176 |
| `all27` | 14 | 4 | 6,000 | ~189 |
| `all81` | 14 | 4 | 6,000 | ~224 |

¹ Preparation totals come from the native receipts that actually finished within their process deadlines; timed-out processes do not contribute a full preparation receipt. Thus these numbers describe *recorded instrumented work*, rather than a complete, isolated CPU-cost comparison across independent GitHub hosts. The 4,000-node count for `mtp_nest9` reflects an additional time-based HOLD before exhausting its node cap on one Bitcoin case; it does **not** demonstrate a better completed decision.

Every arm obtained the same 14 exact source decisions: 3 SAT and 11 UNSAT, with no search nodes for the original Hall/XOR fast paths. Every arm held on all four Bitcoin inputs. At the whole-matrix level, **45 SAT + 165 UNSAT + 60 bounded HOLD = 270 accepted source-aligned results**. There were 2,000-node HOLDs on many nonce cases; the 288-bit Bitcoin-format header challenge reached time HOLD across all arms. No Bitcoin preimage completion improvement was measured.

## Verification

- 10/10 deliberately corrupted N-Nest worker/supervisor reports rejected at the correct subtree; clean 3-,9-,27-,81-leaf roots PASS.
- 270/270 sources and recorded outcomes checked against original CNF hashes; all completed SAT models independently replayed on the original clauses.
- All independently established PHP and odd-charge Tseitin UNSAT statuses remain consistent with exact mathematical source invariants. For every Bitcoin SAT claim, the referee demands the full genesis double-SHA256 hash and complete original-CNF witness; in this run, the Bitcoin outcomes were all HOLD.
- All 15 solver jobs completed successfully; independent referee PASS; final SHA-256 re-download PASS.

## Research interpretation

The outcome confirms that the three-layer architecture is **composable** without losing HXF-1's previous exact structural results. The first iteration's MTP/Fischer/N-Nest branch-order heuristics did not reduce the number of hard cryptographic HOLDs within the tested limits. The next discriminating test is a held-out collection of **general 3-SAT instances with valid witnessed solutions and independently checked UNSAT certificates**, with more than one trial and controlled process resource measurements, so changes in priority have room to matter. The theorem question P versus NP remains open.

Source and rerun protocol: [`experiments/mtp_nnest/`](../experiments/mtp_nnest/README.md), [hosted workflow](../.github/workflows/mtp-nnest-fischer.yml).