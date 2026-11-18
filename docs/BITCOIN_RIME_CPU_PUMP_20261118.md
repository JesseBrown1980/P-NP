# Bitcoin genesis SHA256d: frozen Rime/Pump–Funnel native comparison

**Research acceptance (first hosted run, 2026-11-18):** [PR #56](https://github.com/JesseBrown1980/P-NP/pull/56), [GitHub Actions #44417270526](https://github.com/JesseBrown1980/P-NP/actions/runs/44417270526). Both hosted jobs PASS, with strict Rust 1.81 build, no third-party crate, unchanged source files, original SHA256d and full-CNF witness validation.

## Input and question

Use the actual four frozen November 17 Bitcoin genesis SHA256d CNF files from the original seven-way experiment: nonce4 (236,344 variables, 867,665 clauses), nonce8 (236,931/869,855), nonce32 (239,112/877,879), and free Merkle-root + nonce (361,188/1,325,357). Read the original archived CNFs from hosted run #44213748169, or regenerate them with its unchanged source generator. Verify all four exact original SHA-256 fingerprints before any branch work.

The question is whether immutable Four-Axes Rime GGUF tensor address reads, optionally through batched CPU HTTP-0, provide useful branch ordering on authentic SHA256d SAT workloads, with all computation and verification costs charged.

## Common rules

All arms use **the same native watched-literal DPLL implementation**; Boolean assignment states are signed ternary (-1,0,+1). Only branch order changes:

- DIRECT: unconstrained Bitcoin header input variables in source order
- FISCHER: modular Fischer-address-ordered original free header input variables
- RIME_DISK: deterministic ordering by source-authenticated GGUF tensor values, read on CPU
- PUMP_FUNNEL_BATCH: three complete separate solver attempts (DIRECT, FISCHER, batched HTTP-0 Rime), then compare their checked reports. Accept a SAT result only when a complete original-CNF assignment and independent SHA256d witness pass; allow other routes to return HOLD

Each case has identical node/time limits across the individual routes: nonce4 64 nodes/8 seconds; nonce8 512 nodes/12 seconds; nonce32 and merkle_nonce288 96 nodes/5 seconds. Any UNSAT response is rejected, since the original Bitcoin genesis source provides a mathematically known SAT witness. Honest HOLD_TIME or HOLD_NODES is retained.

Source SHA/parse time and GGUF preflight are recorded separately from each arm's prep+solve+verification wall. The Pump–Funnel total includes all three path computations. Source priority values are deterministic and **not trained Bitcoin SAT guidance**.

## First hosted Bitcoin results

| Original CNF | DIRECT | FISCHER | RIME_DISK | PUMP_FUNNEL_BATCH |
|---|---|---|---|---|
| nonce4 | SAT 0.260613s | SAT 0.203343s | **SAT 0.181571s** | SAT 0.534754s |
| nonce8 | HOLD_TIME 12.004611s | HOLD_TIME 12.002679s | **SAT 3.125503s** | SAT 26.623601s |
| nonce32 | HOLD_NODES 1.810777s | HOLD_NODES 2.016452s | HOLD_NODES 1.966503s | HOLD_NODES 6.379097s |
| merkle_nonce288 | HOLD_NODES 0.031664s | HOLD_NODES 0.029429s | HOLD_NODES 0.034850s | HOLD_NODES 0.096359s |

**6 accepted SAT / 10 bounded HOLD**, 16/16 reports source-bound, all SAT recover nonce **2083236893**. On nonce8, RIME_DISK found the correct original genesis source in 97 nodes and 56,367,474 watched-literal checks. DIRECT and FISCHER reached 12-second HOLD after 430 and 395 nodes. On nonce4, RIME_DISK was approximately 1.44 times as fast as DIRECT in this single hosted comparison.

This is a measured variable-order effect on these fixed SHA256d CNFs. Future acceptance requires exact-source repeat trials, controlled host variance, and broader free-input and blinded instances. General P-versus-NP complexity remains an open research question.

## Replication

The next hosted Actions run is triggered by adding **this documentation file only**. No executable Rust code, source hash, source generator, benchmark limit, or GGUF content changes between first execution and replication. Compare per-case solution status, search nodes and literal checks first, then wall-clock values from independent hosted runners.
