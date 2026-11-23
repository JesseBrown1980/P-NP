# Original eight-square signed-counting identity — independent finite gate

Status: **source inspected; reproducible finite certificate implemented**.
The hosted Rust 1.81 / published-evidence checks are the acceptance authority.
All tests are classical exact-integer computations, not a claim of a
polynomial-time SAT solver or a working large-scale implementation of the
subexponential algorithm from the paper.

## Original source and exact formula

Claude (Anthropic), *A 2^{o(n)} Time Algorithm for 3-SAT*,
Hexagon 2610.00577v1 (2026-10-27),
https://hexagonmath.org/pdf/2610.00577

The 3-bit block formula is the paper's **Equation (12)**; the
ordered sign is given in **Section 4, Equation (27)**.
For x,y,u,v in {0,...,7}, bitwise XOR denoted ^,

    2 [x=v][y=u]
      = [x^y^u^v=0] *
        (sigma(x,y)*sigma(v,u) + sigma(x,u)*sigma(v,y)).

Write p=(p1,p2,p3) and q=(q1,q2,q3) with the most significant
bit first; then

    sigma(p,q) = (-1)^phi(p,q)
    phi(p,q)  = sum_{1<=i<=j<=3} p_i*q_j
                + q1*p2*p3 + p1*q2*p3 + p1*p2*q3.

The exact ordering of sigma arguments matters: sigma is
**asymmetric** on 42 of the 64 ordered three-bit input pairs.
This code uses precisely this formula and returns signed values
in {-1,0,+1}; the "Black" and "White" leaves here are merely
names for the two exact signed terms R and C, while the Brown
leaf stores the common mode with both differences using the
original Relic native freeze/rebuild routine.

The identity is the historical Degen–Graves eight-square identity,
not a new theorem belonging to this research project. The paper
uses it as one ingredient in a far larger recursive algorithm.

## 4,096-case exhaustion and signed cancellation

- Exhaustively check 8^4 = 4,096 inputs (x,y,u,v).
- 64 are matching crossings [x=v][y=u], each with R=C=+1.
- Of the 512 XOR-admissible inputs, the other **448** produce
  nonzero opposite terms R=-C and therefore cancel exactly.
- Each of R and C has **288 positive** and **224 negative** values.
- Verify full signed Relic triples [R,C,0] reconstruct correctly
  in all 4,096 cases, including the 448 cancellations.

## Rank tests: actual mathematical benefit of two orientations

Construct the complete 64-by-64 integer matrices R and C.
Rows/columns are grouped in two ways:

    row split    : (x,y) | (u,v)
    column split : (x,u) | (y,v).

The expected ranks are:

| Signed term | Row split | Column split |
|---|---:|---:|
| R = sigma(x,y) sigma(v,u) XOR-filtered | 8 | 64 |
| C = sigma(x,u) sigma(v,y) XOR-filtered | 64 | 8 |

A fixed prime-field elimination over GF(1,000,081) gives exact
lower bounds on the rational ranks. The paper's explicit
eight-state factorizations (Equation (30)) give complementary
rational rank upper bounds for the two rank-8 cells. For the
full-rank cells, modular rank 64 proves rational rank 64.
The rank measurements are a concrete structural advantage of
choosing complementary *contraction orders*, not a claim that
running both orders always saves work.

## #3SAT finite comparison

Input: 12 real Boolean variables grouped into x,y,u,v,
plus crossing constraints x=v and y=u; arbitrary remaining
3-CNF clause factors on those 12 variables.

For each of **35** independent deterministic CNF fixtures
(8 fixed including contradictory-SAT, 27 source-seeded cases):

1. Direct exhaustive counter: 4,096 bit assignments, test
   the ordinary 3-CNF and crossing equality.
2. Direct equality-pruned counter: only 64 assignments with
   (u,v)=(y,x). This is the efficient small-instance baseline.
3. Signed tensor counter: 4,096 assignments, count each
   clause-satisfying bit assignment with the two signed terms;
   numerator R+C must be even, nonnegative and equal to twice
   both direct counts.

The finite 35-fixture corpus has **1,032 total satisfying
assignments** across its cases, with **34 SAT** and **1 UNSAT**
fixtures. The sum of all nontrivial witness-cancellation events
across the corpus is **6,897**. These values were independently
calculated by a separate Python arithmetic implementation before
the hosted Rust run; the committed source uses no Python runtime.

The direct brute and signed scans each evaluate 143,360 candidate
bit assignments for this 35-case corpus. The equality-pruned
baseline evaluates only **2,240** assignments. As expected,
the signed rewriting alone costs more work on these small
instances than the obvious equality-pruned baseline.

## Evidence boundary and research continuation

The paper's asymptotic improvement requires structured
factor-network rewrites, balancing, grouping, and the recursive
cost recurrence. This PR implements **only one exact local rewrite**
and small finite counts; it does not implement the full recursive
algorithm or claim asymptotic performance.

A next gate may build explicit small graph/clique tensor networks
and measure the rank-aware row/column contraction cost against
a common optimized variable-elimination order, including signed
summation and Brown witness bookkeeping.

Verification:
- exact 4,096 input identity and 4,096 Relic witnesses
- 4 exact 64-by-64 matrix ranks
- 35 #3SAT equal-count fixtures
- malformed three-bit inputs, malformed clause and asymmetric
  sign negative controls
- mandatory repository Rust 1.81 offline, strict Clippy
  and independent published-evidence acceptance
- original \`main\`, original solver, original Omega operators,
  and all other open drafts unchanged.
