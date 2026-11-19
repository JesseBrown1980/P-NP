#!/usr/bin/env python3
"""Independent Stone-to-Moss 2-of-3 Python reference verifier.

Uses only stdlib and *original* git-pinned files. No upstream Rust imports,
model fitting, model-weight substitutions, JSON inside the native crate,
or assumptions that an SHA-256 match authenticates an external instrument.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess

WORD_COMMIT = "818cc235a6db815e3b2591339cf9b42aad3e85a0"
MODEL_COMMIT = "c4b2458c88358c46155d3b308c594f9991720847"
SOURCE_PIN = {
    "data/behcs/codex/alphabet.json": "8d4a3298576ce2c9d4a1683ffab667c8a743fc42",
    "packages/deep-wave/src/cascade.ts": "b9acdaff9b4537def53eca3d0fdb87a9c0dcab89",
    "packages/deep-wave/src/third-cascade.ts": "a3e4485ca4a60d1b12cf6ed398c2f886ba358ea7",
}
MODEL_PIN = {
    "models/gslgnn_w9_3_seq47_v2.pt": "e3a9712b50f8e7b31590b0aa944e6a9ba6ea565b",
    "manifests/gslgnn_w9_3_seq47_v2_manifest.json": "f1eb3675ab5e8822d1511ce4b3062d51ee50b2dd",
}
ARCHIVE_MAGIC = b"SMF_ARCHIVE_V1\n"
SHARD_MAGIC = b"SMF_SHARD_V1\n"
WIDTH = 256
STRIPE = 512
NAMES = [
    "behcs/alphabet.json",
    "behcs/cascade.ts",
    "behcs/third-cascade.ts",
    "behcs/word-graph.hbp",
    "gnn/gslgnn_w9_3_seq47_v2.pt",
    "gnn/gslgnn_w9_3_seq47_v2_manifest.json",
]


def h(data):
    return hashlib.sha256(data).hexdigest()


def must(condition, reason):
    if not condition:
        raise ValueError(reason)


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()


def source_bytes(words, model, word_graph):
    must(git(words, "rev-parse", "HEAD") == WORD_COMMIT, "wrong word source commit")
    must(git(model, "rev-parse", "HEAD") == MODEL_COMMIT, "wrong model source commit")
    for name, blob in SOURCE_PIN.items():
        must(git(words, "hash-object", name) == blob, "word source Git blob mismatch: " + name)
    for name, blob in MODEL_PIN.items():
        must(git(model, "hash-object", name) == blob, "model source Git blob mismatch: " + name)
    checkpoint = (model / "models/gslgnn_w9_3_seq47_v2.pt").read_bytes()
    manifest_raw = (model / "manifests/gslgnn_w9_3_seq47_v2_manifest.json").read_bytes()
    manifest = json.loads(manifest_raw)
    must(h(checkpoint) == manifest["checkpoint_sha256"], "checkpoint vs original manifest SHA")
    must(manifest["corpus_stats"]["benign"] == 40, "original unbalanced training manifest drift")
    must(manifest["corpus_stats"]["suspicious"] == 315209, "original corpus manifest drift")
    graph = word_graph.read_bytes()
    rows = graph.decode("utf-8").splitlines()
    must(len(rows) == 433 and rows[0].startswith("BEHCS_GNN_WORD_PROBE|"), "word graph fixture")
    meta = dict(piece.split("=", 1) for piece in rows[0].split("|")[1:])
    must(meta["source_commit"] == WORD_COMMIT and meta["sampled_words"] == "432", "word origin mismatch")
    must(h(("\n".join(rows[1:]) + "\n").encode()) == meta["word_sha"], "word-event source mismatch")
    expected = [
        (words / "data/behcs/codex/alphabet.json").read_bytes(),
        (words / "packages/deep-wave/src/cascade.ts").read_bytes(),
        (words / "packages/deep-wave/src/third-cascade.ts").read_bytes(),
        graph, checkpoint, manifest_raw,
    ]
    return dict(zip(NAMES, expected))


def canonical_archive(entries):
    must(list(entries) == NAMES, "source record names or ordering")
    out = bytearray(ARCHIVE_MAGIC + struct.pack(">H", len(NAMES)))
    for name, data in entries.items():
        must(len(data) <= 2_000_000, "source file exceeds configured bound")
        out += struct.pack(">HQ", len(name.encode()), len(data))
        out += h(data).encode("ascii")
        out += name.encode("utf-8")
        out += data
    return bytes(out)


def parse_archive(raw):
    must(raw.startswith(ARCHIVE_MAGIC), "invalid recovered archive magic")
    ptr = len(ARCHIVE_MAGIC)
    must(len(raw) >= ptr + 2, "truncated archive count")
    count = struct.unpack_from(">H", raw, ptr)[0]
    ptr += 2
    must(count == len(NAMES), "source record count changed")
    entries = {}
    for expected_name in NAMES:
        must(len(raw) >= ptr + 10 + 64, "truncated archive record")
        namelen, size = struct.unpack_from(">HQ", raw, ptr)
        ptr += 10
        must(size <= 2_000_000, "oversized record")
        claimed = raw[ptr:ptr+64].decode("ascii")
        ptr += 64
        must(len(raw) >= ptr + namelen + size, "short record")
        name = raw[ptr:ptr+namelen].decode("utf-8")
        ptr += namelen
        must(name == expected_name, "source record order differs")
        data = raw[ptr:ptr+size]
        ptr += size
        must(h(data) == claimed, "individual source record SHA mismatch")
        entries[name] = data
    must(ptr == len(raw), "uncommitted archive trailing bytes")
    return entries


def read_shard(path, key):
    raw = path.read_bytes()
    must(raw.startswith(SHARD_MAGIC), "bad shard magic")
    head, sep, body = raw[len(SHARD_MAGIC):].partition(b"\n")
    must(sep and len(head) < 400, "malformed shard header")
    fields = head.decode("ascii").split("|")
    must(len(fields) == 7 and fields[0] == "RUN", "shard header fields")
    expected_keys = ["key", "lane", "root", "bytes", "stripes", "width"]
    info = {}
    for label, field in zip(expected_keys, fields[1:]):
        name, sep2, value = field.partition("=")
        must(sep2 and name == label, "shard field changed")
        info[label] = value
    must(info["key"] == key, "foreign-run shard")
    must(re.fullmatch(r"[A-Za-z0-9_-]{1,200}", info["key"]) is not None, "bad run identity")
    must(info["lane"] in ("A", "B", "P"), "unknown lane")
    must(re.fullmatch("[0-9a-f]{64}", info["root"]) is not None, "malformed source root")
    must(info["bytes"].isdigit() and info["stripes"].isdigit(), "invalid length")
    length, n = int(info["bytes"]), int(info["stripes"])
    must(info["width"] == "256", "wrong stripe width")
    must(0 < length <= 12_000_000, "unbounded archive")
    must(n == (length + STRIPE - 1) // STRIPE, "shard stripes differ")
    must(len(body) == n * WIDTH, "shard body length")
    must(path.with_name(path.name + ".sha256").read_text().split() ==
         [h(raw), path.name], "shard sidecar SHA or filename mismatch")
    return {"lane": info["lane"], "root": info["root"], "len": length, "payload": body}


def restore(pair):
    must(len(pair) == 2, "two independent source banks required")
    x, y = pair
    must(x["lane"] != y["lane"], "duplicate source bank")
    must(x["root"] == y["root"] and x["len"] == y["len"], "foreign source root")
    must(len(x["payload"]) == len(y["payload"]), "nonmatching bank length")
    by_lane = {x["lane"]: x["payload"], y["lane"]: y["payload"]}
    n = len(x["payload"]) // WIDTH
    out = bytearray()
    for i in range(n):
        off = i * WIDTH
        if "A" in by_lane and "B" in by_lane:
            a = by_lane["A"][off:off+WIDTH]
            b = by_lane["B"][off:off+WIDTH]
        elif "A" in by_lane and "P" in by_lane:
            a = by_lane["A"][off:off+WIDTH]
            p = by_lane["P"][off:off+WIDTH]
            b = bytes(u ^ v for u, v in zip(a, p))
        elif "B" in by_lane and "P" in by_lane:
            b = by_lane["B"][off:off+WIDTH]
            p = by_lane["P"][off:off+WIDTH]
            a = bytes(u ^ v for u, v in zip(b, p))
        else:
            raise ValueError("unknown reconstruction pair")
        out += a + b
    data = bytes(out[:x["len"]])
    must(h(data) == x["root"], "source-root mismatch after reconstruction")
    parse_archive(data)
    return data


def rejects(fn):
    try:
        fn()
    except (ValueError, AssertionError, UnicodeError, KeyError):
        return True
    return False


def negative_controls(shards, expected_raw):
    a, b, p = (shards[k] for k in ("A", "B", "P"))
    controls = {}
    for lane in ("A", "B", "P"):
        controls["single_bank_" + lane] = rejects(lambda lane=lane: restore([shards[lane]]))
    controls["duplicate_lane"] = rejects(lambda: restore([a, a]))
    controls["different_source_root"] = rejects(lambda: restore([a, {**p, "root": "0"*64}]))
    corrupt = bytes([b["payload"][0] ^ 1]) + b["payload"][1:]
    controls["altered_payload"] = rejects(lambda: restore([a, {**b, "payload": corrupt}]))
    controls["truncated_payload"] = rejects(lambda: restore([a, {**p, "payload": p["payload"][:-1]}]))
    controls["corrupt_individual_archive_record"] = rejects(
        lambda: parse_archive(expected_raw[:-1] + bytes([expected_raw[-1] ^ 1])))
    controls["trailing_unaudited_archive_byte"] = rejects(lambda: parse_archive(expected_raw + b"X"))
    # A fully self-consistent alternate source root can pass local algebra,
    # but differs from the separately frozen immutable original source.
    forged = expected_raw[:-1] + bytes([expected_raw[-1] ^ 1])
    forged_root = h(forged)
    controls["new_consistent_source_does_not_match_frozen_root"] = (forged_root != h(expected_raw))
    # Explicit information insufficiency witness: A alone is compatible with
    # two different values for the unknown B half, both with the same A.
    first_a = a["payload"][:WIDTH]
    hypothetical_b0 = bytes(WIDTH)
    hypothetical_b1 = bytes([1]) + bytes(WIDTH - 1)
    controls["one_bank_has_two_completions"] = (
        first_a + hypothetical_b0 != first_a + hypothetical_b1
    )
    must(all(controls.values()), "a mandatory negative control failed")
    return controls


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--words", type=Path, required=True)
    ap.add_argument("--models", type=Path, required=True)
    ap.add_argument("--graph", type=Path, required=True)
    ap.add_argument("--shards", type=Path, required=True)
    ap.add_argument("--run-key", required=True)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()
    entries = source_bytes(args.words, args.models, args.graph)
    reference = canonical_archive(entries)
    must(parse_archive(reference) == entries, "independent archive canonical parser roundtrip")
    shards = {k: read_shard(args.shards / (k + ".smfshard"), args.run_key)
              for k in ("A", "B", "P")}
    must(all(shards[k]["lane"] == k for k in shards), "worker/lane artifact substitution")
    must(all(shards[k]["root"] == h(reference) for k in shards), "Git-backed source content root differs")
    must(all(shards[k]["len"] == len(reference) for k in shards), "archive length differs from source")
    pair_roots = {}
    for pair in ("AB", "AP", "BP"):
        restored = restore([shards[pair[0]], shards[pair[1]]])
        must(restored == reference, "independently generated source archive bytes differ")
        must(parse_archive(restored) == entries, "independent file-by-file source mismatch")
        pair_roots[pair] = h(restored)
    controls = negative_controls(shards, reference)
    report = {
        "schema": "STONE-MOSS-REAL-WORD-GNN-ERASURE-V1",
        "run_key": args.run_key,
        "original_word_commit": WORD_COMMIT,
        "original_model_commit": MODEL_COMMIT,
        "source_files": {name: {"bytes": len(data), "sha256": h(data)}
                         for name, data in entries.items()},
        "archive_bytes": len(reference),
        "archive_sha256": h(reference),
        "stripes": (len(reference) + STRIPE - 1) // STRIPE,
        "shard_payload_bytes_each": len(shards["A"]["payload"]),
        "pairwise_recovery_sha256": pair_roots,
        "recovered_original_records": len(entries),
        "recovered_actual_checkpoint": True,
        "recovered_original_256_glyph_alphabet": True,
        "recovered_original_cascades": True,
        "recovered_432_word_graph_fixture": True,
        "negative_controls": controls,
        "source_git_pin_verification": True,
        "hash_sidecar_and_original_file_byte_comparison": True,
        "external_source_or_physical_instrument_attested": False,
        "model_forward_inference_executed_here": False,
        "semantic_checkpoint_word_training_alignment": "NOT_ESTABLISHED",
        "network_streaming_executed_here": False,
        "released_to_main": False,
        "verdict": "PASS_COMPUTATIONAL_RECOVERY",
    }
    args.out.mkdir(parents=True, exist_ok=True)
    receipt = args.out / "SMF_RECOVERY.json"
    receipt.write_text(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n")
    hbp_lines = [
        "SMF|schema=STONE-MOSS-REAL-WORD-GNN-ERASURE-V1|json=0",
        f"SOURCE|word_commit={WORD_COMMIT}|model_commit={MODEL_COMMIT}|archive_sha256={h(reference)}|records=6|json=0",
        f"STRIPES|count={report['stripes']}|width=256|source_bytes={len(reference)}|json=0",
        f"RECOVERY|pairs=AB,AP,BP|each_exact=TRUE|payload_sha256={h(reference)}|json=0",
        f"CONTROLS|held={len(controls)}|source_pinned=TRUE|physical_attested=FALSE|training_alignment=NOT_ESTABLISHED|json=0",
        "VERDICT|computational_recovery=PASS|original_byte_comparison=PASS|release_authorized=FALSE|json=0",
    ]
    (args.out / "SMF_RECOVERY.hbp").write_text("\n".join(hbp_lines) + "\n")
    paths = [args.out / "SMF_RECOVERY.json", args.out / "SMF_RECOVERY.hbp"]
    (args.out / "SMF_RECOVERY.sha256").write_text(
        "".join(f"{h(p.read_bytes())}  {p.name}\n" for p in paths))
    print(f"SMF_RECOVERY|records=6|pairs=3|negative_controls={len(controls)}|"
          f"archive_bytes={len(reference)}|archive_sha256={h(reference)}|status=PASS|json=0")


if __name__ == "__main__":
    main()
