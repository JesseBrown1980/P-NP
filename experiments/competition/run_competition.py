#!/usr/bin/env python3
"""Source-frozen 21-case, two-process fair-input SAT competition driver.

Same source per pair; no train/test tuning. Record all failed and HOLD cases.
This process logs executable launch+solve time and independent peak RSS.
"""
import ast
import hashlib
import sys
from pathlib import Path
from subprocess import run
from time import monotonic_ns

sys.path.insert(0, str(Path(__file__).resolve().parents[1].joinpath("sat")))
import engine


def run_arm(method, kind, n, seed, folder):
    path = folder.joinpath(method + ".receipt")
    memory = folder.joinpath(method + ".rss")
    started = monotonic_ns()
    result = run(
        ["/usr/bin/time", "-f", "%M", "-o", str(memory),
         sys.executable, "-s", "-B", "sat/competition.py",
         method, kind, str(n), seed],
        capture_output=True, text=True, check=False)
    elapsed = monotonic_ns() - started
    path.write_text(result.stdout + result.stderr, encoding="ascii")
    assert result.returncode == 0, (method, kind, n, result.stderr[-1200:])
    lines = [line for line in result.stdout.splitlines()
             if line.startswith("PNP_MATCH|")]
    assert len(lines) == 1 and lines[0].endswith("|json=0")
    payload = lines[0].split("|", 2)[1]
    record = ast.literal_eval(payload)
    assert isinstance(record, dict) and record["seed"] == seed
    assert record["kind"] == kind and record["variables"] == n
    assert record["method"] == (
        "YANG_MILLS_FROZEN" if method == "legacy" else "RIME_PH_BSGS_CRT")
    peak = memory.read_text(encoding="ascii").splitlines()[-1]
    assert peak.isdigit()
    record["process_launch_and_solve_ns"] = elapsed
    record["isolated_peak_rss_kib"] = int(peak)
    return record


def main(target):
    root = Path(target)
    root.mkdir(parents=True, exist_ok=True)
    statuses = {"legacy_exact": 0, "rime_exact": 0,
                "legacy_hold": 0, "rime_hold": 0, "agreements": 0}
    for index, (kind, n, seed) in enumerate(engine.heldout_cases()):
        name = kind + "-" + str(n)
        case_root = root.joinpath("cases", name)
        case_root.mkdir(parents=True, exist_ok=True)
        raw = engine.dimacs(n, engine.heldout_source(kind, n, seed))
        case_root.joinpath("source.cnf").write_text(raw, encoding="ascii")
        order = ("legacy", "rime") if index % 2 == 0 else ("rime", "legacy")
        rows = {}
        for method in order:
            rows[method] = run_arm(method, kind, n, seed, case_root)
        assert rows["legacy"]["source_sha256"] == rows["rime"]["source_sha256"]
        assert rows["legacy"]["proof_sha256"] == rows["rime"]["proof_sha256"]
        assert rows["legacy"]["rank"] == rows["rime"]["rank"]
        for arm in ("legacy", "rime"):
            status = rows[arm]["status"]
            assert status in ("EXACT", "EXACT_UNSAT", "HOLD_BUDGET")
            statuses[arm + ("_hold" if status == "HOLD_BUDGET" else "_exact")] += 1
        if rows["legacy"]["status"] != "HOLD_BUDGET" and (
                rows["rime"]["status"] != "HOLD_BUDGET"):
            assert rows["legacy"]["model_count"] == rows["rime"]["model_count"]
            statuses["agreements"] += 1
        paired = dict(case=name, variables=n, kind=kind, seed=seed,
                      original_dimacs_sha256=hashlib.sha256(
                          raw.encode("ascii")).hexdigest(),
                      alternated_execution_order=order, **rows)
        case_root.joinpath("PAIR.txt").write_text(repr(paired) + "\n",
                                                  encoding="ascii")
        print("PNP_ARENA_CASE|" + name +
              "|legacy_status=" + rows["legacy"]["status"] +
              "|rime_status=" + rows["rime"]["status"] +
              "|legacy_nodes=" + str(rows["legacy"]["search_nodes"]) +
              "|rime_nodes=" + str(rows["rime"]["search_nodes"]) +
              "|legacy_total_ns=" +
              str(rows["legacy"]["process_launch_and_solve_ns"]) +
              "|rime_total_ns=" +
              str(rows["rime"]["process_launch_and_solve_ns"]) +
              "|source_frozen=PASS|status=PASS", flush=True)
    assert sum(statuses[k] for k in ("legacy_exact", "legacy_hold")) == 21
    assert sum(statuses[k] for k in ("rime_exact", "rime_hold")) == 21
    print("PNP_ARENA_FINAL|cases=21|paired_source_sha=PASS|"
          "paired_proof_sha=PASS|equal_node_budget=20000|"
          "alternated_order=PASS|independent_checker=REQUIRED|" +
          "|".join(key + "=" + str(value)
                   for key, value in statuses.items()) +
          "|status=RESEARCH_PILOT", flush=True)


if __name__ == "__main__":
    assert len(sys.argv) == 2
    main(sys.argv[1])
