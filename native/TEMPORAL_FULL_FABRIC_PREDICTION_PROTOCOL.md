# PRL temporal-memory evidence across the entire native binary/ternary fabric

## Scientific question

The accepted paper by Nicolas Gosling and colleagues, *Probing the
memory of a superconducting qubit environment*, PRL accepted
2026-10-28, DOI 10.1103/j1r2-5ph7, motivated P-NP's original
Solomon-population-inspired classical TLS simulator and the independent
three-regime synthetic held-out stochastic event corpus. This gate asks:

**Does the full binary/ternary/Omnibit/Fischer/native HTTP-0 path preserve
useful causal event-history information, and can source-bound room
recall reuse its actual computations without corrupting prediction?**

This is a classical synthetic temporal-prediction experiment, not a
reproduction of the superconducting qubit measurements. No hardware
rate fit or physical g2 measurement is claimed.

## Fully matched design

The nine source-authenticated original cases contain 8,000 training,
4,000 validation, 8,000 held-out, and 8,000 count-preserving shuffled
held-out events each. The original source archive parser and its SHA
checks are reused. Train-only fitted models have original capacities
0,1,3,27,81. Model selection uses **validation-only Brier + fixed
complexity penalty**, before the held-out or shuffled sequences are
fed through the fabric. No train/selection leak is allowed.

Each ordered and shuffled 8,000-event stream is divided into 297
ordered packets of 27 trits (last packet has 8 observations and 19
canonical zero-trit padding positions). Every packet is encoded and
transmitted over all four original binary alphabets 64, 256, 1024,
4096. A deterministic operator schedule cycles through all 40
actual original Omnibit operators, using their real forward/inverse.
Backend HTTP-0 routes these messages into Fischer-addressed native
microkernel rooms. The independent child must return the original
27 trits and the actual 243-tick matrix recurrence output; both
are checked on every request. The worker gets no prediction oracle
or held-out outcome not already contained in the packet being
transported. Prediction is computed causally in order from the
reconstructed event sequence, using the original train-fitted model.

18 event streams x 297 packets x 4 floors =
**21,384 HTTP-0 requests per pass**. There are three passes:
- Uncached backend room control: 21,384 calculations.
- Cache first pass: 5,346 calculations + 16,038 representation
  reuse hits across the four equivalent floor encodings.
- Cache second pass: 21,384 verified repeat hits, without further
  calculation.

Total: **64,152 actual HTTP-0 messages**, with **37,422 real
source-keyed recall hits** and an 18-room visited subset of capacity
27. Predictor outputs are scored over all **144,000** unique held-out
and shuffled events, with exact integer Brier squares. Source train
and validation outcomes are never transported or used in held-out
selection, and every read is verified against the immutable source
trace. The comparison controls use the same trained model in both
ordered and count-preserving shuffled conditions.

The original observation-trained stochastic TLS model should show
positive held-out improvement versus memoryless reference for all
six coupled-source cases, lose that improvement on all six
count-preserving shuffled cases, and select the null reference on
all three IID controls. All nine per-case raw Brier totals and signed
advantages are retained independently rather than simply recording
a PASS count. The original separately implemented causal online
prediction loop must match the original Brier scorer exactly.

## GNN and claim boundary

Full-matrix Omnibit40/bridge and 60D/GNN source roots are carried
through PNP_FULL_MATRIX.hbp, alongside PNP_TLS.hbp,
PNP_TLS_STOCHASTIC.hbp, PNP_GNN_GATE.hbp and PNP_CHECKPOINT.hbp.
Actual trained GNN model forward parity remains HOLD. No surrogate
or fake GNN score is used for event prediction; genuine original
G4 acceptance remains separately tested upstream. This gate uses
the **original observation-trained classical TLS model** (not a
learned GNN or loaded HRM). It tests preservation and causal
predictive quality through the full native transport.

Performance outcomes must separately charge the serial HTTP-0
roundtrips and model/cache storage. A cache hit is a saved recurrence,
not an improvement in Brier probability, and the absence of
parallelism means wall comparisons do not establish scaling laws.

## Evidence and protection

    pnp-native temporal-fabric-attest evidence evidence
    pnp-native temporal-fabric-verify evidence evidence
    pnp-native temporal-fabric-inspect evidence/PNP_TEMPORAL_FABRIC_RESULTS.bin

Output files are PNP_TEMPORAL_FABRIC_RESULTS.bin, .hbp, .hbi, .anchor,
and independently checked PNP_TEMPORAL_FABRIC_DIGESTS.sha256.
All HBI/HBP rows and raw exact result records are regenerated on
verification; latency readings are logged but kept outside the
canonical hash receipts. Reject tampered records, witness or index.
Source archives are unchanged. Previously accepted automatic
owned-scratch GULP deletion remains active.

Status: implementation submitted for hosted Rust 1.81, strict Clippy,
full native CI and independent published-artifact review. Predictive
claims will follow actual acceptance logs.
