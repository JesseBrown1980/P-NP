#!/usr/bin/env python3
"""Proof-referee accepted fourth engine versus replayed three incumbents.

The fourth engine proof check must have already passed. This separately
checks the 42 source-frozen baseline receipts and summarizes outcomes
without importing any old or new engine.
"""
import ast
import hashlib
import sys
from pathlib import Path


CASES = (
    "php3_2", "php4_3", "php5_4", "php6_5",
    "php8_7", "php12_11", "php21_20",
    "php3_3", "php4_4", "php6_6",
    "tseitin16", "tseitin32", "tseitin64", "tseitin128",
)
ARMS = ("ym_fischer", "rime_new", "rime_original")


def read(path):
    return ast.literal_eval(Path(path).read_text(encoding="ascii"))


def main(folder):
    root = Path(folder)
    fourth = root / "receipts"
    baseline = root / "baseline"
    expected = {}
    for case in CASES:
        path = root / "fixtures" / (case + "__original.cnf")
        original = path.read_bytes()
        exp = "SAT" if case.startswith("php") and (
            lambda p, h: p == h)(*map(int, case[3:].split("_"))) else "UNSAT"
        expected[case] = exp
        sha = hashlib.sha256(original).hexdigest()
        new = read(fourth / (case + "__original.txt"))
        assert new["search_nodes"] == 0
        assert (new["status"] == "SAT" if exp == "SAT" else
                new["status"] in ("UNSAT_HALL", "UNSAT_XOR"))
        for arm in ARMS:
            older = read(baseline / (arm + "__" + case + ".txt"))
            assert older["case"] == case and older["arm"] == arm
            assert older["input_sha256"] == sha
            assert older["variables"] == new["variables"]
            assert older["status"] in (
                "SAT", "UNSAT_DPLL", "UNSAT_XOR", "HOLD_BUDGET")
            if older["status"] == "SAT":
                assert exp == "SAT"
            elif older["status"] in ("UNSAT_DPLL", "UNSAT_XOR"):
                assert exp == "UNSAT"
            else:
                assert older["search_nodes"] == 2000
            print("FOURTH_COMPARISON|case=" + case + "|arm=" + arm +
                  "|older=" + older["status"] +
                  "|old_nodes=" + str(older["search_nodes"]) +
                  "|new=" + new["status"] +
                  "|new_nodes=" + str(new["search_nodes"]) +
                  "|old_ns=" + str(older["total_ns"]) +
                  "|new_ns=" + str(new["total_ns"]) +
                  "|same_source=PASS", flush=True)
    assert len(expected) == 14
    for arm in ARMS:
        rows = [read(baseline / (arm + "__" + case + ".txt"))
                for case in CASES]
        completed = sum(row["status"] != "HOLD_BUDGET" for row in rows)
        assert completed == 11
        total_nodes = sum(row["search_nodes"] for row in rows)
        new_rows = [read(fourth / (case + "__original.txt"))
                    for case in CASES]
        assert sum(row["search_nodes"] for row in new_rows) == 0
        print("FOURTH_AGGREGATE|arm=" + arm +
              "|old_complete=" + str(completed) +
              "|old_hold=" + str(14 - completed) +
              "|old_search_nodes=" + str(total_nodes) +
              "|new_complete=14|new_hold=0|new_search_nodes=0|"
              "source_identical=PASS", flush=True)
    print("FOURTH_COMPARISON_FINAL|baseline_arms=3|"
          "original_cases=14|same_input_replays=42|"
          "baseline_complete_per_arm=11|"
          "new_complete=14|new_search_nodes=0|"
          "independent_referee=REQUIRED|status=PASS")


if __name__ == "__main__":
    assert len(sys.argv) == 2
    main(sys.argv[1])
