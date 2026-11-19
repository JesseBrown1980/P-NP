#!/usr/bin/env python3
"""300-second original Bitcoin SHA256d research-arm calibration.

The 2026-11-17 historical Fischer/RIME/HXF source is kept byte-identical.
Its old 1024-node stop is intercepted at its call boundary, replaced by an
explicit 1,000,000-node bound, and independently metered against a 300-second
per-worker wall cap. This is a research-side reference, not accepted native
Rust production code or a model of trained MTP inference.
"""
import argparse
import ast
from contextlib import redirect_stdout
import hashlib
from io import StringIO
from pathlib import Path
import re
import subprocess
import sys
from time import monotonic_ns

ROOT = Path(__file__).resolve().parents[1]
BTC = ROOT / "experiments" / "bitcoin"
ARMS = ("hxf4", "ym_fischer", "rime_new", "rime_original")
CASES = {
    "nonce32": ("6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3", 32),
    "merkle_nonce288": ("601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b", 288),
}
SOURCE_PINS = {
    "arm.py": "c130a41c31370062c317cc530ed5785cd3959dfb",
    "watched.py": "0c27f4293798fe451c620f4871e6c886e389fe61",
    "genesis.py": "aa13b39569697f22f1a1604e5cb5bda2274b7ad8",
}
NODE_CAP = 1_000_000
WALL_SECONDS = 300
PREFIX = bytes.fromhex(
    "01000000" + "00"*32 +
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a" +
    "29ab5f49" + "ffff001d")
NONCE = 2083236893
HASH = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"

def check(ok, reason):
    if not ok:
        raise ValueError(reason)

def sha(data):
    return hashlib.sha256(data).hexdigest()

def dh(header):
    check(len(header) == 80, "original Bitcoin header is not 80 bytes")
    return hashlib.sha256(hashlib.sha256(header).digest()).digest()[::-1].hex()

def source(case, directory):
    expected, bits = CASES[case]
    cnf = directory / (case + ".cnf")
    check(sha(cnf.read_bytes()) == expected, "original Bitcoin CNF SHA changed")
    manifest = ast.literal_eval((directory / (case + ".txt")).read_text("ascii"))
    check(manifest["name"] == case and
          manifest["input_sha256"] == expected and
          manifest["free_bits_total"] == bits, "original Bitcoin case manifest changed")
    return manifest

def engine(args):
    check(args.arm in ARMS and args.case in CASES, "incorrect research arm")
    source(args.case, args.source)
    for name, expected in SOURCE_PINS.items():
        actual = subprocess.check_output(
            ["git", "-C", str(ROOT), "hash-object", str((BTC / name).relative_to(ROOT))],
            text=True).strip()
        check(actual == expected, "historical research source file was modified: " + name)
    sys.path.insert(0, str(BTC))
    import arm as original
    original_watched = original.watched.solve
    def extended_watched(n, clauses, priority=(), limit=1024):
        check(limit == 1024, "historical watched caller changed")
        return original_watched(n, clauses, priority=priority, limit=NODE_CAP)
    original.watched.solve = extended_watched
    if args.arm == "hxf4":
        sys.path.insert(0, str(ROOT / "experiments" / "fourth"))
        import engine as fourth
        original_fourth = fourth.run
        def extended_fourth(n, clauses, cap=1024):
            check(cap == 1024, "historical HXF-1 caller changed")
            return original_fourth(n, clauses, cap=NODE_CAP)
        fourth.run = extended_fourth
    stream = StringIO()
    with redirect_stdout(stream):
        original.main(args.arm, args.case,
                      str(args.source / (args.case + ".cnf")),
                      str(args.source / (args.case + ".txt")))
    records = [line for line in stream.getvalue().splitlines()
               if line.startswith("BITCOIN_ARM|")]
    check(len(records) == 1 and records[0].endswith("|json=0"),
          "historical research arm emitted no accepted result")
    historical = ast.literal_eval(records[0].split("|", 2)[1])
    check(historical["arm"] == args.arm and historical["case"] == args.case,
          "source-bound research arm result mismatch")
    check(historical["status"] in ("SAT", "HOLD_BUDGET"),
          "unexpected claimed outcome")
    check(historical["node_cap"] == 1024,
          "original 1024 argument not observable")
    check(0 <= historical["search_nodes"] <= NODE_CAP,
          "extended node bound violated")
    historical["node_cap"] = NODE_CAP
    historical["original_caller_node_arg"] = 1024
    historical["extended_search_authorized"] = True
    print("BTC300_RESEARCH_ENGINE|" + repr(historical) + "|json=0", flush=True)

def driver(args):
    manifest = source(args.case, args.source)
    out = args.output
    out.mkdir(parents=True, exist_ok=True)
    started = monotonic_ns()
    cmd = ["/usr/bin/time", "-f", "%M", "-o", str(out / "rss.txt"),
           "timeout", "--kill-after=3s", str(WALL_SECONDS) + "s",
           sys.executable, "-s", "-B", str(Path(__file__).resolve()),
           "engine", "--arm", args.arm, "--case", args.case,
           "--source", str(args.source.resolve())]
    completed = subprocess.run(cmd, capture_output=True, check=False)
    elapsed = monotonic_ns() - started
    stdout = completed.stdout.decode("ascii", errors="replace")
    stderr = completed.stderr.decode("ascii", errors="replace")
    (out / "solver.stdout").write_text(stdout, encoding="utf-8")
    (out / "solver.stderr").write_text(stderr, encoding="utf-8")
    status = "HOLD_TIME" if completed.returncode == 124 else None
    inner = None
    recovered = merkle = witness = "NONE"
    if completed.returncode == 0:
        lines = [line for line in stdout.splitlines()
                 if line.startswith("BTC300_RESEARCH_ENGINE|")]
        check(len(lines) == 1 and lines[0].endswith("|json=0"),
              "no extended genuine research-arm receipt")
        inner = ast.literal_eval(lines[0].split("|", 2)[1])
        check(inner["arm"] == args.arm and inner["case"] == args.case and
              inner["extended_search_authorized"] is True and
              inner["node_cap"] == NODE_CAP and
              inner["cnf_sha256"] == CASES[args.case][0],
              "wrong source, node limit, or research arm")
        status = inner["status"]
        check(status in ("SAT", "HOLD_BUDGET"), "uncertified research result")
        if status == "SAT":
            recovered = inner["recovered_nonce"]
            merkle = inner["recovered_merkle_raw_hex"]
            witness = inner["header_witness_sha256"]
    elif completed.returncode != 124:
        raise RuntimeError("extended research arm failed rc=" +
                           str(completed.returncode) + " stderr=" + stderr[-2000:])
    rss_file = out / "rss.txt"
    rss = rss_file.read_text("ascii").splitlines() if rss_file.is_file() else []
    peak = int(rss[-1]) if rss and rss[-1].isdigit() else -1
    receipt = {
        "case": args.case, "arm": args.arm, "status": status,
        "cnf_sha256": CASES[args.case][0],
        "variables": manifest["variables"], "clauses": manifest["clauses"],
        "free_bits": manifest["free_bits_total"],
        "node_limit": NODE_CAP, "wall_seconds": WALL_SECONDS,
        "historical_wall_seconds": 20, "historical_node_limit": 1024,
        "elapsed_ns": elapsed, "memory_kib": peak,
        "solver_exit": completed.returncode, "original_arm_receipt": inner,
        "recovered_nonce": recovered, "recovered_merkle_raw_hex": merkle,
        "witness_sha256": witness,
    }
    (out / "receipt.txt").write_text(repr(receipt) + "\n", encoding="ascii")
    print("BTC300_RESEARCH|" + repr({
        key: value for key, value in receipt.items() if key != "original_arm_receipt"
    }) + "|json=0", flush=True)

def verify(args):
    reference = PREFIX + NONCE.to_bytes(4, "little")
    check(dh(reference) == HASH, "public Bitcoin genesis SHA256d altered")
    negatives = 0
    for offset in (1, 2, 4, 8):
        changed = bytearray(reference)
        changed[36] ^= offset
        check(dh(changed) != HASH, "altered Merkle field accepted")
        negatives += 1
        changed = bytearray(reference)
        changed[76] ^= offset
        check(dh(changed) != HASH, "altered nonce accepted")
        negatives += 1
    sat = node_holds = time_holds = 0
    rows = []
    for case in CASES:
        manifest = source(case, args.source)
        for arm in ARMS:
            folder = args.results / arm / case
            r = ast.literal_eval((folder / "receipt.txt").read_text("ascii"))
            check(r["case"] == case and r["arm"] == arm and
                  r["cnf_sha256"] == CASES[case][0] and
                  r["variables"] == manifest["variables"] and
                  r["clauses"] == manifest["clauses"] and
                  r["free_bits"] == manifest["free_bits_total"],
                  "forged or cross-labeled research receipt")
            check(r["node_limit"] == NODE_CAP and r["wall_seconds"] == WALL_SECONDS and
                  r["historical_node_limit"] == 1024 and
                  r["historical_wall_seconds"] == 20, "budget contract changed")
            check(isinstance(r["elapsed_ns"], int) and r["elapsed_ns"] > 0,
                  "missing elapsed wall measurement")
            check(all((folder / name).is_file() for name in
                      ("solver.stdout", "solver.stderr", "rss.txt")),
                  "missing research worker raw evidence")
            status = r["status"]
            check(status in ("SAT", "HOLD_BUDGET", "HOLD_TIME"),
                  "unsupported research outcome")
            inner = r["original_arm_receipt"]
            if status == "HOLD_TIME":
                check(r["solver_exit"] == 124 and inner is None and
                      r["witness_sha256"] == "NONE", "HOLD_TIME was manufactured")
                time_holds += 1
            else:
                check(r["solver_exit"] == 0 and isinstance(inner, dict),
                      "research internal solver receipt absent")
                check(inner["cnf_sha256"] == CASES[case][0] and
                      inner["node_cap"] == NODE_CAP and
                      inner["status"] == status, "uncoupled extended engine evidence")
                if status == "HOLD_BUDGET":
                    check(inner["search_nodes"] == NODE_CAP and
                          r["witness_sha256"] == "NONE",
                          "node cap HOLD without full node consumption")
                    node_holds += 1
                else:
                    nonce = r["recovered_nonce"]
                    check(isinstance(nonce, int) and 0 <= nonce < 2**32 and
                          ((nonce ^ NONCE) >> manifest["nonce_free_bits"]) == 0,
                          "Bitcoin nonce source bits changed")
                    merkle = bytes.fromhex(r["recovered_merkle_raw_hex"])
                    check(len(merkle) == 32, "Merkle field is not 256 bits")
                    header = bytearray(PREFIX + nonce.to_bytes(4, "little"))
                    header[36:68] = merkle
                    check(dh(header) == HASH and
                          sha(header) == r["witness_sha256"],
                          "SAT claim failed original full Bitcoin SHA256d")
                    sat += 1
            rows.append(f"BTC300_RESEARCH_ROW|arm={arm}|case={case}|status={status}|"
                        f"sha256={CASES[case][0]}|wall_seconds=300|nodes_cap=1000000|"
                        f"elapsed_ns={r['elapsed_ns']}|json=0")
    check(sat + time_holds + node_holds == 8, "eight source-case/engine receipts required")
    body = "\n".join(rows) + "\n"
    root = sha(body.encode())
    final = (f"BTC300_RESEARCH_FINAL|engines=4|cases=2|observations=8|"
             f"budget_seconds=300|node_cap=1000000|sat={sat}|"
             f"hold_time={time_holds}|hold_node={node_holds}|"
             f"original_bitcoin=PASS|negative_headers={negatives}|"
             f"rows_sha256={root}|status=PASS|json=0\n")
    args.out.write_text(final + body, encoding="ascii")
    print(final.rstrip())

def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="action", required=True)
    for name in ("engine", "run"):
        p = sub.add_parser(name)
        p.add_argument("--arm", choices=ARMS, required=True)
        p.add_argument("--case", choices=tuple(CASES), required=True)
        p.add_argument("--source", type=Path, required=True)
        if name == "run":
            p.add_argument("--output", type=Path, required=True)
    v = sub.add_parser("verify")
    v.add_argument("--source", type=Path, required=True)
    v.add_argument("--results", type=Path, required=True)
    v.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()
    if args.action == "engine":
        engine(args)
    elif args.action == "run":
        driver(args)
    else:
        verify(args)

if __name__ == "__main__":
    main()
