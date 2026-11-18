# Automatic post-GULP reclamation: preserve verified knowledge, discard raw scratch

The original [Omni GULP](https://github.com/JesseBrown1980/asolaria-behcs-256/blob/818cc235a6db815e3b2591339cf9b42aad3e85a0/packages/omni-gulp-gc/src/gulp.ts) groups events by `event/kind` and `glyph/glyph_id`, retaining counts and summaries. Its companion [Omni GC](https://github.com/JesseBrown1980/asolaria-behcs-256/blob/818cc235a6db815e3b2591339cf9b42aad3e85a0/packages/omni-gulp-gc/src/gc.ts) was originally designed to retain raw log archives. This project now adds an **explicit revised retention policy**: *raw, owned, temporary processing batches may be physically discarded once GULP summaries and source-linked verification receipts are committed*. Immutable source files, protected audit receipts, model tensors and separately designated audit archives remain intact.

**There is no per-batch human approval.** The source/GAC policy is pinned and verification is mechanical: commit → verify → discard → verify again. Errors enter HOLD and preserve the unprocessed raw input.

## Exact native sequence

1. Accept only an owned temporary event batch at the fixed scoped location `evidence/PNP_GULP_EPHEMERAL/pending.events.hbp`. The sandbox owns this path; arbitrary supplied source paths are not deletion targets. Reject symlinks, non-files, extra entries, missing/newline-broken events, duplicate sequence IDs and unrecognized record fields. The original test body is [GULP_OWNED_EVENTS.hbp](fixtures/GULP_OWNED_EVENTS.hbp); the harness creates a separate writable copy that is the only deletion target.
2. Recheck all ten pinned GAC/NN/Omni Git source blobs and the published 16-receipt fabric HBI/HBP chain. The original `gulp.ts` is **source authentication evidence** only, with no JavaScript/Node execution.
3. Process every input event in deterministic order. Aggregate **event-kind** and **glyph** counts. The reference fixture contains 10 events: four mistakes, three genius findings, two routes, one neutral; glyph distribution is four MINUS, three PLUS and three ZERO.
4. Write new `PNP_GULP.hbp`, `PNP_GULP.hbi` and `PNP_GULP.anchor` with exclusive creation and synchronized disk writes. They retain the original raw byte count and SHA-256 digest, original GAC/GC evidence root, exact processed-event count and all kind/glyph totals. Every receipt has an HBI offset and length, SHA-256, SH prefix and linked HASH predecessor. They explicitly record `raw_policy=DISCARD_AFTER_COMMIT`.
5. Independently reconstruct the retained counts and re-serialize the entire expected HBP/HBI/anchor archive. Re-read the input and verify it has not changed since processing. **Only after both checks pass**, physically unlink `pending.events.hbp`, remove the now-empty `PNP_GULP_EPHEMERAL` directory, sync the parent, and verify both paths are absent.
6. Create and flush a source-bound `PNP_GULP_DISCARD.hbp` containing the original raw SHA-256, retained HBP/HBI/anchor digests, deleted owned path, exact event count and `manual_approval=NOT_REQUIRED`. On restart the verifier recomputes all retained receipts, checks the discard marker and requires the temporary raw input to remain absent. A missing marker or any residual input is HOLD.

The policy preserves **compacted knowledge and an auditable cryptographic commitment** rather than retaining every original raw event byte. After physical deletion, complete original raw reconstruction is intentionally unavailable. Full source-blob and designated audit-archive preservation from earlier gates remains unchanged.

## Native commands

With original GAC, neural fabric and Omni source checkouts pinned at the revisions already certified by Gate A:

```text
native/target/release/pnp-native fabric-gc-attest source-gac source-neural source-omni evidence

mkdir evidence/PNP_GULP_EPHEMERAL
cp native/fixtures/GULP_OWNED_EVENTS.hbp evidence/PNP_GULP_EPHEMERAL/pending.events.hbp

native/target/release/pnp-native gulp-discard source-gac source-neural source-omni evidence
native/target/release/pnp-native gulp-discard-verify source-gac source-neural source-omni evidence
```

Expected postcondition: the `PNP_GULP_EPHEMERAL` directory is **absent**, including its original full batch. The retained aggregate HBP/HBI/anchor/discard marker exist and pass independent digest checks. Repeating the command refuses to overwrite or re-delete.

## Negative acceptance and scope

CI checks that corrupt or incomplete events are **not deleted**, a premature/occupied journal prevents deletion, and a complete commit deletes raw input without a human confirmation. It also checks that any modified HBI/HBP/anchor/discard marker is rejected; raw data unexpectedly reappearing fails the restart replay. All original files and archives are byte-for-byte preserved. The published summary is replayed by a separate hosted acceptance job.

This is an **actual file reclamation test over a disposable native event-batch**, not yet direct integration with the original JSON NDJSON event streams. The current native interface uses a canonical non-JSON HBP ingestion format and integer-only Rust 1.81. Future direct GULP ingestion and a live log-rotation integration require source-matched input parsing, concurrency/crash recovery and explicit configuration of disposable versus protected source classes. Those remain distinct acceptance gates, without a per-operation live operator approval requirement.
