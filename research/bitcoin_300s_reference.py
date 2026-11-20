#!/usr/bin/env python3
"""300s Bitcoin SHA256d reference on the original immutable full-CNF inputs.

Research-only external SAT solver harness, separate from Rust-native kernel.
Reuses the original model checker and preserves historical 12s/20s receipts.
"""
import argparse
import ast
import hashlib
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "experiments" / "bitcoin"
ARMS = ("minisat", "cadical", "picosat")
CASES = {
    "nonce32": ("6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3", 32),
    "merkle_nonce288": ("601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b", 288),
}
PIN = {"runner.py": "8230c55e8b43c416e2f84410c7526d3848646cf7",
       "genesis.py": "aa13b39569697f22f1a1604e5cb5bda2274b7ad8"}
PREFIX = bytes.fromhex("01000000" + "00"*32 +
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a" +
    "29ab5f49" + "ffff001d")
NONCE = 2083236893
TARGET = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"

def require(ok, why):
    if not ok:
        raise ValueError(why)

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

def double_hash(header):
    require(len(header) == 80, "header is not 80 bytes")
    return hashlib.sha256(hashlib.sha256(header).digest()).digest()[::-1].hex()

def source_check(path, case):
    expected, bits = CASES[case]
    require(sha((path / (case + ".cnf")).read_bytes()) == expected, "original CNF changed")
    manifest = ast.literal_eval((path / (case + ".txt")).read_text("ascii"))
    require(manifest["name"] == case and manifest["input_sha256"] == expected
            and manifest["free_bits_total"] == bits, "original source manifest changed")
    return manifest

def run_case(args):
    for name, blob in PIN.items():
        actual = subprocess.check_output(
            ["git", "-C", str(ROOT), "hash-object", str((SRC / name).relative_to(ROOT))],
            text=True).strip()
        require(actual == blob, "original generator or runner changed: " + name)
    source_check(args.source, args.case)
    sys.path.insert(0, str(SRC))
    import runner
    require(runner.LIMIT_EXTERNAL == 12 and runner.LIMIT_NATIVE == 20,
            "historical time limits changed")
    runner.LIMIT_EXTERNAL = 300
    runner.main(args.arm, args.case, str(args.source), str(args.output))
    print(f"BTC300_BUDGET|arm={args.arm}|case={args.case}|seconds=300|"
          "source=ORIGINAL_CNF|original_runner=BYTE_PINNED|status=PASS|json=0", flush=True)

def verify(args):
    require(double_hash(PREFIX + NONCE.to_bytes(4, "little")) == TARGET,
            "genesis reference identity wrong")
    controls = 0
    for mask in (1, 2, 4, 8):
        header = bytearray(PREFIX + NONCE.to_bytes(4, "little"))
        header[36] ^= mask
        require(double_hash(header) != TARGET, "altered merkle accepted")
        controls += 1
        header = bytearray(PREFIX + NONCE.to_bytes(4, "little"))
        header[76] ^= mask
        require(double_hash(header) != TARGET, "altered nonce accepted")
        controls += 1
    lines = []
    sat = hold = 0
    for case in CASES:
        source = source_check(args.source, case)
        for arm in ARMS:
            folder = args.results / arm / case
            r = ast.literal_eval((folder / "receipt.txt").read_text("ascii"))
            require(r["arm"] == arm and r["case"] == case and
                    r["cnf_sha256"] == CASES[case][0], "worker/source identity changed")
            require(r["variables"] == source["variables"] and
                    r["clauses"] == source["clauses"] and
                    r["source_bits"] == source["nonce_free_bits"],
                    "reported source dimensions differ")
            require(r["limit_seconds"] == 300 and
                    isinstance(r["elapsed_ns"], int) and r["elapsed_ns"] > 0,
                    "300s budget or timing record missing")
            require(all((folder / name).is_file() for name in
                        ("solver.stdout", "solver.stderr", "rss.txt")),
                    "missing raw solver evidence")
            status = r["status"]
            require(status in ("SAT", "HOLD_TIME"), "unexpected verdict")
            if status == "SAT":
                require(r["solver_exit"] == 10, "noncanonical SAT exit")
                nonce = r["recovered_nonce"]
                require(isinstance(nonce, int) and 0 <= nonce < (1 << 32) and
                        ((nonce ^ NONCE) >> source["nonce_free_bits"]) == 0,
                        "original nonce constraints differ")
                merkle = bytes.fromhex(r["recovered_merkle_raw_hex"])
                require(len(merkle) == 32, "incorrect 256-bit merkle field")
                hdr = bytearray(PREFIX + nonce.to_bytes(4, "little"))
                hdr[36:68] = merkle
                require(double_hash(hdr) == TARGET and sha(hdr) == r["witness_sha256"],
                        "false Bitcoin SHA256d witness")
                sat += 1
            else:
                require(r["solver_exit"] == 124 and
                        r["recovered_nonce"] == "NONE" and
                        r["witness_sha256"] == "NONE", "incorrect HOLD receipt")
                hold += 1
            lines.append(f"BTC300_CASE|arm={arm}|case={case}|status={status}|"
                         f"source_sha={CASES[case][0]}|limit_seconds=300|"
                         f"elapsed_ns={r['elapsed_ns']}|memory_kib={r['memory_kib']}|json=0")
    require(sat + hold == 6, "incomplete solver/case matrix")
    body = "\n".join(lines) + "\n"
    result = (f"BTC300_FINAL|schema=1|cases=2|arms=3|observations=6|"
              f"budget_seconds_each=300|sat={sat}|hold={hold}|"
              f"original_header=PASS|wrong_header_controls={controls}|"
              f"independent_header_witness=PASS|node_limited_arms_rerun=NO|"
              f"rows_sha256={sha(body.encode())}|status=PASS|json=0\n")
    args.out.write_text(result + body, encoding="ascii")
    print(result.strip())

def main():
    ap = argparse.ArgumentParser()
    cmd = ap.add_subparsers(dest="action", required=True)
    run = cmd.add_parser("run")
    run.add_argument("--arm", choices=ARMS, required=True)
    run.add_argument("--case", choices=tuple(CASES), required=True)
    run.add_argument("--source", type=Path, required=True)
    run.add_argument("--output", type=Path, required=True)
    check = cmd.add_parser("verify")
    check.add_argument("--source", type=Path, required=True)
    check.add_argument("--results", type=Path, required=True)
    check.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()
    if args.action == "run":
        run_case(args)
    else:
        verify(args)

if __name__ == "__main__":
    main()
