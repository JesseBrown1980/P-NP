#!/usr/bin/env python3
"""Execute all 61 anonymous fourth-engine inputs and 42 original-arm replays."""
import ast
import hashlib
import sys
from pathlib import Path
from subprocess import run
from time import monotonic_ns

import engine as fourth
import fixtures

ROOT = Path(__file__).resolve().parents[2]
ARMS = ("ym_fischer", "rime_new", "rime_original")


def main(folder):
    output = Path(folder)
    sources = output / "fixtures"
    receipts = output / "receipts"
    baseline = output / "baseline"
    receipts.mkdir(parents=True, exist_ok=True)
    baseline.mkdir(parents=True, exist_ok=True)
    fixtures.main(str(sources))
    records = []
    original = []
    for line in (sources / "FIXTURES.txt").read_text(
            encoding="ascii").splitlines():
        name, n, declared, expect, sha = ast.literal_eval(line)
        path = sources / (name + ".cnf")
        raw = path.read_bytes()
        assert hashlib.sha256(raw).hexdigest() == sha
        nn, clauses = fourth.parse(raw.decode("ascii"))
        assert nn == n and len(clauses) == declared
        r = fourth.run(n, clauses, cap=2000)
        (receipts / (name + ".txt")).write_text(
            repr(r) + "\n", encoding="ascii")
        records.append((name, r["family"], r["status"],
                        r["search_nodes"], r["total_ns"]))
        print("FOURTH_CONTEST|name=" + name + "|method=" + r["family"] +
              "|status=" + r["status"] +
              "|nodes=" + str(r["search_nodes"]) +
              "|elapsed_ns=" + str(r["total_ns"]), flush=True)
        if name.endswith("__original"):
            original.append((name.removesuffix("__original"), path, expect,
                             r["status"]))
    assert len(original) == 14 and len(records) == 61
    for arm in ARMS:
        for case, path, expected, new_status in original:
            started = monotonic_ns()
            child = run([
                sys.executable, "-s", "-B",
                str(ROOT / "experiments" / "array" / "native.py"),
                arm, case, str(path)
            ], capture_output=True, text=True, check=False, timeout=40)
            launch_inclusive_ns = monotonic_ns() - started
            if child.returncode:
                raise RuntimeError(
                    "baseline " + arm + " " + case + " exit " +
                    str(child.returncode) + ": " + child.stderr[-1500:])
            lines = [x for x in child.stdout.splitlines()
                     if x.startswith("SAT_ARRAY_NATIVE|")]
            assert len(lines) == 1 and lines[0].endswith("|json=0")
            row = ast.literal_eval(lines[0].split("|", 2)[1])
            assert row["arm"] == arm and row["case"] == case
            assert row["expected"] == expected
            assert row["input_sha256"] == hashlib.sha256(
                path.read_bytes()).hexdigest()
            row["process_inclusive_ns"] = launch_inclusive_ns
            (baseline / (arm + "__" + case + ".txt")).write_text(
                repr(row) + "\n", encoding="ascii")
            print("FOURTH_BASELINE|arm=" + arm + "|case=" + case +
                  "|status=" + row["status"] +
                  "|nodes=" + str(row["search_nodes"]) +
                  "|elapsed_ns=" + str(row["total_ns"]) +
                  "|process_inclusive_ns=" + str(launch_inclusive_ns),
                  flush=True)
    assert len(list(baseline.glob("*.txt"))) == 42
    print("FOURTH_CONTEST_FINAL|new_engine_fixtures=61|"
          "replayed_originals_per_arm=14|old_arms=3|"
          "baseline_replays=42|total_executions=103|status=PASS")


if __name__ == "__main__":
    assert len(sys.argv) == 2
    main(sys.argv[1])
