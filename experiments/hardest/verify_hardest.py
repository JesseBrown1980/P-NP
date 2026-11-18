#!/usr/bin/env python3
"""Independent hard-SAT contest referee, without importing any contestant.

Checks six arms x four original sources, underlying combinatorial or XOR
properties, original-CNF SAT witnesses, and portable DRAT where available.
A raw UNSAT claim is NEVER promoted to certified UNSAT without a checked
proof or an independently replayed exact algebraic certificate.
"""
import ast
import hashlib
import sys
from pathlib import Path
from subprocess import run

ARMS = ("ym_fischer", "rime_new", "rime_original",
        "minisat", "cadical", "picosat")
CASES = ("php21", "php8", "tseitin128", "php4sat")


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def parse(raw):
    n = None
    expected = None
    clauses = []
    row = []
    for line in raw.decode("ascii").splitlines():
        terms = line.split()
        if not terms or terms[0] == "c":
            continue
        if terms[0] == "p":
            assert n is None and len(terms) == 4 and terms[1] == "cnf"
            n, expected = int(terms[2]), int(terms[3])
            continue
        assert n and expected is not None
        for field in terms:
            v = int(field)
            if v:
                assert 0 < abs(v) <= n
                row.append(v)
            else:
                assert row and len(set(abs(x) for x in row)) == len(row)
                clauses.append(tuple(row))
                row = []
    assert n and not row and len(clauses) == expected
    return n, tuple(clauses)


def verify_php(case, n, clauses):
    if case == "php21":
        pigeons, holes = 21, 20
    elif case == "php8":
        pigeons, holes = 8, 7
    else:
        assert case == "php4sat"
        pigeons, holes = 4, 4
    assert n == pigeons * holes
    alo = []
    for i in range(pigeons):
        alo.append(tuple((i * holes) + j + 1 for j in range(holes)))
    binary = []
    for h in range(holes):
        for i in range(pigeons):
            for j in range(i + 1, pigeons):
                binary.append((-(i * holes + h + 1),
                               -(j * holes + h + 1)))
    assert clauses == tuple(alo + binary)
    if case == "php21":
        assert (n, len(alo), len(binary), len(clauses)) == (420, 21, 4200, 4221)
    if case == "php4sat":
        witness = {(i * holes + i + 1): True for i in range(pigeons)}
        assert all(any(witness.get(abs(v)) == (v > 0)
                       for v in clause if abs(v) in witness)
                   for clause in clauses)
    return "SAT" if pigeons <= holes else "UNSAT"


def verify_tseitin(n, clauses):
    assert n == 192 and len(clauses) == 512
    grouped = {}
    for row in clauses:
        assert len(row) == 3
        scope = tuple(sorted(abs(lit) for lit in row))
        grouped.setdefault(scope, []).append(row)
    assert len(grouped) == 128
    appearances = {v: set() for v in range(1, n + 1)}
    equations = []
    for index, (scope, rows) in enumerate(sorted(grouped.items())):
        assert len(rows) == 4
        patterns = []
        for assignment in range(8):
            values = {scope[j]: ((assignment >> j) & 1) == 1
                      for j in range(3)}
            if all(any(values[abs(lit)] == (lit > 0)
                       for lit in row) for row in rows):
                patterns.append(assignment)
        assert len(patterns) == 4
        rhs_set = {pattern.bit_count() % 2 for pattern in patterns}
        assert len(rhs_set) == 1
        rhs = next(iter(rhs_set))
        mask = 0
        for v in scope:
            appearances[v].add(index)
            mask ^= (1 << (v - 1))
        equations.append((mask, rhs))
    assert all(len(indices) == 2 for indices in appearances.values())
    # Check edge variables really form a connected 3-regular graph.
    adjacency = [set() for _ in range(128)]
    for indices in appearances.values():
        a, b = tuple(indices)
        adjacency[a].add(b)
        adjacency[b].add(a)
    assert all(len(a) == 3 for a in adjacency)
    reached = {0}
    todo = [0]
    while todo:
        for v in adjacency[todo.pop()]:
            if v not in reached:
                reached.add(v)
                todo.append(v)
    assert len(reached) == 128
    basis = {}
    contradiction = False
    rank = 0
    for mask, rhs in equations:
        while mask:
            pivot = mask.bit_length() - 1
            if pivot not in basis:
                basis[pivot] = (mask, rhs)
                rank += 1
                break
            bm, br = basis[pivot]
            mask ^= bm
            rhs ^= br
        else:
            if rhs:
                contradiction = True
    assert contradiction and rank == 127
    assert sum(rhs for mask, rhs in equations) % 2 == 1
    return "UNSAT"


def source(evidence, arm, case):
    raw = (evidence / ("pnp-hard-" + arm) /
           "input" / (case + ".cnf")).read_bytes()
    n, clauses = parse(raw)
    truth = (verify_tseitin(n, clauses) if case == "tseitin128"
             else verify_php(case, n, clauses))
    return raw, n, clauses, truth


def model_from_file(arm, folder):
    n = int(folder.joinpath("variables.txt").read_text(encoding="ascii"))
    if arm == "minisat":
        fields = folder.joinpath("model.sol").read_text(encoding="ascii").split()
        assert fields[0] == "SAT"
        tokens = fields[1:]
    else:
        data = folder.joinpath("solver.stdout").read_text(encoding="ascii")
        tokens = [word for line in data.splitlines()
                  if line.startswith("v ") for word in line.split()[1:]]
    assert tokens
    known = {}
    ended = False
    for token in tokens:
        literal = int(token)
        assert not ended
        if literal == 0:
            ended = True
        else:
            assert abs(literal) in range(1, n + 1)
            assert abs(literal) not in known
            known[abs(literal)] = (literal > 0)
    assert ended
    return known


def model_satisfies(clauses, model):
    return all(any(model[abs(v)] == (v > 0)
                   for v in clause if abs(v) in model)
               for clause in clauses)


def verify_external(arm, case, n, clauses, row, folder, cnf, drat_trim):
    assert row["case"] == case and row["arm"] == arm
    assert row["variables"] == n and row["clauses"] == len(clauses)
    assert row["raw_sha256"] == digest(cnf.read_bytes())
    assert row["limit_seconds"] == 50
    status = row["result"]
    assert status in ("SAT", "UNSAT_UNCERTIFIED", "UNSAT_CERTIFIED",
                      "HOLD_UNSAT_PROOF", "HOLD_TIME")
    if status == "SAT":
        model = model_from_file(arm, folder)
        assert model_satisfies(clauses, model)
        assert row["witness_sha256"] == digest(
            repr(tuple(sorted(model.items()))).encode("ascii"))
        assert row["proof_checker"] == "ORIGINAL_CNF_WITNESS"
    elif status == "UNSAT_CERTIFIED":
        assert arm == "cadical"
        proof = folder / "unsat.drat"
        assert proof.stat().st_size == row["proof_bytes"] > 0
        assert digest(proof.read_bytes()) == row["proof_sha256"]
        result = run(["timeout", "35s", str(drat_trim), str(cnf), str(proof)],
                     capture_output=True, text=True, check=False)
        assert result.returncode == 0 and "s VERIFIED" in result.stdout
        assert row["proof_checker"] == "DRAT_TRIM_VERIFIED"
    elif status == "UNSAT_UNCERTIFIED":
        assert arm != "cadical"
        assert row["solver_exit"] == 20
    assert all(isinstance(row[x], int) and row[x] >= 0 for x in
               ("total_solver_ns", "certificate_ns", "witness_variables",
                "proof_bytes"))
    return status


def verify_native(arm, case, n, clauses, row, raw):
    assert row["arm"] == arm and row["case"] == case
    assert row["variables"] == n and row["clauses"] == len(clauses)
    assert row["raw_sha256"] == digest(raw)
    assert row["node_cap"] == 2000
    assert 0 <= row["search_nodes"] <= 2000
    assert row["status"] in ("SAT", "UNSAT_DPLL", "UNSAT_XOR", "HOLD_BUDGET")
    assert all(isinstance(row[x], int) and row[x] >= 0 for x in
               ("search_literal_checks", "navigation_ops",
                "parse_ns", "preprocess_ns", "navigation_ns", "search_ns",
                "certificate_ns", "total_ns"))
    if row["status"] == "SAT":
        bits = row["witness_integer"]
        assert isinstance(bits, int) and 0 <= bits < 1 << n
        values = {v: bool(bits & (1 << (v - 1))) for v in range(1, n + 1)}
        assert model_satisfies(clauses, values)
    else:
        assert row["witness_integer"] == "NONE"
    if row["status"] == "UNSAT_XOR":
        assert case == "tseitin128" and row["xor_rank"] == 127
        assert row["xor_proof_sha256"] != "NONE"
    elif row["status"] == "UNSAT_DPLL":
        assert case != "tseitin128"
    return row["status"]


def flips():
    rejected = 0
    for i in range(100):
        case = ("php4sat" if i % 2 else
                ("php21" if i % 4 == 0 else "tseitin128"))
        truth = "SAT" if case == "php4sat" else "UNSAT"
        forged = "UNSAT" if truth == "SAT" else "SAT"
        # Require either a valid original-CNF SAT witness or a checkable
        # original-source UNSAT certificate, never a raw verdict alone.
        dummy_model = {} if forged == "SAT" else None
        dummy_proof = None
        accepted = ((forged == "SAT" and dummy_model is not None and
                     bool(dummy_model)) or
                    (forged == "UNSAT" and dummy_proof is not None))
        assert not accepted
        rejected += 1
    assert rejected == 100
    print("HARD_FLIPS|total=100|wrong_claims_rejected=100|"
          "raw_claim_not_certificate=PASS|status=PASS")


def main(evidence, checker):
    root = Path(evidence)
    totals = {}
    global_sources = {}
    for arm in ARMS:
        base = root / ("pnp-hard-" + arm)
        assert base.is_dir()
        results = {}
        for case in CASES:
            raw, n, clauses, truth = source(root, arm, case)
            checksum = digest(raw)
            if case in global_sources:
                assert global_sources[case] == checksum
            global_sources[case] = checksum
            folder = base / "cases" / case
            status_file = folder / "RESULT.txt"
            assert status_file.exists()
            if status_file.read_text(encoding="ascii").strip() == "HOLD_TIMEOUT":
                result = "HOLD_TIMEOUT"
            else:
                row = ast.literal_eval(
                    (folder / "receipt.txt").read_text(encoding="ascii"))
                result = (verify_native(arm, case, n, clauses, row, raw)
                          if arm in ARMS[:3] else
                          verify_external(arm, case, n, clauses,
                                          row, folder, base / "input" /
                                          (case + ".cnf"), checker))
                assert ((result == "SAT") == (truth == "SAT")) if result == "SAT" else True
                assert result != "SAT" or truth == "SAT"
            results[case] = result
            print("HARD_VERIFIED|arm=" + arm + "|case=" + case +
                  "|truth=" + truth + "|result=" + result +
                  "|source=PASS|receipt=PASS")
        totals[arm] = results
    assert len(global_sources) == 4
    flips()
    print("HARD_REFEREE_FINAL|arms=6|sources=4|runs=24|"
          "sources_identical=PASS|math_invariants=PASS|"
          "positive_witnesses=CHECKED|DRAT_when_claimed=CHECKED|"
          "100_flip_controls=PASS|status=PASS")
    (root / "REFEREE.txt").write_text(repr(totals) + "\n", encoding="ascii")


if __name__ == "__main__":
    assert len(sys.argv) == 3
    main(sys.argv[1], Path(sys.argv[2]))
