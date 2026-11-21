# Nine Shadow Cats under Brown's paid-Shannon ledger — independent test

**Status:** Experimental source-observation mapping, tested independently from
the accepted Bitcoin solver in stacked draft PR; actual hosted CI receipts
determine acceptance. No existing source, SAT solver, original CNF, GGUF
or previously certified frontier is modified by this research slice.

## Source basis and the unresolved operator detail

The actual `COMBINED-BOOK-OF-LAWS` Book VIII.A.1 quotes the Rule of One:
**Shannon is the moving universe; Fischer is the frozen universe;
either way the RimeSphere has an information cost.** Book IV.4/IV.7
establishes that a prior is charged when decoder and encoder need to hold
it. Book IX.A.2 distinguishes three shadow keys from an additional
counter-spinning wave-key pair. Book IX.A.5 records six toruses around
each central shadowcat zone. Law 36's four-point experiment arranges
three directions (forward, bilateral, backward) at three radii
(1, 2, 3), with the bilateral observer at a different information
availability boundary. RIME laws 6, 13, 15, 19, 20 and 22 require
shared-center conservation, source-sphere addressing, frozen playback
and paid third-channel closure.

**Operator clarification:** three Schrödinger boxes × three cats = nine
at the first level, with higher 27-fold nesting possible. The earlier
historical records sometimes use three boxes × nine cats. Both statements
are retained. Neither establishes exact torus dynamics, glyph indexing,
wave-key serialization or physical GPU timing.

## The precise experimental definition

The first source-only nine-cat implementation is **a proposed mapping of
Law 36, not the canonical historical shader or physical torus**:

- Three boxes: forward, bilateral, backward; three depths per box:
  r=1,2,3 = nine signed scalar observations.
- A source room uses 27 cyclically addressed balanced trits.
- The six directed torus links are positions center ± 1, ± 2,
  ± 3 modulo 27. At each depth, forward sums the nearest `r`
  source trits; backward sums the opposite ones; bilateral sums
  the `r` pairwise products. Bilateral reads both sides from a
  **previously frozen source bank**; it is not available as
  unseen future streaming input without a separate payment.
- The two opposite wave keys are exact order-three rotations
  R(x)=((x+2) mod 3)-1 and R², verified R³=I on every source trit.
  They are reversible coordinate choices, not newly observed
  independent plaintext.
- The Brown ledger charges every held source bank, sourced sign-index
  allocation, frozen closure, and serialized observer snapshot. Each
  cat's signed output could be transmitted as a two-byte signed integer.
  Nine per inspected candidate = 18 bytes; 27 candidates = 486 bytes,
  before transport and decoder. Recompute-on-demand avoids persistent
  observer payload but still reads the paid bank and spends CPU cycles.
- **Negative control:** the nine local scalar observations do not
  identify all 27 source trits; many altered remote sources have identical
  observations. No arbitrary third or cryptographic preimage is
  inferred from these nine values alone.

## Actual Bitcoin measurements

Both source-root authenticated original SHA256d SAT CNFs are read in full
from [frozen real-source run #44652775916](https://github.com/JesseBrown1980/P-NP/actions/runs/44652775916),
with immutable SHA256:
- `nonce32`, 239,112 variables and 877,879 original clauses,
  SHA `6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3`.
- `merkle_nonce288`, 361,188 variables and 1,325,357 original clauses,
  SHA `601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b`.

The signed index is computed from **every original clause literal**.
The nine cats inspect 27 consecutive variable addresses anchored at
the previously certified first Fischer choice, for each source.
This is a **source-centered observation comparison** and **not the
full original Fischer-ranked DPLL candidate list**: it cannot claim
solver node speedup or P=NP. A second standalone Python referee
re-parses the original CNFs, re-computes the signed bank and all
243 nine-cat sample observations for each case, and reproduces
the selected candidate and source bank SHA.

The public Bitcoin genesis 80-byte header provides a byte-exact,
source-authenticated ternary round trip and Law-22 closure test.
After padding to 81 bytes, two retained thirds (54 B) plus their
paid third closure (27 B) reconstruct exactly: **81 paid bytes
recover 80 original bytes**, a recoverability result, not
compression. With the closure absent, the missing third is
not determined by the first two.

The native companion `brown_ledger.rs` remains the authoritative
Rust 1.81 test for signed center/remainder/dual separations, paid
byte-count, 120 signed rooms, forward/back delta playback and
tamper rejection. Its existing source is tested alongside
`three_shadow.rs` and the independent frontier/adversarial tests.

## Future falsification gate

Only after authenticating source identity, ledger cost and the
nine-operator definition should the source-bound DPLL selector be
modified. The needed fair comparison uses identical DPLL visited nodes,
pre-registered candidate sets, split/uninterrupted frontier hashes,
held-out problems, total wall/CPU time, whole-bank bytes,
source-index allocations, all wave/torus computations, and
stateful checkpoint costs.

A successful exact restart or an alternative candidate is an
engineering acceptance. An unknown SHA256d preimage still requires
a completed Bitcoin source-model witness.
