#!/usr/bin/env python3
"""Source-anchored forward inference from an actually recovered Stone-to-Moss archive.

External PyTorch research reference only: never imported into native Rust.
The three-bank recovery referee and native replay remain separate authorities.
"""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import sys

import torch
import stone_moss_recovery_verify as smf
import gnn_original_checkpoint_compare as original

CHECKPOINT = "gnn/gslgnn_w9_3_seq47_v2.pt"
GRAPH = "behcs/word-graph.hbp"


def require(ok, message):
    if not ok:
        raise ValueError(message)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()


def sha(value):
    return hashlib.sha256(value).hexdigest()


def role_hash(name, value):
    return sha(b"SMF_FIVE_MEMBER_V1\x00" + name.encode() + b"\x00" + canonical(value))


def pinned_git(root):
    return subprocess.check_output(
        ["git", "-C", str(root), "rev-parse", "HEAD"], text=True
    ).strip()


def parse_checkpoint(data):
    state = torch.load(io.BytesIO(data), map_location="cpu", weights_only=True)
    if isinstance(state, dict) and "state_dict" in state:
        state = state["state_dict"]
    require(isinstance(state, dict) and len(state) > 4, "recovered checkpoint is not a state dictionary")
    require(all(isinstance(k, str) and isinstance(v, torch.Tensor) for k, v in state.items()),
            "recovered checkpoint state schema changed")
    return state


def build_model(state, origin):
    model, dim, hidden = original.construct("gsl", state, origin)
    require(dim == 6 and hidden == 64, "original GSLGNN shape changed")
    count = original.normalize_keys(state, model)
    model.eval()
    return model, count


def compare_forward(recovered_model, source_model, events):
    records = {}
    with torch.no_grad():
        for wave in (8, 9, 10, 11):
            features, edges = original.graph(events, wave, 6)
            rec = recovered_model(features, edges)
            src = source_model(features, edges)
            require(torch.equal(rec, src), f"recovered/source trained forward parity wave {wave}")
            altered = features.clone()
            altered[12:, 1] = 1 - altered[12:, 1]
            altered[12:, 2] = 1 - altered[12:, 2]
            require(torch.equal(recovered_model(altered, edges), source_model(altered, edges)),
                    f"recovered/source feature-change parity wave {wave}")
            records[str(wave)] = {
                "event_nodes": 36,
                "total_nodes": 48,
                "directed_edges": 72,
                "outputs_equal": True,
                "perturbed_outputs_equal": True,
                "output_micro_sha": sha(b"SMF_WAVE_MICRO_V1\n" + b"".join(
                    struct.pack("<q", round(float(t) * 1_000_000)) for t in rec
                )),
            }
    return records


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--health", type=Path, required=True)
    ap.add_argument("--words", type=Path, required=True)
    ap.add_argument("--models", type=Path, required=True)
    ap.add_argument("--graph", type=Path, required=True)
    ap.add_argument("--shards", type=Path, required=True)
    ap.add_argument("--recovered", type=Path, required=True)
    ap.add_argument("--recovery-receipt", type=Path, required=True)
    ap.add_argument("--run-key", required=True)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()
    require(re.fullmatch(r"[A-Za-z0-9_-]{1,200}", args.run_key) is not None, "invalid run identity")
    torch.manual_seed(20261119)
    torch.set_num_threads(1)
    torch.use_deterministic_algorithms(True)

    # Independent referee already checked all three bank pairs; repeat the
    # archive and bank-root checks here before any model weights are loaded.
    root_dir, manifest = original.source_auth(args.health, args.words, args.models)
    raw = args.recovered.read_bytes()
    entries = smf.parse_archive(raw)
    expected = smf.source_bytes(args.words, args.models, args.graph)
    require(entries == expected, "recovered archive contains source-substituted records")
    require(raw == smf.canonical_archive(expected), "recovered archive is noncanonical")
    rec = json.loads(args.recovery_receipt.read_bytes())
    require(rec["run_key"] == args.run_key, "recovery/inference epoch mismatch")
    require(rec["archive_sha256"] == sha(raw), "recovery source root differs")
    require(set(rec["pairwise_recovery_sha256"]) == {"AB", "AP", "BP"},
            "recovery receipt pair roster changed")
    require(all(v == sha(raw) for v in rec["pairwise_recovery_sha256"].values()),
            "one or more bank-pair roots differ")
    require(rec["verdict"] == "PASS_COMPUTATIONAL_RECOVERY", "recovery owner gate absent")
    require(rec["source_git_pin_verification"] is True, "Git source provenance absent")
    shards = {name: smf.read_shard(args.shards / (name + ".smfshard"), args.run_key)
              for name in ("A", "B", "P")}
    require(all(shards[name]["lane"] == name and shards[name]["root"] == sha(raw)
                for name in shards), "source shard graph differs")
    require(all(smf.restore((shards[a], shards[b])) == raw
                for a, b in (("A", "B"), ("A", "P"), ("B", "P"))),
            "unverified shard predecessor")

    recovered_checkpoint = entries[CHECKPOINT]
    original_checkpoint = (args.models / "models/gslgnn_w9_3_seq47_v2.pt").read_bytes()
    require(sha(recovered_checkpoint) == manifest["checkpoint_sha256"], "checkpoint manifest digest differs")
    require(recovered_checkpoint == original_checkpoint, "actual recovered checkpoint bytes differ")
    require(entries[GRAPH] == args.graph.read_bytes(), "source-derived word graph changed")
    word_blob, events = original.word_events(args.graph)
    require(word_blob == entries[GRAPH], "word event graph is not recovered original")
    sys.path.insert(0, str(root_dir.resolve()))
    from gsl_gnn import GSLGNN

    class Origin:
        pass

    origin = Origin()
    origin.GSLGNN = GSLGNN
    restored_state = parse_checkpoint(recovered_checkpoint)
    original_state = parse_checkpoint(original_checkpoint)
    require(set(restored_state) == set(original_state) and all(
        torch.equal(restored_state[k], original_state[k]) for k in restored_state
    ), "checkpoint tensor parity failed")
    restored_model, parameter_count = build_model(restored_state, origin)
    original_model, reference_count = build_model(original_state, origin)
    require(parameter_count == reference_count, "strict model parameter count differs")
    forward = compare_forward(restored_model, original_model, events)
    metrics_recovered = original.metrics(restored_model, events, 6)
    metrics_original = original.metrics(original_model, events, 6)
    require(metrics_recovered == metrics_original, "recovered versus original model metric mismatch")
    metrics_recovered.pop("raw_host_sha")

    # Source-substitution and epoch-negative tests have independent roots,
    # even when an alternative archive is internally checksum-consistent.
    alternate = dict(entries)
    alternate[CHECKPOINT] = bytes([recovered_checkpoint[0] ^ 1]) + recovered_checkpoint[1:]
    coherent_forgery = smf.canonical_archive(alternate)
    require(smf.parse_archive(coherent_forgery) == alternate, "coherent control malformed")
    require(sha(coherent_forgery) != rec["archive_sha256"], "coherent substituted origin accepted")
    require(sha(alternate[CHECKPOINT]) != manifest["checkpoint_sha256"], "altered checkpoint accepted")
    require(smf.rejects(lambda: smf.read_shard(
        args.shards / "A.smfshard", args.run_key + "-replay")), "cross-epoch shard accepted")

    checkout_sha = pinned_git(Path("."))
    require(re.fullmatch(r"[0-9a-f]{40}", checkout_sha) is not None, "invalid checkout ref")
    if os.getenv("GITHUB_SHA"):
        require(checkout_sha == os.environ["GITHUB_SHA"], "CI checkout does not match GitHub workflow ref")
    source_ids = {
        "pnp_checkout_sha": checkout_sha,
        "word_commit": smf.WORD_COMMIT,
        "trained_model_commit": smf.MODEL_COMMIT,
        "healthcare_origin_commit": original.HEALTH_SHA,
        "archive_sha256": sha(raw),
        "word_fixture_sha256": sha(word_blob),
        "trained_checkpoint_sha256": sha(recovered_checkpoint),
        "healthcare_model_source_blob": original.HEALTH_BLOBS["gsl_gnn.py"],
    }
    workflow = {
        "run_key": args.run_key,
        "epoch": "IS",
        "lane_roster": ["A", "B", "P"],
        "recovery_pairs": ["AB", "AP", "BP"],
        "archive_bytes": len(raw),
        "source_auth": "PINNED_GIT_OBJECTS",
        "model_execution": "RECOVERED_CHECKPOINT_FORWARD",
        "human_release": "NOT_AUTHORIZED",
    }
    graph = {
        "nodes": [
            {"id": "source_words", "type": "PINNED_SOURCE", "sha": smf.WORD_COMMIT},
            {"id": "source_checkpoint", "type": "PINNED_MODEL", "sha": sha(recovered_checkpoint)},
            {"id": "source_healthcare_model", "type": "PINNED_MODEL_CODE",
             "sha": original.HEALTH_BLOBS["gsl_gnn.py"]},
            {"id": "word_graph", "type": "SOURCE_WORDS", "sha": sha(word_blob)},
            *[{"id": "bank_" + k, "type": "INDEPENDENT_GITHUB_WORKER_SHARD",
               "sha": sha((args.shards / (k + ".smfshard")).read_bytes())} for k in ("A", "B", "P")],
            {"id": "recovered_archive", "type": "EXACT_RECONSTRUCTED_ARCHIVE", "sha": sha(raw)},
            {"id": "inference", "type": "MEASURED_TRAINED_FORWARD",
             "sha": metrics_recovered["prediction_micro_sha"]},
        ],
        "edges": [
            {"from": "source_words", "to": "word_graph", "relation": "GENERATES"},
            {"from": "source_checkpoint", "to": "bank_A", "relation": "ENCODED_IN"},
            {"from": "source_checkpoint", "to": "bank_B", "relation": "ENCODED_IN"},
            {"from": "source_checkpoint", "to": "bank_P", "relation": "ENCODED_IN"},
            *[{"from": "bank_" + k, "to": "recovered_archive", "relation": "PARTICIPATES_IN_RECOVERY"}
              for k in ("A", "B", "P")],
            {"from": "recovered_archive", "to": "inference", "relation": "PROVIDES_WEIGHTS"},
            {"from": "word_graph", "to": "inference", "relation": "PROVIDES_INPUT"},
            {"from": "source_healthcare_model", "to": "inference", "relation": "EXECUTES_ARCHITECTURE"},
        ],
    }
    roles = {
        "HBI": role_hash("HBI", source_ids),
        "HBP": role_hash("HBP", workflow),
        "SHA": role_hash("SHA", {"archive": sha(raw), "checkpoint": sha(recovered_checkpoint)}),
        "SH": role_hash("SH", {"graph": graph, "feature_contract": "TRAINER_SIX_FEATURES_PROXY_ADAPTER"}),
    }
    roles["HASH"] = role_hash("HASH", {"roles": roles, "forward": metrics_recovered})
    result = {
        "schema": "SMF_RECOVERED_GNN_FORWARD_PROVENANCE_V1",
        "temporal": {
            "WAS": {"original_word_commit": smf.WORD_COMMIT, "original_model_commit": smf.MODEL_COMMIT},
            "IS": {"workflow_ref": checkout_sha, "run_key": args.run_key,
                   "recovery": "PASS", "trained_forward": "PASS"},
            "WILL": {"balanced_evaluation": "NOT_MEASURED", "future_state": "UNCOMMITTED"},
        },
        "source_ids": source_ids,
        "workflow": workflow,
        "typed_graph": graph,
        "five_roles": roles,
        "metrics": metrics_recovered,
        "wave_forward_parity": forward,
        "strict_loaded_parameters": parameter_count,
        "checkpoint_load": "STRICT",
        "recovered_checkpoint_forward": "PASS",
        "original_checkpoint_forward_parity": "EXACT_SAME_CPU_RUN",
        "coherent_source_substitution_rejected": True,
        "replay_rejected": True,
        "original_labels": "UNAVAILABLE",
        "balanced_accuracy": "NOT_COMPUTED",
        "word_feature_alignment": "EXPERIMENTAL_PROXY",
        "model_score_sensitivity": "MEASURED_SEPARATELY",
        "external_source_attested": False,
        "github_live_branch_refs_measured": False,
        "verdict": "PASS_RECOVERED_CHECKPOINT_FORWARD",
    }
    args.out.mkdir(parents=True, exist_ok=True)
    json_path = args.out / "SMF_GNN_FORWARD.json"
    hbp_path = args.out / "SMF_GNN_FORWARD.hbp"
    json_path.write_bytes(canonical(result) + b"\n")
    hbp = (
        f'SMF_GNN_FORWARD|schema=1|run={args.run_key}|archive_sha={sha(raw)}|'
        f'checkpoint_sha={sha(recovered_checkpoint)}|word_graph_sha={sha(word_blob)}|'
        f'prediction_micro_sha={metrics_recovered["prediction_micro_sha"]}|'
        f'five_member_root={roles["HASH"]}|bank_pairs=AB_AP_BP|'
        f'checkpoint_load=STRICT|recovered_forward=PASS|'
        f'original_source_forward_parity=EXACT|balanced_accuracy=NOT_COMPUTED|'
        f'source_attested=PINNED_GIT_ONLY|status=PASS|json=0\n'
    )
    hbp_path.write_text(hbp)
    (args.out / "SMF_GNN_FORWARD.sha256").write_text(
        f"{sha(json_path.read_bytes())}  {json_path.name}\n"
        f"{sha(hbp_path.read_bytes())}  {hbp_path.name}\n"
    )
    print(hbp.strip(), flush=True)


if __name__ == "__main__":
    main()
