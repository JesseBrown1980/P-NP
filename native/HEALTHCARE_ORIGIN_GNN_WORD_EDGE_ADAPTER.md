# Healthcare-origin edge learning → ASOLARIA trained word-wave diagnostic

Date: 2026-11-18/19

## Source and original idea

Original author repository:
`JesseBrown1980/AI-healthCare-project@e0048c0ad60705ecb447642f08b14b5c1bde7437`.

The foundational design classified **relations**, not isolated nodes.
`backend/anomaly_detector/models/graph_builder.py` explicitly maps
`LogEvent(source_entity,destination_entity,event_id,action,metadata)`
to a source→destination edge, retains every event ID and builds
semantic entity features. The clinical variant retains patient,
medication, condition, provider, lab nodes and named relationship
types (prescribed, diagnosed, measured, interacts_with, treats).
These are separate graph tasks; this integration uses only
**nonclinical BEHCS word events** and no identifiable patient data.

The original PyTorch model family is:

| Original model | Git blob | Core mechanism |
|---|---|---|
| EdgeLevelGNN, `gnn_baseline.py` | `510f78890ec94b113f0610afbade8bafe6ca20e0` | Two GCN layers, source/target edge MLP |
| PrototypeGNN | `99e3087a10ee58e90c0935f5ab63b72fd3cdd07e` | Trainable per-class prototypes |
| ContrastiveGNN | `56329e61eb3e6ddb3ee97b46f997dd8dd8c6b39f` | Supervised contrastive edge projection |
| GSLGNN | `886b3b0c0cdbddba983fa8c3ae083c4520d38f0e` | Learned adjacency plus original graph branch |

The GSLGNN source blob is byte-identical to the later ASOLARIA
trained-model source. Its original healthcare service factory defaults
to **16 input features**, hidden dimension 32 and four classes. The
checked-in healthcare service instantiates a model but has its optional
checkpoint autoload disabled. The later ASOLARIA trained checkpoint
`gslgnn_w9_3_seq47_v2.pt` uses **six input features**, hidden
dimension 64, and two classes, with strict checkpoint SHA
`9e50723b10c9de5c27baa5bc9386340bf72f76bcae2877064a47e4627a86602c`.
The two configurations must never be silently mixed.

## What the first trained checkpoint did

[Original checkpoint inference run #44440538271](https://github.com/JesseBrown1980/P-NP/actions/runs/44440538271)
passed strict original weights and independent source-witness checks.
The 12-node / 36-edge synthetic event aggregation produced a
class-1 prediction near **0.529157** on every edge. Naive,
symmetric, and confidence-weighted cascade feedback saturated all 36
edge priors; periodic reset left six saturated. This was genuine
trained-model inference on an **experimental node-feature adapter**,
not a recovered labeled training set.

[Feature-sensitivity run #44441044569](https://github.com/JesseBrown1980/P-NP/actions/runs/44441044569)
then perturbed all six input features substantially. It reported
score changes of 70–884 nanounits for individual controls and
`prediction_sensitivity=FLAT` under its numerical threshold.
This confirms the need for source-semantic topology and calibrated
features. The original manifest's ~315,209 positive versus 40 negative
edges also makes class-prior collapse plausible; this is a
hypothesis, not an established causal attribution.

## Next bridge: preserve each word as an edge-bearing entity

The existing `gnn_word_probe` generates 432 source-valid original
six-axis word coordinates, 36 per wave × 12 waves, each SHA-attested
and assigned the original eight-glyph BEHCS address.

The enhanced source-pinned
`.github/workflows/pnp-gnn-trained-wave-reference.yml` now:

1. Checks the exact original healthcare Git commit and all four
   model source blobs, plus `graph_builder.py`.
2. Uses each original word as a distinct intermediary vertex in
   `actor → word-event → target`, preserving 36 event IDs in the
   held-out wave. This is **48 nodes, 72 directed edges** instead of
   reducing those word identities to 12 nodes/36 edges.
3. Constructs 6-feature node vectors matching the trained ASOLARIA
   schema (edge volume/100, average and maximum risk/9,
   failure fraction, online flag, trust tier/3). Per-event values
   are **explicit experimental proxies**; no new medical or labeled
   corpus is invented.
4. Independently imports the byte-identical healthcare GSLGNN class,
   loads the exact same trained parameters strictly and compares
   forward outputs **element-for-element** against the ASOLARIA
   model implementation.
5. Separately runs the original healthcare 16-feature
   `EdgeLevelGNN` architecture as a **never-trained structural
   control**, using typed semantic entity/word features. Its
   output distribution is not a healthcare accuracy estimate.
6. Records trained score spread on the event-bearing graph and
   untrained structural-control spread, with explicit SHA, source
   commit, weights, graph size, and quality-status labels.

The purpose is to determine whether event/verb/state preservation
reveals score differentiation under the original trained weights.
If scores remain flat, assess checkpoint class-prior dominance and
feature-domain mismatch next. Any re-training or labeling must use
a separate, authentic, balanced, held-out corpus and independent
review of class-specific performance.

## Reproducibility and technical cautions

The healthcare `GraphBuilder` currently seeds its noise using
Python's process-randomized `hash(entity_id)` and mutates the global
PyTorch RNG. That is not a stable cross-process identity encoding.
The source-bound ASOLARIA probe instead uses exact word SHA-256
and fixed semantic features to make receipts reproducible.
The original 16-feature graph builder and ASOLARIA 6-feature
trained checkpoint are distinct model contracts.

**Acceptance boundaries:** successful inference verifies checkpoint
compatibility and source-derived forward execution. Differentiated
predictions verify sensitivity. Clinical performance, general
vocabulary intelligence, anomaly-detection calibration and
held-out balanced metrics require separate evidence.
