#!/usr/bin/env python3
"""Original array + deterministic unseen identifier and clause permutations.

The engine is passed ONLY anonymous n, clauses. Expected SAT status and the
fixture label are kept in the independent test runner, never passed to solver.
"""
import hashlib
import random
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "experiments" / "array"))
import suite


def rewrite(n, clauses, seed):
    """Blindly relabel variables, reverse clause list and shuffle literals."""
    rng = random.Random(seed)
    permutation = list(range(1, n + 1))
    rng.shuffle(permutation)
    changed = []
    for clause in clauses:
        row = [permutation[abs(v) - 1] * (1 if v > 0 else -1)
               for v in clause]
        rng.shuffle(row)
        changed.append(tuple(row))
    rng.shuffle(changed)
    return n, tuple(changed)


def negative_controls():
    # These are not a recognized exact clique or complete parity encoding.
    # They MUST be sent through generic bounded DPLL.
    yield "broken_php_edge", 6, (
        (1, 2), (3, 4), (5, 6),
        (-1, -3), (-1, -5), (-3, -5),
        (-2, -4), (-2, -6))
    yield "overlapping_positive_groups", 3, (
        (1, 2), (2, 3), (-1, -3))
    yield "mixed_clause", 3, (
        (1, 2), (-2, 3), (-1, -3))
    yield "contradictory_units", 1, (
        (1,), (-1,))
    # Opposing parity constraints on three variables, with one clause
    # absent from odd parity, allowing a real satisfying assignment.
    group_even = ((1, 2, 3), (1, -2, -3),
                  (-1, 2, -3), (-1, -2, 3))
    group_odd_missing = ((-1, -2, -3), (-1, 2, 3),
                         (1, -2, 3))
    yield "incomplete_xor_group", 3, group_even + group_odd_missing


def sources():
    for case in suite.CASES:
        n, clauses, expected = suite.definition(case)
        yield case + "__original", n, clauses, expected
        for trial in range(3):
            seed = int.from_bytes(hashlib.sha256((
                case + ":identity-permutation:" + str(trial)).encode("ascii")
                ).digest()[:8], "big")
            nn, cc = rewrite(n, clauses, seed)
            assert nn == n
            yield case + "__permutation" + str(trial), nn, cc, expected
    for name, n, clauses in negative_controls():
        yield "negative__" + name, n, clauses, "BY_CHECKER"


def dimacs(n, clauses):
    rows = ["p cnf " + str(n) + " " + str(len(clauses))]
    rows.extend(" ".join(str(v) for v in row) + " 0"
                for row in clauses)
    return "\n".join(rows) + "\n"


def main(out):
    folder = Path(out)
    folder.mkdir(parents=True, exist_ok=True)
    records = []
    for name, n, clauses, expected in sources():
        raw = dimacs(n, clauses)
        path = folder / (name + ".cnf")
        path.write_text(raw, encoding="ascii")
        sha = hashlib.sha256(raw.encode("ascii")).hexdigest()
        records.append((name, n, len(clauses), expected, sha))
    assert len(records) == 61
    (folder / "FIXTURES.txt").write_text(
        "".join(repr(row) + "\n" for row in records),
        encoding="ascii")
    print("FOURTH_FIXTURES|cases=14|permuted=42|original=14|"
          "malformed_structure_controls=5|total=61|status=PASS")


if __name__ == "__main__":
    assert len(sys.argv) == 2
    main(sys.argv[1])
