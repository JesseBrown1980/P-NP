#!/usr/bin/env python3
"""Frozen 14-case SAT benchmark array for six independent arm runs.

Seven UNSAT PHP, three SAT PHP, and four 3-regular odd-charge Tseitin.
14 CNFs x six arms = 84 source-identical solver observations.
"""
import hashlib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "experiments" / "hardest"))
import challenge

CASES = (
    "php3_2", "php4_3", "php5_4", "php6_5",
    "php8_7", "php12_11", "php21_20",
    "php3_3", "php4_4", "php6_6",
    "tseitin16", "tseitin32", "tseitin64", "tseitin128",
)

ARMS = ("ym_fischer", "rime_new", "rime_original",
        "minisat", "cadical", "picosat")


def definition(case):
    assert case in CASES
    if case.startswith("php"):
        p, h = (int(s) for s in case[3:].split("_"))
        n, clauses = challenge.php(p, h)
        return n, clauses, ("SAT" if p == h else "UNSAT")
    vertices = int(case.removeprefix("tseitin"))
    n, clauses, edges, attempts = challenge.tseitin(vertices)
    assert len(edges) == 3 * vertices // 2
    assert len(clauses) == 4 * vertices
    assert attempts <= 10000
    return n, clauses, "UNSAT"


def dimacs(case):
    n, clauses, expectation = definition(case)
    lines = [
        "c P-NP array-v1 source-freeze " + case +
        " expected=" + expectation,
        "p cnf " + str(n) + " " + str(len(clauses)),
    ]
    lines.extend(" ".join(str(v) for v in clause) + " 0"
                 for clause in clauses)
    raw = "\n".join(lines) + "\n"
    assert challenge.parse(raw) == (n, clauses)
    return raw


def build(out):
    folder = Path(out)
    folder.mkdir(parents=True, exist_ok=True)
    rows = []
    for case in CASES:
        raw = dimacs(case)
        n, clauses, expected = definition(case)
        path = folder / (case + ".cnf")
        path.write_bytes(raw.encode("ascii"))
        sha = hashlib.sha256(raw.encode("ascii")).hexdigest()
        rows.append((case, n, len(clauses), expected, sha))
        print("SAT_ARRAY_SOURCE|case=" + case + "|variables=" +
              str(n) + "|clauses=" + str(len(clauses)) +
              "|expected=" + expected + "|sha256=" + sha +
              "|status=PASS", flush=True)
    (folder / "SOURCES.txt").write_text(
        "".join(repr(row) + "\n" for row in rows), encoding="ascii")
    assert len(rows) == 14
    assert len({row[-1] for row in rows}) == 14
    assert sum(row[3] == "UNSAT" for row in rows) == 11
    print("SAT_ARRAY_FINAL|cases=14|unsat=11|sat=3|arms=6|"
          "observations=84|source_generator=FROZEN|status=PASS")


if __name__ == "__main__":
    assert len(sys.argv) == 2
    build(sys.argv[1])
