# P-NP software acceptance — Yang–Mills-grade Rust 1.81 cutover

**Effective 2026-11-17 São Paulo / 2026-11-18 UTC. Governing rule for all implementation acceptance and all earlier P-NP implementations.**

## Non-negotiable native software contract

1. **Executable research kernels are Rust 1.81.0** with an explicit `rust-toolchain.toml`, Cargo.lock, no third-party network dependencies, and `--offline --locked` tests and release builds.
2. **Exact arithmetic only:** integers, bounded balanced trits, checked products/sums, explicit exact comparisons; zero floating-point type/literal or compiled project floating-point arithmetic. No `unsafe` code. Native binary and tests must pass Clippy with `-D warnings` and scan the generated Rust LLVM IR.
3. **0 Node.js, 0 JSON, 0 V8** in the accepted runtime and receipt formats. Receipts use deterministic HBP/HBI text and SHA-256 manifests. Historical filenames and archival records remain available for provenance but are **not native implementations**.
4. **Original sources frozen and authenticated.** Hash the exact source, inputs, Rust binary and outputs; no silent changes to imported historical models, matrices or tests. Historical Python and workflows are pinned to the original Git blobs.
5. **Independent replay and negative controls.** Original-source mathematical/solution checks, byte-exact replays, third-seat watcher checks, and planted corruption rejection are mandatory. Referee outcomes distinguish `PASS`, `FAIL`, `HOLD`, and `NOT_COMPUTED` and must not promote a partial result.
6. **Published hosted acceptance.** Tests, offline build, Clippy, native integer-source and compiled-IR checks, pinned legacy audit, reproducible HBP receipts, adversarial tests and a second independent archive checksum must pass before a new implementation receives native software acceptance.
7. **No acceptance by inheritance:** a green earlier Python benchmark or solver trial cannot substitute for passing the Rust native gate. Any new SAT theorem claim requires a separate mathematical proof; a finite completion result is insufficient.

## Retrospective status (all historical P-NP implementations)

The immutable [LEGACY_PINNED.hbp](../policy/LEGACY_PINNED.hbp) enumerates **44 original Python executable files** and **10 original legacy GitHub Actions workflows**: **54/54 software artifacts have `FAIL_NATIVE_ACCEPTANCE`** under this new contract. This includes the original SAT engine, historical Fischer adaptations, fourth HXF, array and Bitcoin source generators, 378 RIME tests, MTP/N-Nest studies and their earlier verification scripts. They remain byte-identical and readable. Unmerged Python-first branches, including the completion/scaling prototype and third-fabric-witness prototype, also require a native Rust rewrite and full acceptance before promotion.

**Research-evidence status is separate.** Exact historical CNF SAT witnesses, verified SHA-256 hashes, measurements, and archival mathematical constraints remain available for inspection. The new FAIL label evaluates implementation conformity, not an assertion that every earlier answer or mathematical calculation was false. In particular, no P=NP mathematical theorem has been accepted.

## First native scope

The first `native/` release demonstrates the **uniform N × 3 × 3 × 3 matrix address table** with per-room Omega center and exactly 1 center, 6 face, 12 edge and 8 corner cells; exact frozen 3D source with recorded 2D projection; integer Fischer on-demand modular-address play; and a fail-closed 3-seat worker/reflection/**matrix-as-fabric** wave cascade with 3, 9, 27 and 81 room levels. Native tests reject **42 planted worker/reflection/fabric corruptions** throughout the nested hierarchy and replay all **3,240 addressed original cells**. The frozen pilot currently retains the complete source cells and does not claim data compression, a trained GNN or an independent inversion of cryptographic hashes.

**Next native acceptance tasks (HOLD):** migration of the complete SAT decision engine and exact proofs; trained GNN traversal-to-function production; FNN; Host-8 room machinery; OmniDispatcher, OmniSpindle, OmniRouters, OmniHotel and OmegaHotel; Omega/Omnibit tournament; full source-authenticated matrix fabric queries. These must be ported from original source and separately certified, not mocked into PASS by naming an adapter.

## Hard enforcement

Workflow [pnp-native-certification.yml](../.github/workflows/pnp-native-certification.yml) runs on every pull request to main, every main push, and merge queue. It rejects missing/mutated legacy Git blobs, new non-Rust executable files or new uncontrolled CI workflows, while accepting the one source-pinned Rust native path. The native auditor also tests altered legacy source and an introduced Python runtime as red/green negative controls. Two jobs—`native-rust181-int-only-clippy-sources-replay` and `published-evidence-independent-check`—must both pass.

**Repository branch protection/ruleset administrators should mark both native jobs as required.** A workflow alone records a failing check but cannot block an administrator or an unprotected branch from merging; no false claim of protected-branch enforcement is implied.

### Yang–Mills source precedent

Compared against the original Yang–Mills [`native-rust.yml`](https://github.com/JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C/blob/main/.github/workflows/native-rust.yml), [`no_float.yml`](https://github.com/JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C/blob/main/.github/workflows/no_float.yml), and [`no-float-compiled.yml`](https://github.com/JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C/blob/main/.github/workflows/no-float-compiled.yml). This P-NP gate adopts the relevant Rust 1.81, integer, Clippy, source-pin, red-team, and independent-replay controls without importing Yang–Mills physics results as P-NP results.