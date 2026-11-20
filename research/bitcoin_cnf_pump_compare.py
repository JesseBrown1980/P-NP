#!/usr/bin/env python3
"""Independent original-input Bitcoin direct-vs-Rust-matrix SAT referee.

Checks every reconstructed source byte, exact original solver receipts,
full target SHA256d for SAT, all source CNF clauses of a SAT model, and
fully charged matrix transport timing. Does not import Rust pump code.
"""
import argparse
import ast
import hashlib
from pathlib import Path

CASES = {
    "nonce32": "6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3",
    "merkle_nonce288": "601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b",
}
PREFIX = bytes.fromhex(
    "01000000" + "00" * 32 +
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a" +
    "29ab5f49" + "ffff001d"
)
NONCE = 2083236893
TARGET = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"

def need(ok, why):
    if not ok:
        raise ValueError(why)

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

def target_hash(header):
    need(len(header) == 80, "not an 80-byte Bitcoin header")
    return hashlib.sha256(hashlib.sha256(header).digest()).digest()[::-1].hex()

def fields(path, prefix):
    rows = path.read_text("ascii").splitlines()
    need(len(rows) == 1 and rows[0].startswith(prefix + "|"), "bad pump witness line")
    entries = rows[0].split("|")[1:]
    items = {}
    for item in entries:
        key, sep, val = item.partition("=")
        need(bool(sep) and key not in items and key, "duplicate or invalid receipt field")
        items[key] = val
    need(items.get("json") == "0" and items.get("status") == "PASS",
         "unaccepted pump receipt")
    return items

def independent_sat_model(stdout):
    model = {}
    terminated = False
    for line in stdout.splitlines():
        if not line.startswith("v "):
            continue
        for token in line.split()[1:]:
            value = int(token)
            need(not terminated, "model continuation after termination")
            if value == 0:
                terminated = True
            else:
                need(abs(value) not in model and abs(value) > 0, "duplicate SAT variable")
                model[abs(value)] = value > 0
    need(terminated and bool(model), "SAT lacks full external solver assignment")
    return model

def check_original_cnf(path, model, variables, clauses):
    seen = 0
    pending = []
    header_seen = False
    with path.open(encoding="ascii") as source:
        for text in source:
            parts = text.split()
            if not parts or parts[0] == "c":
                continue
            if parts[0] == "p":
                need(not header_seen and len(parts) == 4 and parts[1] == "cnf" and
                     int(parts[2]) == variables and int(parts[3]) == clauses,
                     "DIMACS source dimensions changed")
                header_seen = True
                continue
            need(header_seen, "clauses before DIMACS header")
            for token in parts:
                lit = int(token)
                if lit == 0:
                    need(bool(pending) and
                         any(model.get(abs(v)) == (v > 0) for v in pending),
                         "original Bitcoin CNF SAT witness rejected")
                    pending.clear()
                    seen += 1
                else:
                    need(1 <= abs(lit) <= variables, "literal outside original bound")
                    pending.append(lit)
    need(header_seen and seen == clauses and not pending, "truncated DIMACS clauses")

def independently_verify_sat(case, cnf, manifest, raw_result, path):
    model = independent_sat_model((path / "solver.stdout").read_text("ascii"))
    check_original_cnf(cnf, model, manifest["variables"], manifest["clauses"])
    bits = manifest["nonce_free_bits"]
    low = sum(1 << bit for bit, variable in manifest["nonce_variables"]
              if model.get(variable) is True)
    nonce = (NONCE & ~((1 << bits) - 1)) | low
    candidate = bytearray(PREFIX + nonce.to_bytes(4, "little"))
    if manifest["merkle_free_bits"]:
        new_merkle = sum(1 << bit for bit, variable in manifest["merkle_variables"]
                         if model.get(variable) is True)
        candidate[36:68] = new_merkle.to_bytes(32, "little")
    need(target_hash(candidate) == TARGET and
         sha(candidate) == raw_result["witness_sha256"] and
         raw_result["recovered_nonce"] == nonce and
         raw_result["recovered_merkle_raw_hex"] == candidate[36:68].hex(),
         "independent original Bitcoin double-hash witness rejected")

def check(args):
    need(target_hash(PREFIX + NONCE.to_bytes(4, "little")) == TARGET,
         "Bitcoin genesis reference failed")
    for index in [1, 2, 4, 8]:
        changed = bytearray(PREFIX + NONCE.to_bytes(4, "little"))
        changed[36] ^= index
        need(target_hash(changed) != TARGET, "altered Merkle false-green")
        changed = bytearray(PREFIX + NONCE.to_bytes(4, "little"))
        changed[76] ^= index
        need(target_hash(changed) != TARGET, "altered nonce false-green")
    rows = []
    status_counts = {"SAT": 0, "HOLD_TIME": 0}
    for case, expected in CASES.items():
        original = (args.source / (case + ".cnf")).read_bytes()
        reconstructed = (args.results / case / "matrix" / (case + ".cnf")).read_bytes()
        need(sha(original) == expected and reconstructed == original,
             "full-CNF matrix inverse differs from original Bitcoin source")
        manifest = ast.literal_eval((args.source / (case + ".txt")).read_text("ascii"))
        need(manifest["input_sha256"] == expected and manifest["name"] == case,
             "original source manifest drift")
        pump = fields(args.results / case / "matrix" / (case + ".hbp"),
                      "BTC_CNF_PUMP")
        need(pump["case"] == case and pump["source_sha256"] == expected and
             pump["restored_sha256"] == expected and
             int(pump["source_bytes"]) == len(original) and
             int(pump["frames_27_trits"]) == (len(original) + 3) // 4 and
             pump["full_frames_reversed"] == pump["frames_27_trits"] and
             int(pump["frozen_brown_checkpoints"]) > 0 and
             int(pump["brown_paid_witness_bytes"]) > 0 and
             pump["original_cnf_for_solver"] == "BYTE_IDENTICAL" and
             pump["sha256d_search"] == "NOT_RUN" and
             len(pump["witness_proof_sha256"]) == 64,
             "CNF matrix receipt incomplete")
        pump_ns = int(pump["pump_wall_ns"])
        need(pump_ns > 0, "matrix transport work was not metered")
        for route in ("direct", "matrix"):
            evidence = args.results / case / route / "cadical" / case
            outcome = ast.literal_eval((evidence / "receipt.txt").read_text("ascii"))
            need(outcome["arm"] == "cadical" and outcome["case"] == case and
                 outcome["cnf_sha256"] == expected and
                 outcome["limit_seconds"] == 300 and
                 outcome["variables"] == manifest["variables"] and
                 outcome["clauses"] == manifest["clauses"],
                 "original solver or budget changed between routes")
            need((evidence / "solver.stdout").is_file() and
                 (evidence / "solver.stderr").is_file() and
                 (evidence / "rss.txt").is_file(),
                 "missing original SAT execution evidence")
            status = outcome["status"]
            need(status in status_counts and outcome["elapsed_ns"] > 0,
                 "invalid SAT run status or timing")
            if status == "SAT":
                need(outcome["solver_exit"] == 10, "SAT return code mismatch")
                independently_verify_sat(
                    case, args.source / (case + ".cnf"), manifest, outcome, evidence
                )
            else:
                need(outcome["solver_exit"] == 124 and
                     outcome["recovered_nonce"] == "NONE" and
                     outcome["witness_sha256"] == "NONE", "false HOLD witness")
            status_counts[status] += 1
            total = outcome["elapsed_ns"] + (pump_ns if route == "matrix" else 0)
            rows.append(
                f"BTC_CNF_COMPARE|case={case}|route={route}|solver=cadical|"
                f"status={status}|source_sha256={expected}|"
                f"solver_wall_ns={outcome['elapsed_ns']}|"
                f"pump_wall_ns={pump_ns if route == 'matrix' else 0}|"
                f"accounted_total_ns={total}|"
                f"solver_budget_seconds=300|full_cnf_identity=PASS|json=0"
            )
    body = "\n".join(rows) + "\n"
    final = (
        f"BTC_CNF_COMPARISON_FINAL|schema=1|cases=2|routes=2|solver=cadical|"
        f"observations=4|sat={status_counts['SAT']}|hold_time={status_counts['HOLD_TIME']}|"
        f"budget_solver_seconds=300|source_byte_equivalence=PASS|"
        f"independent_original_clause_check=PASS|"
        f"altered_header_controls=8|receipt_rows_sha256={sha(body.encode())}|"
        f"status=PASS|json=0\n"
    )
    args.out.write_text(final + body, encoding="ascii")
    print(final.strip())
    print(body, end="")

def main():
    p = argparse.ArgumentParser()
    p.add_argument("--source", type=Path, required=True)
    p.add_argument("--results", type=Path, required=True)
    p.add_argument("--out", type=Path, required=True)
    check(p.parse_args())

if __name__ == "__main__":
    main()
