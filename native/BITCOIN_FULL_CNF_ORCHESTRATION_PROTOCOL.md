# Full original Bitcoin CNF: direct SAT versus 27-cell pump experiment

Stage: draft P-NP PR #57. Hosted acceptance at an exact commit is required.
This experiment follows the accepted 300-second MiniSat/CaDiCaL/PicoSAT
direct reference (all six large-case trials were HOLD_TIME). The older native
matrix funnel tested the exact 80-byte public genesis header. This stage
moves the actual original 32-bit and 288-bit free-field DIMACS CNF bytes
through the matrix before calling an unmodified external SAT solver.

## Executable components

- Native Rust 1.81 exact stream: native/src/bin/bitcoin_cnf_pump.rs
- Original strict Bitcoin source generator:
  experiments/bitcoin/genesis.py (immutable pinned Git blob)
- Original external solver source verifier:
  experiments/bitcoin/runner.py (immutable pinned Git blob)
- Extended 300-second external reference adapter:
  research/bitcoin_300s_reference.py
- Independent matrix versus direct comparator:
  research/bitcoin_cnf_pump_compare.py
- Hosted orchestration:
  .github/workflows/pnp-bitcoin-cnf-orchestration-compare.yml

## Real source and reversible transformation

The original nonce32 source encodes 239,112 SAT variables and 877,879
clauses. The merkle_nonce288 source encodes 361,188 variables and
1,325,357 clauses. Both are the complete two-pass Bitcoin SHA256 Boolean
circuits, pinned by their original source SHA256 roots.

For each four CNF input bytes, the native Rust pump:

1. Maps the u32 source word injectively to 21 balanced ternary digits
   (3^21 exceeds 2^32), with six additional deterministic balanced
   trits carrying a frame-dependent integrity check.
2. Makes a genuine 27-cell trit frame and independently recovers the four
   bytes from that frame; every source byte and frame is exercised.
3. At frame zero, every 243rd frame, and the final frame, makes an actual
   Brown Ledger slice and exact signed change record, verifies freeze/play
   and forward/reverse, and checks a Fischer prime-tower inverse room
   address. The space of finite Fischer positions is used cyclically and
   explicitly labelled to avoid suggesting unique global identifiers.
4. Binds checkpoint data, ordered frame indices and paid serialized
   witness bytes into a domain-separated SHA256 evidence commitment.
5. Writes the complete source-equivalent CNF. Both the Rust producer and
   a separate hosted referee check complete source byte equality.

This is a reversible transport method for the exact hard SAT instance.
The Brown frozen witness samples are explicitly counted, while every
trit frame is independently inverted. A SHA digest is an identity check;
the experiment does not assume it performs cryptographic inversion.

## Direct versus matrix routes

Use CaDiCaL 1.7.4 against both original source cases:

- **Direct**: original pinned CNF -> original historical solver runner
  -> original Bitcoin SHA256d and CNF witness checker.
- **Matrix**: identical original CNF -> full balanced-ternary frame pump
  -> exact inverse and hash/byte verification -> identical original
  historical solver runner and checker.

Both routes receive the same independently source-pinned source SHA and
the same **300-second solver wall-time limit**. Each pair runs on one
GitHub Actions host, with the route order reversed between the two
Bitcoin cases to expose possible warm-cache ordering effects.

The matrix route incurs extra preprocessing, freeze, witness and
reconstruction cost. Record its nanoseconds separately and include
them in its accounted total. Do not combine this experiment with the
previous reference run as if it were the same timed CPU trial.
A subsequent controlled gate can fix the *overall* 300-second budget,
deducting pump time from the solver's allowed time, with repeat trials
and confidence intervals.

The independent stdlib referee checks complete source equivalence,
all four solver receipts, source IDs, version/case/limits, source CNF
clauses for every reported SAT model, full original 80-byte Bitcoin
header and both SHA256 rounds, eight incorrect-header controls, and
four combined wall-time figures. It records SAT or honest HOLD_TIME
separately and archives a full SHA256 manifest with a separate readback.

## Mathematical and engineering interpretation

This is the first end-to-end *actual hard source CNF through matrix to
SAT* comparison. Because both routes deliver identical CNF bytes to the
same solver, the expected difference is authenticated transport and
verification overhead. Any speedup in cryptographic search will require
a further representation-aware search policy or constraint reduction
that changes the algorithm's work while maintaining the original CNF
and final SHA256d oracle.

The finite discrete matrix interpretation remains a research motivation.
The P-versus-NP complexity theorem requires a separate mathematical
proof, including worst-case behavior and fully charged orchestration
overhead.
