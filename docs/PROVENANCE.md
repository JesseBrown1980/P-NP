# Provenance — upstream exact SAT transfer

Original research repository: [JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C](https://github.com/JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C).

**Frozen upstream ref for the initial port:** `b9fd691d74adc72b2e4378db711a2237873cb818` (merge of held-out acceptance #302 into integrated research branch #299). The first P-NP port copies source files without changing their contents, under `sat/`.

| P-NP copy | Upstream Git blob SHA |
|---|---|
| `sat/engine.py` | `0f67b6a32ad830188c1b8194ed212ef4950c3e7b` |
| `sat/verify.py` | `df30dba9aaf079faf64e39af6001355dda5cddb5` |
| `sat/verify_heldout.py` | `ca3362038d954401337403d6168751856459142b` |
| `sat/README.md` | `39e6072198a413e13c4aab65bac20271c65c3ba1` |
| `sat/HELDOUT.md` | `6a5c559feb39657f08998b2ae72cefc0621631e7` |

The original CI workflow (blob `28d16ac6f18e4371d989d21b9f73e70a0d01f1ef`) was adapted to a standalone P-NP workflow: new paths, new artifact name, and a local Python exact-arithmetic AST gate. The upstream repository's larger native Rust/no-float workflows remain in that repository; the standalone policy and original-source replay together guard the transferred Python program.

## Archived upstream evidence

- Initial 11-case source-frozen integrated SAT verification: [hosted run #44184932235](https://github.com/JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C/actions/runs/44184932235): seven independently confirmed counts; four explicit budget HOLDs.
- Original 21-case held-out SAT suite: [hosted run #44186890988](https://github.com/JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C/actions/runs/44186890988): 20 exact counts, one HOLD, 21 matching MiniSat/PicoSAT SAT decisions.
- Four strengthened independent gates: [hosted run #44190628220](https://github.com/JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C/actions/runs/44190628220): proof/result/witness SHA reconstruction, 13 corrupted receipts rejected, independent model checks including planted-30 HOLD, required artifact download+hash. Archive ID `14376416760`, published zip SHA256 `5da2612e7c315f574a94af2adcd0b2a8a559621ea145d2c03ac4d94243836685`.
- [Review discussion](https://github.com/JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C/pull/302) documents the independent mathematical spot-check and the acceptance follow-up.

This new repository becomes the home for ongoing algorithm research. The original Yang–Mills repository preserves its own mathematical and physical results, including the original first-excitation Wilson spectral-weight HOLD.
