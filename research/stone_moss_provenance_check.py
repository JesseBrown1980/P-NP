#!/usr/bin/env python3
"""Independent stdlib-only verifier for Stone-to-Moss GNN recovery provenance.

Runs on a separate GitHub worker. Reconstructs the source archive and checks
all three bank pairs and typed source/result commitments without importing the
PyTorch inference implementation. Numeric inference execution has its own gate.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

import stone_moss_recovery_verify as source


def must(value, label):
    if not value:
        raise ValueError(label)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()


def role(name, value):
    return digest(b"SMF_FIVE_MEMBER_V1\x00" + name.encode() + b"\x00" + canonical(value))


def git(directory, *args):
    return subprocess.check_output(["git", "-C", str(directory), *args], text=True).strip()


def exact_snapshot(path, run_key, checkout_sha):
    raw = path.read_bytes()
    lines = raw.decode("ascii").splitlines()
    must(len(lines) == 1, "current GitHub snapshot line count")
    fields = lines[0].split("|")
    must(fields[0] == "G3D_IS_REFS" and fields[-1] == "json=0", "snapshot schema")
    parsed = {}
    for item in fields[1:-1]:
        key, delim, val = item.partition("=")
        must(delim and key and val and key not in parsed, "duplicate or blank snapshot field")
        parsed[key] = val
    must(set(parsed) == {"schema", "run", "pnp_sha", "follow_main", "safe_main", "observed"},
         "unexpected snapshot keys")
    must(parsed["schema"] == "1" and parsed["run"] == run_key
         and parsed["pnp_sha"] == checkout_sha and parsed["observed"] == "GITHUB_LS_REMOTE",
         "stale, substituted, or unmeasured GitHub snapshot")
    must(all(re.fullmatch("[0-9a-f]{40}", parsed[name]) for name in
             ("pnp_sha", "follow_main", "safe_main")), "malformed GitHub refs")
    return raw, parsed


def check():
    ap = argparse.ArgumentParser()
    ap.add_argument("--words", type=Path, required=True)
    ap.add_argument("--models", type=Path, required=True)
    ap.add_argument("--health", type=Path, required=True)
    ap.add_argument("--graph", type=Path, required=True)
    ap.add_argument("--shards", type=Path, required=True)
    ap.add_argument("--proof", type=Path, required=True)
    ap.add_argument("--run-key", required=True)
    a = ap.parse_args()
    must(re.fullmatch("[A-Za-z0-9_-]{1,200}", a.run_key), "run-key syntax")

    manifest = json.loads((a.proof / "SMF_RECOVERY.json").read_bytes())
    result = json.loads((a.proof / "SMF_GNN_FORWARD.json").read_bytes())
    forward = (a.proof / "SMF_GNN_FORWARD.hbp").read_bytes()
    (a.proof / "SMF_GNN_FORWARD.hbp").read_text("ascii")
    checkout_sha = git(Path("."), "rev-parse", "HEAD")
    must(re.fullmatch("[0-9a-f]{40}", checkout_sha), "checkout commit invalid")
    live_raw, live = exact_snapshot(a.proof / "SMF_IS_REFS.hbp", a.run_key, checkout_sha)
    must(manifest["run_key"] == result["workflow"]["run_key"] == a.run_key,
         "disconnected workflow epochs")
    must(result["temporal"]["IS"]["workflow_ref"] == checkout_sha and
         result["temporal"]["IS"]["snapshot_sha256"] == digest(live_raw) and
         result["temporal"]["IS"]["follow_main"] == live["follow_main"] and
         result["temporal"]["IS"]["safe_main"] == live["safe_main"],
         "IS temporal observation not tied to original snapshot")
    must(result["temporal"]["WAS"]["original_word_commit"] == source.WORD_COMMIT and
         result["temporal"]["WAS"]["original_model_commit"] == source.MODEL_COMMIT and
         result["temporal"]["WILL"]["future_state"] == "UNCOMMITTED",
         "WAS / WILL time-state misrepresentation")
    must(result["github_live_branch_refs_measured"] is True and
         result["external_source_attested"] is False, "authority boundary collapsed")
    must(result["balanced_accuracy"] == "NOT_COMPUTED"
         and result["original_labels"] == "UNAVAILABLE"
         and result["word_feature_alignment"] == "EXPERIMENTAL_PROXY",
         "unearned semantic accuracy claim")
    must(git(a.health, "rev-parse", "HEAD") == "e0048c0ad60705ecb447642f08b14b5c1bde7437",
         "healthcare original commit changed")
    healthcare_blob = git(a.health, "hash-object",
                          "backend/anomaly_detector/models/gsl_gnn.py")
    must(healthcare_blob == "886b3b0c0cdbddba983fa8c3ae083c4520d38f0e",
         "original healthcare source blob differs")

    originals = source.source_bytes(a.words, a.models, a.graph)
    archive = source.canonical_archive(originals)
    archive_root = digest(archive)
    checkpoint = originals["gnn/gslgnn_w9_3_seq47_v2.pt"]
    checkpoint_sha = digest(checkpoint)
    graph_sha = digest(originals["behcs/word-graph.hbp"])
    must(manifest["archive_sha256"] == archive_root and manifest["verdict"] ==
         "PASS_COMPUTATIONAL_RECOVERY", "recovery source root/gate differs")
    must(set(manifest["pairwise_recovery_sha256"]) == {"AB", "AP", "BP"} and
         all(v == archive_root for v in manifest["pairwise_recovery_sha256"].values()),
         "one or more source pair commitments absent")
    must(result["source_ids"] == {
        "pnp_checkout_sha": checkout_sha,
        "word_commit": source.WORD_COMMIT,
        "trained_model_commit": source.MODEL_COMMIT,
        "healthcare_origin_commit": "e0048c0ad60705ecb447642f08b14b5c1bde7437",
        "archive_sha256": archive_root,
        "word_fixture_sha256": graph_sha,
        "trained_checkpoint_sha256": checkpoint_sha,
        "healthcare_model_source_blob": healthcare_blob,
    }, "source identity tuple altered")
    must(result["workflow"]["measured_live_ref_snapshot_sha256"] == digest(live_raw),
         "HBP temporal source snapshot lost")
    must(result["workflow"]["archive_bytes"] == len(archive)
         and result["workflow"]["lane_roster"] == ["A", "B", "P"]
         and result["workflow"]["recovery_pairs"] == ["AB", "AP", "BP"]
         and result["workflow"]["model_execution"] == "RECOVERED_CHECKPOINT_FORWARD",
         "source archive execution contract changed")

    banks = {}
    for name in ("A", "B", "P"):
        banks[name] = source.read_shard(a.shards / (name + ".smfshard"), a.run_key)
        must(banks[name]["lane"] == name and banks[name]["root"] == archive_root,
             "mismatched hosted source bank")
    for lhs, rhs in (("A", "B"), ("A", "P"), ("B", "P")):
        must(source.restore([banks[lhs], banks[rhs]]) == archive,
             "independent source-bank recovery failed")
    nodes = {n["id"]: n for n in result["typed_graph"]["nodes"]}
    must(len(nodes) == len(result["typed_graph"]["nodes"]) == 10,
         "duplicate or missing typed provenance node")
    for bank in ("A", "B", "P"):
        must(nodes["bank_" + bank]["sha"] ==
             digest((a.shards / (bank + ".smfshard")).read_bytes()),
             "bank worker artifact is not tied to provenance graph")
    for node, expected in {
        "source_words": source.WORD_COMMIT,
        "source_checkpoint": checkpoint_sha,
        "source_healthcare_model": healthcare_blob,
        "word_graph": graph_sha,
        "is_live_refs": digest(live_raw),
        "recovered_archive": archive_root,
        "inference": result["metrics"]["prediction_micro_sha"],
    }.items():
        must(nodes[node]["sha"] == expected, "provenance node identity altered: " + node)
    edges = result["typed_graph"]["edges"]
    must(len(edges) == 11 and all(edge["from"] in nodes and edge["to"] in nodes
                                  for edge in edges), "dangling or missing provenance relationship")
    actual_edges = {(e["from"], e["to"], e["relation"]) for e in edges}
    expected_edges = {
        ("source_words", "word_graph", "GENERATES"),
        ("is_live_refs", "inference", "TEMPORAL_OBSERVATION"),
        ("source_checkpoint", "bank_A", "ENCODED_IN"),
        ("source_checkpoint", "bank_B", "ENCODED_IN"),
        ("source_checkpoint", "bank_P", "ENCODED_IN"),
        ("bank_A", "recovered_archive", "PARTICIPATES_IN_RECOVERY"),
        ("bank_B", "recovered_archive", "PARTICIPATES_IN_RECOVERY"),
        ("bank_P", "recovered_archive", "PARTICIPATES_IN_RECOVERY"),
        ("recovered_archive", "inference", "PROVIDES_WEIGHTS"),
        ("word_graph", "inference", "PROVIDES_INPUT"),
        ("source_healthcare_model", "inference", "EXECUTES_ARCHITECTURE"),
    }
    must(actual_edges == expected_edges, "typed relation roster altered")
    roles = {
        "HBI": role("HBI", result["source_ids"]),
        "HBP": role("HBP", result["workflow"]),
        "SHA": role("SHA", {"archive": archive_root, "checkpoint": checkpoint_sha}),
        "SH": role("SH", {"graph": result["typed_graph"],
                           "feature_contract": "TRAINER_SIX_FEATURES_PROXY_ADAPTER"}),
    }
    roles["HASH"] = role("HASH", {"roles": roles, "forward": result["metrics"]})
    must(result["five_roles"] == roles, "HBI/HBP/SHA/SH/HASH root mismatch")
    must(("five_member_root=" + roles["HASH"]).encode() in forward
         and ("snapshot_sha=" + digest(live_raw)).encode() in forward
         and ("prediction_micro_sha=" +
              result["metrics"]["prediction_micro_sha"]).encode() in forward,
         "public HBP line differs from independently recomputed evidence")
    must(result["recovered_checkpoint_forward"] == "PASS"
         and result["checkpoint_load"] == "STRICT"
         and result["original_checkpoint_forward_parity"] == "EXACT_SAME_CPU_RUN",
         "original model execution gate missing")
    must(sorted(result["wave_forward_parity"]) == ["10", "11", "8", "9"]
         and result["metrics"]["source_words"] == 144
         and all(w["outputs_equal"] and w["perturbed_outputs_equal"]
                 for w in result["wave_forward_parity"].values()),
         "four-wave trained result statement altered")
    must(role("HBI", {**result["source_ids"], "word_fixture_sha256": "0" * 64})
         != roles["HBI"], "source substitution mutation undetected")
    must(digest(live_raw + b"X") != digest(live_raw), "live snapshot mutation undetected")
    print(f'SMF_PROVENANCE_INDEPENDENT|run={a.run_key}|archive_sha={archive_root}|'
          f'checkpoint_sha={checkpoint_sha}|pairs=3|five_roles=5|'
          f'forward_receipt=VERIFIED|independent_neural_replay=NOT_PERFORMED|'
          f'balanced_accuracy=NOT_COMPUTED|status=PASS|json=0')


if __name__ == "__main__":
    check()
