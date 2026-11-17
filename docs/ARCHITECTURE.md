# One architecture — frozen constraints, verified representations

```text
Original encoded problem
   │ input SHA-256, generator/benchmark identity, limits
   ▼
Frozen global constraint matrix / CNF
   │
   ├── MTP proposer: recognize XOR / structural motifs / candidate reductions
   │                  each with a reconstruction proof
   │
   ├── Bobby Fischer-inspired selector: estimate total future work,
   │                  propose ordering or representation, compare alternatives
   │
   └── N-Nest independent watchers: reconstruct from original source;
                      catch altered proposal, rank, witness, status or receipt
   │
   ▼
Accepted exact representation + bounded search
   │ exact count OR SAT witness / independently checked UNSAT OR explicit HOLD
   ▼
Proof SHA / result SHA / resource accounting
   │
   ▼
Published artifact + second-job re-download and checksum verification
```

## Current runnable research implementation

- `sat/engine.py`: exact three-variable parity recognition, Gaussian elimination on bitsets, explicit residual SAT constraints, Fischer-inspired static variable order, fixed-order comparator, bounded exact model counting, source and result SHA receipts, controlled deterministic DIMACS families.
- `sat/verify.py`: independent input generation and alternate Gaussian elimination for the original 11-case reference suite.
- `sat/verify_heldout.py`: independent fixture reconstruction; proof-chain, rank, source, result and witness checks; independent exact counting; 13 corrupted receipt trials; both external SAT model checks; 21-case acceptance.
- `.github/workflows/sat-source-certified.yml`: Ubuntu 24.04 source/generator, exact gate, 11+21 cases, release-pinned MiniSat and PicoSAT, per-case output archives, independent verifier, re-download of published artifact.
- `tools/exact_policy.py`: intentionally narrow integer-only static AST check for Python artifacts. It complements the hosted exact integer results; it is not a whole-program proof.

## Research expansion interfaces

Candidate-representation adapters should expose: source SHA, exact assumptions, proposal identity, target formula or factorization, verifier, estimated work/memory, and a receipt. The selector may include XOR bitset algebra, bounded dependency tables, conventional DPLL/CDCL SAT decision and exact model counting, while counting the cost of unsuccessful candidates and verification.

The same proof architecture can later be reused for domain-specific finite physics or other constraint systems **only after** its independent checker is specified for that domain. Original physical results remain anchored to their source repository.

## Related projects

- [N-Nest-Prime](https://github.com/JesseBrown1980/N-Nest-Prime-INFINITE-SELF-REFLECT-AGENTS-NESTED): recursive watcher and operator-as-breaker principles.
- [Bobby Fischer evaluator](https://github.com/JesseBrown1980/bigpickle-rebuild/blob/main/src/fischer-kernel.mjs): original anti-blunder decision scoring, adapted here into SAT-specific measured candidate rankings.
- [Yang–Mills finite-universe research](https://github.com/JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C): original finite-state mathematical application and source-certified SAT pilots.

We import these *ideas and documented verified SAT algorithms*, with clear provenance, rather than relying on hidden behavior or unverified agent authority.
