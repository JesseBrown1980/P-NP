#!/usr/bin/env python3
"""Independent verifier for hosted Bitcoin matrix/300s RIME-Fischer trials.

Checks all six pinned input sources, full source-CNF SAT assignments when
present, actual Bitcoin SHA256d, 300s timeouts, and native matrix receipts.
Does not import the contestant ordering code or watched-DPLL implementation.
"""
import argparse
import hashlib
from pathlib import Path
import subprocess

ARMS = ("ym_fischer", "rime_new", "rime_original")
CASES = {
    "nonce32": ("6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3", 239112, 877879),
    "merkle_nonce288": ("601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b", 361188, 1325357),
}
PREFIX = bytes.fromhex("01000000" + "00"*32 +
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a" +
    "29ab5f49" + "ffff001d")
NONCE = 2083236893
TARGET = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
NODE_CAP = 1_000_000_000_000

def require(ok, reason):
    if not ok:
        raise ValueError(reason)

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

def sha256d(header):
    require(len(header) == 80, "Bitcoin header length differs")
    return hashlib.sha256(hashlib.sha256(header).digest()).digest()[::-1].hex()

def receipt(path, label):
    content = path.read_text("ascii")
    rows = content.splitlines()
    line = next((x for x in rows if x.startswith(label + "|")), None)
    require(line is not None, "missing receipt row " + label)
    fields = {}
    for item in line.split("|")[1:]:
        key, delimiter, value = item.partition("=")
        require(bool(delimiter) and key not in fields, "invalid receipt key")
        fields[key] = value
    return fields

def source_check(case, source):
    expected, variables, clause_total = CASES[case]
    require(sha(source.read_bytes()) == expected, "original Bitcoin CNF SHA differs")
    with source.open("rt", encoding="ascii") as stream:
        for row in stream:
            if row.startswith("p cnf "):
                values = row.split()
                require(len(values) == 4 and
                        int(values[2]) == variables and
                        int(values[3]) == clause_total, "source DIMACS shape differs")
                return
    raise ValueError("source header missing")

def verify_full_cnf(source, model_path, n, m):
    raw = model_path.read_text("ascii").split()
    require(len(raw) == n + 1 and raw[-1] == "0", "incomplete model")
    assignment = bytearray(n + 1)
    for index, word in enumerate(raw[:-1], start=1):
        lit = int(word)
        require(abs(lit) == index, "model must assign every variable once in sorted order")
        assignment[index] = 1 if lit > 0 else 2
    complete = 0
    current_satisfied = False
    current_nonempty = False
    with source.open("rt", encoding="ascii") as stream:
        for row in stream:
            if not row.strip() or row[0] in ("c", "p"):
                continue
            for term in row.split():
                lit = int(term)
                if lit == 0:
                    require(current_nonempty and current_satisfied,
                            "original CNF clause fails claimed SAT model")
                    complete += 1
                    current_satisfied = False
                    current_nonempty = False
                else:
                    require(0 < abs(lit) <= n, "source clause variable out of range")
                    current_nonempty = True
                    current_satisfied = current_satisfied or (
                        assignment[abs(lit)] == (1 if lit > 0 else 2)
                    )
    require(complete == m and not current_nonempty, "wrong original-CNF clause count")
    return assignment

def original_header_model_check(assignment, case, source):
    # Reconstruct original source's manifest without importing its solver.
    import ast
    manifest = ast.literal_eval(source.with_suffix(".txt").read_text("ascii"))
    require(manifest["name"] == case and
            manifest["input_sha256"] == CASES[case][0], "original manifest mismatch")
    nonce_mask = (1 << manifest["nonce_free_bits"]) - 1
    low_nonce = sum(1 << bit for bit, var in manifest["nonce_variables"]
                    if assignment[var] == 1)
    nonce = (NONCE & ~nonce_mask) | low_nonce
    header = bytearray(PREFIX + nonce.to_bytes(4, "little"))
    if manifest["merkle_free_bits"]:
        value = sum(1 << bit for bit, var in manifest["merkle_variables"]
                    if assignment[var] == 1)
        header[36:68] = value.to_bytes(32, "little")
    require(sha256d(header) == TARGET, "independent Bitcoin SHA256d target does not match")
    return sha(header)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--source", type=Path, required=True)
    ap.add_argument("--results", type=Path, required=True)
    ap.add_argument("--matrix", type=Path, required=True)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()
    require(sha256d(PREFIX + NONCE.to_bytes(4, "little")) == TARGET,
            "original genesis header differs")
    for offset in (1, 2, 4, 8):
        alt = bytearray(PREFIX + NONCE.to_bytes(4, "little"))
        alt[36] ^= offset
        require(sha256d(alt) != TARGET, "wrong Merkle header passes")
        alt = bytearray(PREFIX + NONCE.to_bytes(4, "little"))
        alt[76] ^= offset
        require(sha256d(alt) != TARGET, "wrong nonce passes")
    sat = hold = 0
    rows = []
    for case, (source_sha, n, m) in CASES.items():
        source = args.source / (case + ".cnf")
        source_check(case, source)
        for arm in ARMS:
            folder = args.results / arm / case
            r = receipt(folder / "receipt.hbp", "BTC300_MATRIX_SEARCH")
            require(r["arm"] == arm and r["case"] == case and
                    r["original_cnf_sha"] == source_sha and
                    int(r["variables"]) == n and int(r["clauses"]) == m,
                    "source/worker mismatched")
            require(int(r["node_cap"]) == NODE_CAP and
                    int(r["search_budget_seconds"]) == 300,
                    "old 1024-node or short wall-time bound still active")
            matrix_path = folder / "matrix.hbp"
            require(r["matrix_proof_sha"] == sha(matrix_path.read_bytes()),
                    "source matrix proof root differs")
            matrix_r = receipt(matrix_path, "BTC_MATRIX_FUNNEL")
            require(matrix_r["case"] == case and
                    matrix_r["cnf_sha256"] == source_sha and
                    matrix_r["fischer_inverse"] == "EXACT" and
                    matrix_r["brown_freeze_play"] == "EXACT" and
                    matrix_r["solver_search"] == "NOT_RUN",
                    "native original-source funnel not authenticated")
            restored = subprocess.run(
                [str(args.matrix), "verify", case, str(source), str(matrix_path)],
                capture_output=True, check=False
            )
            require(restored.returncode == 0, "independent Rust receipt reconstruction failed")
            elapsed = int(r["search_ns"])
            require(elapsed > 0 and int(r["matrix_ns"]) > 0 and
                    int(r["overall_ns"]) >= elapsed, "timing audit incomplete")
            status = r["status"]
            if status == "SAT":
                require(r["search_exit"] == "0", "SAT process exit not 0")
                witness_file = folder / "model.dimacs"
                require(witness_file.is_file() and
                        r["model_sha"] == sha(witness_file.read_bytes()), "model hash differs")
                assignment = verify_full_cnf(source, witness_file, n, m)
                original_header_model_check(assignment, case, source)
                worker = receipt(folder / "worker.hbp", "BTC300_SEARCH_WORKER")
                require(worker["status"] == "SAT" and
                        worker["model_sha"] == r["model_sha"] and
                        int(worker["node_cap"]) == NODE_CAP, "worker proof differs")
                sat += 1
            else:
                require(status == "HOLD_TIME" and r["search_exit"] == "124",
                        "invalid 300s timeout exit")
                require(elapsed >= 295_000_000_000, "timeout occurred before 300s")
                require(r["model_sha"] == "NONE" and
                        not (folder / "model.dimacs").exists() and
                        not (folder / "worker.hbp").exists(), "false SAT receipt on timeout")
                require(r["search_nodes"] == "UNOBSERVED_ON_TIMEOUT",
                        "unobserved progress misrepresented")
                hold += 1
            rows.append(
                f"BTC300_MATRIX_CASE|case={case}|arm={arm}|status={status}|"
                f"original_source_sha={source_sha}|matrix=REBUILT|"
                f"search_budget=300|node_cap={NODE_CAP}|"
                f"search_ns={elapsed}|json=0"
            )
    require(sat + hold == 6, "missing combined search evidence")
    body = "\n".join(rows) + "\n"
    summary = (
        f"BTC300_MATRIX_FINAL|schema=1|cases=2|arms=3|observations=6|"
        f"full_sha256d=REFERENCE_CHECKED|matrix_replay=PASS|"
        f"node_cap={NODE_CAP}|search_budget_seconds=300|"
        f"sat={sat}|hold_time={hold}|wrong_header_controls=8|"
        f"rows_sha256={sha(body.encode())}|status=PASS|json=0\n"
    )
    args.out.write_text(summary + body, encoding="ascii")
    print(summary.rstrip())
    print(body, end="")

if __name__ == "__main__":
    main()
