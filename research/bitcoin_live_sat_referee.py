#!/usr/bin/env python3
"""Independent standard-library referee for source-pinned native Bitcoin SAT
direct-versus-live full-CNF search and actual 27-cell live decision receipts.

This research-only referee never imports the native solver or matrix codec.
"""
import argparse
import ast
import hashlib
from pathlib import Path

CASES = {
    "nonce32": ("6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3",
                239112, 877879),
    "merkle_nonce288": ("601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b",
                        361188, 1325357),
}
MODES = ("direct", "live")
PREFIX = bytes.fromhex(
    "01000000" + "00" * 32 +
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a" +
    "29ab5f49" + "ffff001d"
)
NONCE = 2083236893
TARGET = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"

def need(ok, reason):
    if not ok:
        raise ValueError(reason)

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

def double_sha(header):
    need(len(header) == 80, "source header byte count changed")
    return hashlib.sha256(hashlib.sha256(header).digest()).digest()[::-1].hex()

def read_hbp(path):
    source = path.read_text("ascii")
    need(source.endswith("\n") and len(source.splitlines()) == 1,
         "receipt must have exactly one line")
    fields = source.strip().split("|")
    need(fields[0] == "PNP_BTC_LIVE_SEARCH", "unexpected receipt family")
    pairs = [item.split("=", 1) for item in fields[1:]]
    need(all(len(item) == 2 for item in pairs), "invalid receipt field")
    result = dict(pairs)
    need(len(result) == len(pairs), "repeated receipt metadata field")
    need(result.get("schema") == "1" and result.get("json") == "0",
         "unexpected receipt schema")
    return result

def positive_fields(row, fields):
    for field in fields:
        need(field in row and row[field].isdigit(), "missing integer field " + field)
        need(int(row[field]) >= 0, "negative meter")
    return {field: int(row[field]) for field in fields}

def verify_cnf_model(cnf, selected, declared_vars, declared_clauses):
    announced = clauses = 0
    pending = []
    with cnf.open("rt", encoding="ascii") as stream:
        for line in stream:
            words = line.split()
            if not words or words[0] == "c":
                continue
            if words[0] == "p":
                need(announced == 0 and words[1] == "cnf" and len(words) == 4,
                     "source DIMACS header")
                need(int(words[2]) == declared_vars and int(words[3]) == declared_clauses,
                     "CNF manifest dimensions differ")
                announced = int(words[3])
                continue
            need(announced != 0, "clauses before DIMACS header")
            for token in words:
                literal = int(token)
                if literal:
                    need(1 <= abs(literal) <= declared_vars, "invalid source SAT literal")
                    pending.append(literal)
                else:
                    need(bool(pending), "empty clause in known SAT source")
                    need(any((abs(lit) in selected) == (lit > 0) for lit in pending),
                         "model fails unmodified original Bitcoin CNF clause")
                    pending.clear()
                    clauses += 1
    need(clauses == declared_clauses and not pending, "incomplete CNF clause replay")

def verify_one(root, source, case, mode):
    expected, var_count, clause_count = CASES[case]
    folder = root / mode / case
    need(folder.is_dir() and (folder / "stdout.log").is_file()
         and (folder / "stderr.log").is_file(), "missing raw worker data")
    record_path = folder / "receipt.hbp"
    timeout_path = folder / "timeout.hbp"
    need(record_path.is_file() != timeout_path.is_file(),
         "exactly one complete or timed-out worker status required")
    if timeout_path.is_file():
        record = timeout_path.read_text("ascii")
        need(record == f"PNP_BTC_TIMEOUT|mode={mode}|case={case}|wall_budget_s=300|"
                       "status=HOLD_TIME|partial_nodes=UNOBSERVED|json=0\n",
             "invalid hosted wall-time HOLD evidence")
        need(not (folder / "model.trits").exists(),
             "time-limited worker must not claim an unverified model")
        return {"mode": mode, "case": case, "status": "HOLD_TIME",
                "nodes": "UNOBSERVED", "propagations": "UNOBSERVED",
                "literal_checks": "UNOBSERVED", "frames": "UNOBSERVED",
                "matrix_ns": "UNOBSERVED", "paid_bytes": "UNOBSERVED",
                "total_ns": "UNOBSERVED", "source": expected}
    row = read_hbp(record_path)
    for key, expected_value in [
        ("mode", mode), ("case", case), ("source_sha256", expected),
        ("variables", str(var_count)), ("clauses", str(clause_count)),
        ("budget_nodes", "128"), ("budget_wall_s", "300"),
        ("routing", "FISCHER_SOURCE_ONLY"), ("known_nonce_injected", "NO"),
        ("actual_solver_state", "YES"),
    ]:
        need(row.get(key) == expected_value, "source/meter identity differs: " + key)
    count = positive_fields(row, [
        "nodes", "propagations", "literal_checks", "branch_frames", "unit_frames",
        "frames", "watcher_reads", "paid_frozen_bytes", "paid_metadata_bytes",
        "matrix_wall_ns", "source_parse_ns", "route_ns", "search_ns", "total_ns",
    ])
    need(0 <= count["nodes"] <= 128, "unbounded search")
    need(count["frames"] == count["branch_frames"] + count["unit_frames"] ==
         count["watcher_reads"], "missing exact live frame watcher")
    need(count["total_ns"] >= count["search_ns"], "impossible measured end-to-end cost")
    if mode == "direct":
        need(all(count[key] == 0 for key in (
            "frames", "watcher_reads", "paid_frozen_bytes",
            "paid_metadata_bytes", "matrix_wall_ns")),
            "direct control executed hidden live matrix work")
        need(row.get("frame_root") == "NONE", "direct control has unexpected proof root")
    else:
        need(len(row.get("frame_root", "")) == 64
             and count["paid_metadata_bytes"] == 192 * count["frames"],
             "missing matrix state evidence or unpaid metadata")
        need(count["paid_frozen_bytes"] > 0 or count["nodes"] == 0,
             "no live matrix frames from observed search")
    status = row.get("status")
    need(status in ("SAT", "HOLD_NODE"), "unexpected native search status")
    if status == "SAT":
        model_path = folder / "model.trits"
        raw = model_path.read_bytes()
        need(sha(raw) == row["model_sha256"], "source model content fingerprint differs")
        need(len(raw) == var_count + 1 and raw[-1:] == b"\n",
             "full SAT assignment length differs")
        need(all(x in (45, 43) for x in raw[:-1]),
             "completed SAT assignment must have only - or + trits")
        selected = {index + 1 for index, letter in enumerate(raw[:-1]) if letter == 43}
        verify_cnf_model(source / f"{case}.cnf", selected, var_count, clause_count)
        manifest = ast.literal_eval((source / f"{case}.txt").read_text("ascii"))
        nonce_mask = (1 << manifest["nonce_free_bits"]) - 1
        nonce_low = sum(1 << bit for bit, var in manifest["nonce_variables"]
                        if var in selected)
        nonce = (NONCE & ~nonce_mask) | nonce_low
        header = bytearray(PREFIX + nonce.to_bytes(4, "little"))
        if manifest["merkle_free_bits"]:
            value = sum(1 << bit for bit, var in manifest["merkle_variables"]
                        if var in selected)
            header[36:68] = value.to_bytes(32, "little")
        need(double_sha(header) == TARGET,
             "SAT witness does not hash to original 256-bit genesis target")
    else:
        need(not (folder / "model.trits").exists() and row.get("model_sha256") == "NONE",
             "HOLD_NODE claims an unjustified model")
    return {"mode": mode, "case": case, "status": status, "source": expected,
            "nodes": str(count["nodes"]),
            "propagations": str(count["propagations"]),
            "literal_checks": str(count["literal_checks"]),
            "frames": str(count["frames"]),
            "matrix_ns": str(count["matrix_wall_ns"]),
            "paid_bytes": str(count["paid_frozen_bytes"] + count["paid_metadata_bytes"]),
            "total_ns": str(count["total_ns"])}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    need(double_sha(PREFIX + NONCE.to_bytes(4, "little")) == TARGET,
         "original Bitcoin genesis source header changed")
    controls = 0
    for flip in (1, 2, 4, 8):
        for index in (36, 76):
            changed = bytearray(PREFIX + NONCE.to_bytes(4, "little"))
            changed[index] ^= flip
            need(double_sha(changed) != TARGET, "altered genesis header passed")
            controls += 1
    for case, (expected, _, _) in CASES.items():
        need(sha((args.source / f"{case}.cnf").read_bytes()) == expected,
             "original Bitcoin source bytes differ")
    results = {(mode, case): verify_one(args.results, args.source, case, mode)
               for mode in MODES for case in CASES}
    parity = 0
    for case in CASES:
        direct = results["direct", case]
        live = results["live", case]
        if direct["status"] != "HOLD_TIME" and live["status"] != "HOLD_TIME":
            need(direct["status"] == live["status"] and
                 direct["nodes"] == live["nodes"] and
                 direct["propagations"] == live["propagations"] and
                 direct["literal_checks"] == live["literal_checks"],
                 "direct/live original-CNF DPLL work diverged under equal node budget")
            parity += 1
    lines = []
    for case in CASES:
        for mode in MODES:
            row = results[mode, case]
            lines.append("BTC_LIVE_TRIAL|case={case}|mode={mode}|status={status}|"
                         "nodes={nodes}|propagations={propagations}|literal_checks={literal_checks}|"
                         "frames={frames}|matrix_ns={matrix_ns}|paid_bytes={paid_bytes}|"
                         "total_ns={total_ns}|source_sha256={source}|json=0".format(**row))
    body = "\n".join(lines) + "\n"
    header = (
        "BTC_LIVE_FINAL|schema=1|source=BITCOIN_GENESIS_SHA256D|"
        "cases=2|modes=2|observations=4|node_budget_each=128|"
        "wall_budget_each_seconds=300|live=ACTUAL_SOLVER_ASSIGNMENTS|"
        f"completed_pair_tree_parity={parity}|wrong_header_controls={controls}|"
        f"rows_sha256={sha(body.encode())}|status=PASS|json=0\n"
    )
    args.out.write_text(header + body, "ascii")
    print(header.rstrip())
    for line in lines:
        print(line)

if __name__ == "__main__":
    main()
