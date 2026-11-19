#!/usr/bin/env python3
"""Reproducible source-pinned original GNN checkpoint comparison.

Runs public original healthcare model classes, real stored checkpoint
weights, and source-verified BEHCS words. No clinical or private training
records, invented ground truth, or calibration claims.
"""
import argparse
import hashlib
import json
import math
import os
import subprocess
import sys
from pathlib import Path

import torch
import torch_geometric

HEALTH_SHA = "e0048c0ad60705ecb447642f08b14b5c1bde7437"
WORD_SHA = "818cc235a6db815e3b2591339cf9b42aad3e85a0"
MODEL_SHA = "c4b2458c88358c46155d3b308c594f9991720847"

HEALTH_BLOBS = {
    "gnn_baseline.py": "510f78890ec94b113f0610afbade8bafe6ca20e0",
    "prototype_gnn.py": "99e3087a10ee58e90c0935f5ab63b72fd3cdd07e",
    "contrastive_gnn.py": "56329e61eb3e6ddb3ee97b46f997dd8dd8c6b39f",
    "gsl_gnn.py": "886b3b0c0cdbddba983fa8c3ae083c4520d38f0e",
}
MODEL_BLOBS = {
    "baseline_model.pt": "26caf5216a4df3a43250e40e1dd421180ca3c901",
    "prototype_model.pt": "1d1be27874e5fa4a7ef88026561033a0e0aac3fe",
    "prototype_model_run2.pt": "3660b469f648aa4287b9f6a19732543a22e72ccc",
    "prototype_model_run3.pt": "1d1be27874e5fa4a7ef88026561033a0e0aac3fe",
    "contrastive_model.pt": "76b7ec50d57e1a70cb9d755edffc75afc15e7a4b",
    "contrastive_model_run2.pt": "6ab8743e0e3dca29de7387443a618272a2c0507f",
    "contrastive_model_run3.pt": "76b7ec50d57e1a70cb9d755edffc75afc15e7a4b",
    "gslgnn_w9_3_seq47.pt": "6cddadc91ff4e6fa2bbfc315d90fa1bab0117c6b",
    "gslgnn_w9_3_seq47_v2.pt": "e3a9712b50f8e7b31590b0aa944e6a9ba565b",  # checked below
}
MODEL_BLOBS["gslgnn_w9_3_seq47_v2.pt"] = "e3a9712b50f8e7b31590b0aa944e6a9ba6ea565b"
MODEL_LIST = [
    ("baseline", "baseline_model.pt"),
    ("prototype", "prototype_model.pt"),
    ("prototype", "prototype_model_run2.pt"),
    ("contrastive", "contrastive_model.pt"),
    ("contrastive", "contrastive_model_run2.pt"),
    ("gsl", "gslgnn_w9_3_seq47.pt"),
    ("gsl", "gslgnn_w9_3_seq47_v2.pt"),
]
ACTORS = ("acer", "liris", "falcon", "felipe", "beast", "dan")
TARGETS = ("acer", "liris", "falcon", "federation", "phantom", "witness")
VERBS = ("scan-dispatch", "scan-result", "drift-detected", "cosign-append", "migration-intent", "heartbeat")
STATES = ("pending", "in-flight", "dispatched", "settled", "failed", "observed")

def sha(value):
    return hashlib.sha256(value).hexdigest()

def gits(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()

def source_auth(health, words, models):
    for root, expected in ((health, HEALTH_SHA), (words, WORD_SHA), (models, MODEL_SHA)):
        assert gits(root, "rev-parse", "HEAD") == expected, f"source commit changed at {root}"
    source_dir = health / "backend/anomaly_detector/models"
    for name, blob in HEALTH_BLOBS.items():
        assert gits(health, "hash-object", f"backend/anomaly_detector/models/{name}") == blob, name
    for name, blob in MODEL_BLOBS.items():
        assert gits(models, "hash-object", f"models/{name}") == blob, name
    assert gits(words, "hash-object", "packages/deep-wave/src/cascade.ts") == "b9acdaff9b4537def53eca3d0fdb87a9c0dcab89"
    assert gits(words, "hash-object", "data/behcs/codex/alphabet.json") == "8d4a3298576ce2c9d4a1683ffab667c8a743fc42"
    manifest = json.loads((models / "manifests/gslgnn_w9_3_seq47_v2_manifest.json").read_text())
    assert sha((models / "models/gslgnn_w9_3_seq47_v2.pt").read_bytes()) == manifest["checkpoint_sha256"]
    assert manifest["corpus_stats"]["benign"] == 40 and manifest["corpus_stats"]["suspicious"] == 315209
    return source_dir, manifest

def word_events(path):
    raw = path.read_bytes()
    lines = raw.decode("utf-8").splitlines()
    assert len(lines) == 433
    head = dict(part.split("=", 1) for part in lines[0].split("|")[1:])
    assert head["all_source_events"] == "93312" and head["sampled_words"] == "432"
    assert sha(("\n".join(lines[1:]) + "\n").encode()) == head["word_sha"]
    events = []
    for idx, row in enumerate(lines[1:]):
        pieces = row.split("|")
        assert pieces[0] == "WORD"
        e = dict(part.split("=", 1) for part in pieces[1:])
        assert int(e["id"]) == idx
        assert e["actor"] in ACTORS and e["target"] in TARGETS
        assert e["verb"] in VERBS and e["state"] in STATES
        wave = int(e["wave"])
        assert wave == idx // 36
        assert 0 <= int(e["risk"]) < 6
        original_word = (
            f'D1:{e["actor"]}/D2:{e["verb"]}/D3:{e["target"]}/'
            f'D4:{e["risk"]}/D5:{e["state"]}/D6:w{wave}'
        )
        assert sha(original_word.encode()) == e["word_sha"]
        events.append(e)
    return raw, events

def graph(events, wave, dim):
    assert dim in (6, 16), f"original feature dimensionality not mapped: {dim}"
    subset = [e for e in events if int(e["wave"]) == wave]
    assert len(subset) == 36 and len({e["word_sha"] for e in subset}) == 36
    x = torch.zeros((48, dim), dtype=torch.float32)
    left, right = [], []
    # Source-compatible 6-feature numeric contract, retained for all models.
    # Additional 16-feature domain indicators follow the original healthcare
    # concept of typed semantic nodes, not an original training-time mapping.
    for a in range(6):
        x[a, :] = torch.tensor([0.06, 0.3, 0.5, 0.0, 1.0, 1/3] + ([0.0]*10 if dim==16 else []))
        x[6+a, :] = torch.tensor([0.06, 0.3, 0.5, 0.0, 1.0, 2/3] + ([0.0]*10 if dim==16 else []))
        if dim == 16:
            x[a, 6] = 1.0
            x[6+a, 7] = 1.0
            x[a, 10] = a / 5
            x[6+a, 11] = a / 5
    for i, e in enumerate(subset):
        actor = ACTORS.index(e["actor"])
        target = TARGETS.index(e["target"])
        risk = int(e["risk"])
        failure = float(e["state"] == "failed")
        x[12+i, :6] = torch.tensor([0.01, risk/9.0, risk/9.0, failure, 1-failure, 1/3])
        if dim == 16:
            x[12+i, 8] = 1
            x[12+i, 9] = VERBS.index(e["verb"]) / 5
            x[12+i, 10] = STATES.index(e["state"]) / 5
            x[12+i, 11] = actor / 5
            x[12+i, 12] = target / 5
            x[12+i, 13] = wave / 11
            x[12+i, 14] = risk / 5
            x[12+i, 15] = i / 35
        left.extend((actor, 12+i))
        right.extend((12+i, 6+target))
    edge_index = torch.tensor([left, right], dtype=torch.long)
    assert edge_index.shape == (2, 72)
    return x, edge_index

def normalize_keys(state, model):
    expected = model.state_dict()
    result = {}
    for name, value in state.items():
        assert isinstance(value, torch.Tensor), f"checkpoint has unsupported non-tensor key {name}"
        options = [name]
        if name.startswith("module."):
            options.append(name[len("module."):])
        for base in list(options):
            if ".lin.bias" in base:
                options.append(base.replace(".lin.bias", ".bias"))
            if base.endswith(".bias") and ".lin." not in base:
                options.append(base.replace(".bias", ".lin.bias"))
        matched = [key for key in options if key in expected and expected[key].shape == value.shape]
        assert len(set(matched)) == 1, f"checkpoint parameter cannot be mapped exactly: {name} -> {matched}"
        key = matched[0]
        assert key not in result, "checkpoint state key collision"
        result[key] = value
    assert set(result) == set(expected), f"missing strict parameter keys: {sorted(set(expected)-set(result))}"
    model.load_state_dict(result, strict=True)
    return len(result)

def open_checkpoint(path):
    state = torch.load(path, map_location="cpu", weights_only=True)
    if isinstance(state, dict) and "state_dict" in state:
        state = state["state_dict"]
    assert isinstance(state, dict) and len(state) > 4
    assert all(isinstance(k, str) for k in state)
    return state

def shape_of(state, needle):
    candidates = [(k, tuple(v.shape)) for k,v in state.items() if k.endswith(needle)]
    assert len(candidates) == 1, f"expected one {needle} tensor, got {candidates}"
    return candidates[0][1]

def construct(model_name, state, origin):
    conv = shape_of(state, "conv1.lin.weight")
    hidden, dim = conv
    assert dim in (6, 16) and hidden in (16, 32, 64, 128), f"unrecognized original dimensionality: {conv}"
    if model_name == "baseline":
        model = origin.EdgeLevelGNN(node_input_dim=dim,hidden_dim=hidden,output_dim=1)
    elif model_name == "prototype":
        ncls, nproto, phidden = shape_of(state, "prototypes")
        assert phidden == hidden and 1 <= nproto <= 8
        model = origin.PrototypeGNN(node_input_dim=dim,hidden_dim=hidden,
                                    num_classes=ncls,num_prototypes_per_class=nproto)
    elif model_name == "contrastive":
        proj, p_hidden = shape_of(state, "projection_head.2.weight")
        ncls, _ = shape_of(state, "classifier.3.weight")
        assert p_hidden == hidden and ncls == 2
        model = origin.ContrastiveGNN(node_input_dim=dim,hidden_dim=hidden,
                                      projection_dim=proj,num_classes=ncls)
    elif model_name == "gsl":
        ncls, _ = shape_of(state, "edge_classifier.5.weight")
        assert ncls == 2
        gsl_dim, in_gsl = shape_of(state, "gsl.key_transform.weight")
        assert in_gsl == dim
        model = origin.GSLGNN(node_input_dim=dim,hidden_dim=hidden,
                              num_classes=ncls,gsl_hidden_dim=gsl_dim)
    else:
        raise ValueError(model_name)
    return model, dim, hidden

def metrics(model, events, dim):
    all_scores = []
    per_wave = []
    perturb_deltas = []
    rewire_deltas = []
    with torch.no_grad():
        for wave in (8,9,10,11):
            x, edges = graph(events, wave, dim)
            y = model(x, edges)
            assert y.shape == (72,) and torch.isfinite(y).all()
            assert bool(((y>=0)&(y<=1)).all())
            repeat = model(x,edges)
            assert torch.equal(y,repeat), "nondeterministic eval checkpoint"
            scores = y[1::2] # each word's event->target edge, in original row order
            per_wave.append(float(scores.max()-scores.min()))
            all_scores.extend(float(t) for t in scores)
            shifted = x.clone()
            shifted[12:,1] = 1 - shifted[12:,1]
            shifted[12:,2] = 1 - shifted[12:,2]
            changed = model(shifted, edges)[1::2]
            perturb_deltas.append(float((changed-scores).abs().mean()))
            rerouted = edges.clone()
            # Change ONLY the named target nodes on the 36 event->target
            # edges. Source and event IDs remain identical.
            rerouted[1,1::2] = 6+(rerouted[1,1::2]-6+1)%6
            reroute_score=model(x,rerouted)[1::2]
            rewire_deltas.append(float((reroute_score-scores).abs().mean()))
    assert len(all_scores)==144 and all(math.isfinite(v) for v in all_scores)
    values = torch.tensor(all_scores, dtype=torch.float32)
    return {
        "source_words":144,"min":min(all_scores),"max":max(all_scores),
        "mean":sum(all_scores)/len(all_scores),
        "score_span":max(all_scores)-min(all_scores),
        "min_wave_span":min(per_wave),"max_wave_span":max(per_wave),
        "feature_delta":max(perturb_deltas),"rewire_delta":max(rewire_deltas),
        "positive_predictions":sum(v>0.5 for v in all_scores),
        "pred_sha":sha(values.numpy().tobytes())
    }

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--health",type=Path,required=True)
    parser.add_argument("--words",type=Path,required=True)
    parser.add_argument("--models",type=Path,required=True)
    parser.add_argument("--fixture",type=Path,required=True)
    parser.add_argument("--output",type=Path,required=True)
    args=parser.parse_args()
    torch.manual_seed(20261119)
    torch.set_num_threads(1)
    torch.use_deterministic_algorithms(True)
    source_dir,manifest=source_auth(args.health,args.words,args.models)
    word_blob,events=word_events(args.fixture)
    sys.path.insert(0,str(source_dir.resolve()))
    from gnn_baseline import EdgeLevelGNN
    from prototype_gnn import PrototypeGNN
    from contrastive_gnn import ContrastiveGNN
    from gsl_gnn import GSLGNN

    class Origin: pass
    origin=Origin()
    origin.EdgeLevelGNN=EdgeLevelGNN
    origin.PrototypeGNN=PrototypeGNN
    origin.ContrastiveGNN=ContrastiveGNN
    origin.GSLGNN=GSLGNN

    records=[]
    report=[]
    for kind, name in MODEL_LIST:
        path=args.models/"models"/name
        state=open_checkpoint(path)
        model,dim,hidden=construct(kind,state,origin)
        parameter_count=normalize_keys(state,model)
        model.eval()
        outcome=metrics(model,events,dim)
        record={
            "model":kind,"checkpoint":name,"checkpoint_git_blob":MODEL_BLOBS[name],
            "checkpoint_sha256":sha(path.read_bytes()),
            "features":dim,"hidden":hidden,"parameter_keys":parameter_count,
            **outcome,
            "strict_weights":"PASS","source":"PINNED","labels":"NOT_PRESENT",
            "balanced_accuracy":"NOT_COMPUTED",
        }
        records.append(record)
        line=(
            f'GNN_MODEL_COMPARE|model={kind}|checkpoint={name}|strict=PASS|'
            f'features={dim}|hidden={hidden}|params={parameter_count}|'
            f'checkpoint_sha={record["checkpoint_sha256"]}|'
            f'score_min_nano={round(outcome["min"]*1e9)}|'
            f'score_max_nano={round(outcome["max"]*1e9)}|'
            f'score_span_nano={round(outcome["score_span"]*1e9)}|'
            f'feature_delta_nano={round(outcome["feature_delta"]*1e9)}|'
            f'rewire_delta_nano={round(outcome["rewire_delta"]*1e9)}|'
            f'positive_144={outcome["positive_predictions"]}|'
            f'prediction_sha={outcome["pred_sha"]}|'
            f'accuracy=NOT_COMPUTED|status=PASS|json=0'
        )
        report.append(line)
        print(line,flush=True)

    for name,alias in (
        ("prototype_model_run3.pt","prototype_model.pt"),
        ("contrastive_model_run3.pt","contrastive_model.pt"),
    ):
        assert MODEL_BLOBS[name] == MODEL_BLOBS[alias]
        assert sha((args.models/"models"/name).read_bytes()) == sha((args.models/"models"/alias).read_bytes())
        line=f'GNN_WEIGHT_ALIAS|file={name}|identical_to={alias}|verified=BYTE_EQUAL|status=PASS|json=0'
        report.append(line)
        print(line,flush=True)
    report_body="\n".join(report)+"\n"
    header=(
        f'GNN_CHECKPOINT_COMPARISON|schema=1|healthcare_commit={HEALTH_SHA}|'
        f'word_commit={WORD_SHA}|models_commit={MODEL_SHA}|'
        f'word_fixture_sha={sha(word_blob)}|'
        f'original_corpus_positive={manifest["corpus_stats"]["suspicious"]}|'
        f'original_corpus_negative={manifest["corpus_stats"]["benign"]}|'
        f'unique_models={len(records)}|alias_models=2|heldout_waves=8_11|'
        f'heldout_edges_per_model=144|'
        f'checkpoint_inference=STRICT|truth_labels=UNAVAILABLE|'
        f'balanced_performance=HOLD|'
        f'records_sha={sha(report_body.encode())}|'
        f'torch={torch.__version__}|pyg={torch_geometric.__version__}|'
        f'status=PASS|json=0\n'
    )
    args.output.write_text(header+report_body,encoding="utf-8")
    args.output.with_suffix(".json").write_text(
        json.dumps({"schema":1,"experiment":"source-pinned-actual-checkpoints",
          "source_sha":sha(word_blob),"records":records,
          "class_labels":"UNAVAILABLE","balanced_performance":"HOLD"},sort_keys=True,indent=2)+"\n"
    )
    print(header.strip(),flush=True)

if __name__ == "__main__":
    main()
