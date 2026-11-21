# Three-language cryptanalysis: independently checkable bounded pilot

## Question and research boundary

Does converting a ciphertext/unknown alphabet into Black, White, and Brown
representations provide independently new information or a real inference
advantage over processing the original observation directly?

This experiment answers by comparing the **same** source observations,
candidate budgets, and hidden inputs across direct and three-view routines.
The source operators come from the existing pinned
'native/src/omega_operator_catalog.rs'; signed cancellation witnesses use
existing 'native/src/relic.rs'. No existing source file or original solver
has been altered.

**Security:** All keys and data are public synthetic/standard laboratory
fixtures. AES implementation is a test-only, non-constant-time diagnostic
using finite-field mathematics (no external crypto dependency). It must
not be used as a production cryptographic library, for wallet recovery,
or as a claim about real-world unknown keys.

## Gate A: understand an unfamiliar 27-symbol language

- Hidden structured rule is an invertible affine permutation over F_3^3.
- Four source examples (origin and three basis vectors) fix all 12
  affine coefficients. Black, white and alternate RNQ views 0/2/7
  each display those same four examples, for 12 encoded observations.
- The learner decodes the views to canonical observations and solves the
  affine rule. One-view and three-view fits must match exactly.
- All other 23 source symbols are withheld from training. Forecast
  all 23 and require exact hidden-rule agreement, plus all three views.
- A contradictory view must be rejected.
- Control: independently use an *arbitrary unknown* 27-symbol permutation
  with 21 observed input/output pairs. The six remaining outputs can be
  assigned in **6! = 720** consistent ways. Translating the same 21 pairs
  into more alphabets gives no additional constraint on the six unknown
  symbols. This is a constructive information-limit counterexample.

## Gate B: controlled toy keyed transform

- A six-round 8-bit Feistel construction with an explicit 4-bit S-box.
- The source oracle uses a fixed 8-bit key and five public test plaintexts,
  generating five output pairs. The key is not an input of the search function.
- Exhaustively evaluate all 256 candidate keys using the same five pairs.
- Reversible RNQ 0/2/7 views must return the identical matching-key set
  as the direct baseline. No key advantage is credited for restating
  ciphertexts in multiple alphabets.

## Gate C: published AES-128 standard encryption

- Native, dependency-free finite-field AES-128 encryptor checked against
  FIPS-197 plaintext '00112233445566778899aabbccddeeff', key
  '000102030405060708090a0b0c0d0e0f', expected ciphertext
  '69c4e0d86a7b0430d8cdb78070b4c55a'.
- Second independent well-known public vector:
  all-zero AES-128 block and key to
  '66e94bd4ef8a2c3b884cfa59ca342b2e'.
- The **first 120 key bits are given to the search function**.
  Only the final 8 key bits are hidden from its inputs. The resulting
  256-key exhaustive search is *not* a test of full 128-bit AES key recovery.
- Direct and three-view searches each perform exactly 256 full AES-128
  encryption calls, returning the same candidate set.
- Tamper one ciphertext byte and require that no candidate from the
  restricted search matches.
- Exhaustively check all 256 possible bytes under a signed Brown
  triple (B,W,R)=(x,-x,0): common-mode zero with exact reconstruction
  from stored signed differences.

## Intended finite census (hosted results are the authority)

| Quantity | Required |
|---|---:|
| Affine training states | 4 |
| Derived RNQ training observations | 12 |
| Affine withheld predictions | 23/23 |
| Arbitrary permutation unobserved symbols | 6 |
| Consistent arbitrary completions | 720 |
| Toy key candidates checked | 256 |
| Toy matching-key count | 1 |
| AES known-prefix key candidates checked | 256 |
| AES matching-key count | 1 |
| Direct AES encryptions | 256 |
| Three-view AES encryptions | 256 |
| Independent information from reversible views | 0 |

The counts above are acceptance targets rather than performance claims.
Brown triple witness storage and view-transform work are additional costs;
no throughput improvement is inferred.

## What would justify a next gate

An independent source of information (for example, known plaintext,
related-message structure, protocol metadata or externally observed
state transitions) must be identified and charged to *every* algorithm
being compared. A genuine result must win on held-out accuracy or a
measured resource budget against matched established baselines.
Reversible representations alone conserve the information already
present in the source.

All development runs under the preexisting mandatory P-NP Rust 1.81
native tests, strict Clippy, frozen-source audit and independent
published-evidence verifier. PR is draft until both checks pass.
