#!/usr/bin/env python3
"""Real-source 300s RIME/Fischer SAT search after native Bitcoin matrix transport.

Research-only Python adapter for the historical algorithmic ordering code.
Original CNFs, historical ordering and watched-DPLL logic remain unchanged.
The 1,024-node cap is explicitly raised; GNU timeout limits the entire
source-parse + order-generation + solver child process to 300 seconds.
"""
import argparse
import ast
import hashlib
from pathlib import Path
import subprocess
import sys
from time import monotonic_ns

ROOT = Path(__file__).resolve().parents[1]
OLD = ROOT / "experiments" / "bitcoin"
ARMS = ("ym_fischer", "rime_new", "rime_original")
SOURCES = {
    "nonce32": ("6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3", 32),
    "merkle_nonce288": ("601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b", 288),
}
SRC_BLOBS = {
    "genesis.py": "aa13b39569697f22f1a1604e5cb5bda2274b7ad8",
    "arm.py": "c130a41c31370062c317cc530ed5785cd3959dfb",
    "watched.py": "0c27f4293798fe451c620f4871e6c886e389fe61",
}
NODE_CAP = 1_000_000_000_000
TIME_SECONDS = 300

def require(ok, why):
    if not ok:
        raise ValueError(why)

def digest(data):
    return hashlib.sha256(data).hexdigest()

def original_sources(case, path):
    for name, blob in SRC_BLOBS.items():
        result = subprocess.check_output(
            ["git", "-C", str(ROOT), "hash-object",
             str((OLD / name).relative_to(ROOT))], text=True).strip()
        require(result == blob, "historical solver source drift: " + name)
    expected, width = SOURCES[case]
    raw = path.read_bytes()
    require(digest(raw) == expected, "original Bitcoin CNF source fingerprint changed")
    manifest = ast.literal_eval(path.with_suffix(".txt").read_text("ascii"))
    require(manifest["name"] == case and
            manifest["input_sha256"] == expected and
            manifest["free_bits_total"] == width, "original CNF manifest changed")
    return raw, manifest

def search_worker(args):
    begin = monotonic_ns()
    raw, manifest = original_sources(args.case, args.cnf)
    sys.path.insert(0, str(OLD))
    import arm as original_arm
    import watched as original_watched
    import genesis as original_genesis
    variables, clauses = original_arm.parse(raw)
    require(variables == manifest["variables"] and len(clauses) == manifest["clauses"],
            "solver received incorrect full-CNF dimensions")
    nonce_variables = tuple(manifest["nonce_variables"])
    merkle_variables = tuple(manifest["merkle_variables"])
    routing_bits = nonce_variables + tuple((32 + bit, var) for bit, var in merkle_variables)
    order, phi_ops, routing_ns = original_arm.get_order(
        args.arm, clauses, variables, routing_bits
    )
    # Historical source remains byte-identical; 1,024-node limit is lifted
    # for this separate research arm, while wall time is bounded by the parent.
    result = original_watched.solve(variables, clauses, priority=order, limit=NODE_CAP)
    require(result["status"] in ("SAT", "HOLD_BUDGET", "UNSAT_DPLL"),
            "original solver emitted unknown status")
    if result["status"] == "UNSAT_DPLL":
        raise ValueError("UNSAT contradicts independently verified Bitcoin genesis witness")
    if result["status"] == "HOLD_BUDGET":
        raise ValueError("new search node cap reached before 300-second limit")
    positive = set(result["witness"])
    require(result["status"] == "SAT" and result["witness"] is not None, "missing SAT witness")
    require(all(any((abs(lit) in positive) == (lit > 0) for lit in row)
                for row in clauses), "SAT witness fails original CNF")
    nonce = sum(1 << bit for bit, variable in nonce_variables if variable in positive)
    mask = (1 << manifest["nonce_free_bits"]) - 1
    nonce = (original_genesis.GENESIS_NONCE & ~mask) | nonce
    header = bytearray(original_genesis.HEADER_PREFIX + nonce.to_bytes(4, "little"))
    if merkle_variables:
        value = sum(1 << bit for bit, variable in merkle_variables if variable in positive)
        header[36:68] = value.to_bytes(32, "little")
    calculated = hashlib.sha256(hashlib.sha256(header).digest()).digest()[::-1].hex()
    require(calculated == original_genesis.GENESIS_HASH, "original Bitcoin SHA256d mismatch")
    # Retain the entire signed Boolean assignment for independent clause replay.
    assignments = " ".join(str(var if var in positive else -var)
                           for var in range(1, variables + 1)) + " 0\n"
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / "model.dimacs").write_text(assignments, encoding="ascii")
    report = (
        f"BTC300_SEARCH_WORKER|arm={args.arm}|case={args.case}|status=SAT|"
        f"node_cap={NODE_CAP}|search_nodes={result['nodes']}|"
        f"literal_checks={result['literal_checks']}|"
        f"propagations={result['propagations']}|phi_ops={phi_ops}|"
        f"routing_ns={routing_ns}|elapsed_ns={monotonic_ns()-begin}|"
        f"original_cnf_sha={digest(raw)}|original_header_sha={digest(header)}|"
        f"model_sha={digest(assignments.encode())}|json=0\n"
    )
    (args.output / "worker.hbp").write_text(report, encoding="ascii")
    print(report, end="", flush=True)

def orchestrate(args):
    begin = monotonic_ns()
    raw, manifest = original_sources(args.case, args.cnf)
    args.output.mkdir(parents=True, exist_ok=True)
    matrix = args.output / "matrix.hbp"
    # The reference header transport is deliberately separate from the
    # unknown SAT search variables; it is not a discovered preimage.
    for action in ("attest", "verify"):
        step = subprocess.run(
            [str(args.matrix), action, args.case, str(args.cnf), str(matrix)],
            text=True, capture_output=True, check=False
        )
        (args.output / ("matrix_" + action + ".log")).write_text(
            step.stdout + step.stderr, encoding="utf-8"
        )
        require(step.returncode == 0, "native Bitcoin matrix failed: " + action +
                ": " + step.stderr[-1000:])
    matrix_ns = monotonic_ns() - begin
    matrix_text = matrix.read_text("ascii")
    require("fischer_inverse=EXACT|brown_freeze_play=EXACT|" in matrix_text and
            "solver_search=NOT_RUN|cryptographic_preimage_discovery=NOT_CLAIMED|" in matrix_text,
            "native funnel proof differs")
    command = [
        "/usr/bin/time", "-f", "%M", "-o", str(args.output / "search_rss.txt"),
        "timeout", "--kill-after=3s", str(TIME_SECONDS) + "s",
        sys.executable, "-s", "-B", str(Path(__file__).resolve()), "worker",
        "--arm", args.arm, "--case", args.case,
        "--cnf", str(args.cnf), "--output", str(args.output)
    ]
    start_search = monotonic_ns()
    run = subprocess.run(command, capture_output=True, check=False)
    search_ns = monotonic_ns() - start_search
    (args.output / "search.stdout").write_bytes(run.stdout)
    (args.output / "search.stderr").write_bytes(run.stderr)
    status = "SAT" if run.returncode == 0 else "HOLD_TIME" if run.returncode == 124 else "ERROR"
    require(status != "ERROR", "research search failed rc=" + str(run.returncode) +
            " stderr=" + run.stderr.decode("utf-8", errors="replace")[-1000:])
    if status == "SAT":
        worker = (args.output / "worker.hbp").read_text("ascii")
        require("|status=SAT|" in worker and f"|node_cap={NODE_CAP}|" in worker,
                "worker did not return a supported original-CNF model")
        require((args.output / "model.dimacs").is_file(), "missing SAT source model")
        model_sha = digest((args.output / "model.dimacs").read_bytes())
    else:
        require(not (args.output / "worker.hbp").exists(), "unexpected completed worker on timeout")
        require(not (args.output / "model.dimacs").exists(), "unexpected model on timeout")
        model_sha = "NONE"
    line = (
        f"BTC300_MATRIX_SEARCH|schema=1|arm={args.arm}|case={args.case}|"
        f"original_cnf_sha={digest(raw)}|variables={manifest['variables']}|"
        f"clauses={manifest['clauses']}|matrix_proof_sha={digest(matrix.read_bytes())}|"
        f"model_sha={model_sha}|matrix_ns={matrix_ns}|search_ns={search_ns}|"
        f"overall_ns={monotonic_ns()-begin}|node_cap={NODE_CAP}|"
        f"search_nodes={'WORKER_RECEIPT' if status=='SAT' else 'UNOBSERVED_ON_TIMEOUT'}|"
        f"search_budget_seconds={TIME_SECONDS}|status={status}|json=0\n"
    )
    (args.output / "receipt.hbp").write_text(line, encoding="ascii")
    print(line, end="", flush=True)

def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="action", required=True)
    for name in ("worker", "run"):
        p = sub.add_parser(name)
        p.add_argument("--arm", choices=ARMS, required=True)
        p.add_argument("--case", choices=tuple(SOURCES), required=True)
        p.add_argument("--cnf", type=Path, required=True)
        p.add_argument("--output", type=Path, required=True)
        if name == "run":
            p.add_argument("--matrix", type=Path, required=True)
    args = ap.parse_args()
    if args.action == "worker":
        search_worker(args)
    else:
        orchestrate(args)

if __name__ == "__main__":
    main()
