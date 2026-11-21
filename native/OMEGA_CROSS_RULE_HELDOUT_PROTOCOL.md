# Omega cross-rule held-out prediction — native pilot

## Existing sources and new question

This pilot reuses the mainline Rust 1.81 Relic center/prefix implementation
('native/src/relic.rs') and the exact 40-operator original Omnibit catalog
('native/src/omega_operator_catalog.rs', pinned to upstream Git blob
'1535e158b21b1c2b9ece4dce84e0e9779498ec19').
It preserves the current six-view, third-seat, and Brown Ledger research
branches without substituting a new Bitcoin solver.

Question: can a quadratic ternary transition be inferred from five reversible
RNQ presentations of 20 training states, then correctly forecast the entire
transition of seven unobserved states in a sixth presentation?

## Protocol

- Source states: all 27 ordered ternary triples; Relic invariant center (0,1).
- The source transition is an explicitly fixed **synthetic quadratic map**
  over F_3, with 10 monomials (constant, linear, three squares, three cross
  terms), and 3 output coordinates. It is not a learned historical GNN law.
- Whole-state holdout: ordinals divisible by four (7 triples); 20 other
  triples are training inputs.
- Input and output are transported through five original reversible RNQ
  operator IDs 0–4. The sixth operator ID 5 is never present in fitting.
- Fit via exact F_3 Gaussian elimination with full rank required, then compare
  *every* training observation against its independently decoded state.
- Fit the same polynomial from view 0 alone. Exact equality of both fits
  is a mandatory **no-new-information control**.
- Predict each held-out transition before generating the withheld oracle
  result. Also check the complete original 40-operator catalog on all seven
  withheld inputs, with independent inverse.
- Reject changed one-view outputs, malformed digits, unknown operators,
  rank-deficient training matrices, and common-mode-only reconstruction.
- The existing native unit-test CI executes the pilot. No new CI workflow,
  engine runtime, Node, JSON, floating-point operations, or private data.

## Intended exact census (verify from hosted CI before acceptance)

| Metric | Exact fixture count |
|---|---:|
| Distinct training states | 20 |
| Five-view training pairs | 100 |
| Completely withheld source states | 7 |
| Sixth-view predictions | 7 |
| Original 40-operator held-out portability checks | 280 |
| Training symbol payload, 8 bits per symbol, before framing | 600 bytes |
| Canonical unique input/output symbol payload before framing | 120 bytes |
| Learned polynomial coefficients | 30 trits |

The 600 and 120 bytes are **raw symbol payload measurements only**,
not a completed Shannon ledger or a compression ratio. A fully charged
archive must add labels, framing, source and operator code, trained
coefficients, witnesses, hashes, verifier, and compute/storage cost.

## Interpretation

Five reversible presentations reuse the same twenty source transitions.
Their information rank is determined by the unique source states, and one
bijective view already contains all of their information. Correct predictions
on seven unseen inputs establish exact generalization **within the declared
degree-two model class** on this bounded synthetic problem. Portability
across 40 operators establishes representation consistency, not six
independent physical universes, learned cross-language semantics,
a cryptographic preimage inverse, a polynomial worst-case SAT bound,
or P=NP.

Next gate: an independently frozen source transition not generated from the
same assumed polynomial family, matched held-out controls against existing
direct / one-shadow / three-shadow / nine-cat Bitcoin frontiers, and a
full Brown Ledger including residuals, identities, and runtime costs.
