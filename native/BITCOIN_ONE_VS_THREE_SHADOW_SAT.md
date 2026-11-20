# Original Bitcoin SAT: one 2D shadow versus three 2D shadows

This is the controlled **branch-choice** experiment stacked on independently
accepted three-light Bitcoin RimeSphere PR #59. Prior original-source and
three-shadow streaming acceptance are preserved; PRs #57, #58 and #59
are not modified by this experiment.

## What is actually being tested

The operator's geometric method produces three orthogonal X/Y/Z signed
line-sum shadows from each 3×3×3 cube. The independently accepted source
pipeline verified **all 10,827,176 original Bitcoin RimeSphere rooms**,
with 19 independent projected scalar measurements plus 8 residual
source trits to reconstruct each signed 27-cell object exactly.

**The next computational question** is whether three orthogonal views
alter or improve real SAT branch selection. The research kernel
`native/src/shadow_selector.rs` now constructs a **live 27-variable
candidate room** from the genuine original Bitcoin CNF's signed occurrence
counts (positive count minus negative count, quantized to −/0/+) and the
solver's real ternary current assignment (assigned trits override
source-only signs). This is intentionally distinct from the immutable
source-byte RimeSphere used to authenticate and freeze the original
Bitcoin data.

The two experimental modes see identical original CNF and candidate
Fischer ordering:

- **One shadow** computes X-direction signed 3×3 line sums, and scores
  each open candidate by the magnitude of its X-direction pixel.
- **Three shadows** computes all X/Y/Z orthogonal projections, uses
  their three absolute pixel magnitudes as a summed candidate score,
  **and reconstructs the entire 27-cell room with the exact 8-trit
  source witness** before accepting the projected candidate.

Tie-breaks are the original Fischer order in both modes. Both preserve
the exact CNF, all unit propagations and both ± Boolean branches.
Neither classifier is trained on the known Bitcoin genesis nonce.

A third **direct** control uses the same authentic source, root GGUF
and Fischer order with no projected branch-selection overhead.

## What counts as a valid result

The first comparison is intentionally an **exact 2-node pilot** on
both full original SHA256d Bitcoin CNFs. It measures the first
post-propagation branch decision and the next actual DPLL work while
charging original-source parsing, GGUF verification, Fischer ordering,
source-sign initialization, per-room projected bytes, three-shadow
reconstruction, live matrix watcher bytes and wall time.

The two-node cap is a **work counter**, not a clock or an algorithmic
lifetime. The source solver itself has **no artificial wall timer**
in this pilot. GitHub limits the compute session, as all hosted
workers require finite resources. A node-limited outcome is reported
as `HOLD_NODE`, preserving exact counts; only a complete source model
may report SAT after independent original CNF and full SHA256d checks.

The native worker `native/src/bin/bitcoin_shadow_compare.rs` records
`first_order_candidate`, `first_shadow_candidate` and the
`first_shadow_score`, along with actual projection and live search
work. These values let us see exactly if the additional two 2D views
change a real original-source decision.

The independent standard-library referee
`research/bitcoin_shadow_selector_referee.py` opens the *already
accepted* original Bitcoin source and three-shadow GGUF for each case,
checks every receipt, projected bytes per room (9 vs 35), each signed
original-source root, the absence of nonce injection, and actual
source-only SAT models whenever one is returned. It independently
compares one-shadow and three-shadow first choices.

Workflow: `.github/workflows/pnp-bitcoin-shadow-choice.yml`.
The same accepted original artifact SHA pins are reused from runs:

- Original RimeSphere and original CNF:
  `44652775916` (independent pass)
- Exact three-light original RimeSphere and GGUF:
  `44725165515` (independent pass)

## Evidence boundary

These projections may improve decisions, increase overhead, or have
no decision difference. Both a different choice and a faster wall time
are **empirical questions**. Rank-19 three-shadow reconstruction alone
does not establish a SAT efficiency gain, and a two-node pilot does
not establish an asymptotic complexity result.

Once the pilot passes, the next test can raise the node-work ceiling,
and an independently restartable full DPLL frontier can be frozen to
GGUF/RimeSphere for **continuous Fischer play** across compute sessions.
That will preserve the user's intended clock-free logical computation.
