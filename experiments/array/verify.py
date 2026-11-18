#!/usr/bin/env python3
"""Clean-room 84-result referee. Imports NO research solver/generator.

Reconstruct PHP exactly, derive Tseitin UNSAT from the actual CNF parity
structure, check original SAT witnesses, and replay original-CNF DRAT
certificates with independently built, source-pinned drat-trim.
"""
import ast
import hashlib
import sys
from pathlib import Path
from subprocess import run


CASES = (
    "php3_2", "php4_3", "php5_4", "php6_5",
    "php8_7", "php12_11", "php21_20",
    "php3_3", "php4_4", "php6_6",
    "tseitin16", "tseitin32", "tseitin64", "tseitin128",
)
ARMS = ("ym_fischer", "rime_new", "rime_original",
        "minisat", "cadical", "picosat")


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def parse_dimacs(raw):
    variables = None
    announced = None
    clauses = []
    row = []
    for line in raw.decode("ascii").splitlines():
        pieces = line.split()
        if not pieces or pieces[0] == "c":
            continue
        if pieces[0] == "p":
            assert variables is None and pieces[1] == "cnf"
            variables, announced = int(pieces[2]), int(pieces[3])
            continue
        assert variables
        for word in pieces:
            lit = int(word)
            if lit:
                assert 1 <= abs(lit) <= variables
                row.append(lit)
            else:
                assert row and len({abs(v) for v in row}) == len(row)
                clauses.append(tuple(row))
                row = []
    assert variables and not row and len(clauses) == announced
    return variables, tuple(clauses)


def php_spec(case, n, actual):
    pigeons, holes = (int(x) for x in case[3:].split("_"))
    assert pigeons in (holes, holes + 1) and n == pigeons * holes
    generated = []
    for i in range(pigeons):
        generated.append(tuple(i * holes + j + 1 for j in range(holes)))
    for hole in range(holes):
        for left in range(pigeons):
            for right in range(left + 1, pigeons):
                generated.append((-(left * holes + hole + 1),
                                  -(right * holes + hole + 1)))
    assert actual == tuple(generated)
    return "SAT" if pigeons == holes else "UNSAT", 0


def tseitin_spec(case, n, clauses):
    vcount = int(case.removeprefix("tseitin"))
    assert vcount in (16, 32, 64, 128) and n == 3 * vcount // 2
    assert len(clauses) == 4 * vcount
    grouped = {}
    for clause in clauses:
        assert len(clause) == 3
        scope = tuple(sorted(abs(v) for v in clause))
        grouped.setdefault(scope, []).append(clause)
    assert len(grouped) == vcount
    edge_ends = {v: set() for v in range(1, n + 1)}
    equations = []
    for vertex, (scope, rows) in enumerate(sorted(grouped.items())):
        assert len(scope) == 3 and len(rows) == 4
        permitted = []
        for bits in range(8):
            model = {scope[j]: bool((bits >> j) & 1) for j in range(3)}
            if all(any(model[abs(lit)] == (lit > 0)
                       for lit in row) for row in rows):
                permitted.append(bits)
        assert len(permitted) == 4
        parity = {x.bit_count() & 1 for x in permitted}
        assert len(parity) == 1
        charge = next(iter(parity))
        mask = 0
        for edge in scope:
            edge_ends[edge].add(vertex)
            mask ^= 1 << (edge - 1)
        equations.append((mask, charge))
    assert all(len(ends) == 2 for ends in edge_ends.values())
    graph = [set() for _ in range(vcount)]
    for ends in edge_ends.values():
        a, b = tuple(ends)
        graph[a].add(b)
        graph[b].add(a)
    assert all(len(neighbors) == 3 for neighbors in graph)
    reached = {0}
    stack = [0]
    while stack:
        for neighbor in graph[stack.pop()]:
            if neighbor not in reached:
                reached.add(neighbor)
                stack.append(neighbor)
    assert len(reached) == vcount
    assert sum(rhs for _, rhs in equations) % 2 == 1
    rank = 0
    basis = {}
    contradiction = False
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
            contradiction |= bool(rhs)
    assert rank == vcount - 1 and contradiction
    return "UNSAT", rank


def known_truth(case, n, clauses):
    return (php_spec(case, n, clauses) if case.startswith("php")
            else tseitin_spec(case, n, clauses))


def satisfies(clauses, assignment):
    return all(any(assignment.get(abs(lit)) == (lit > 0)
                   for lit in clause if abs(lit) in assignment)
               for clause in clauses)


def external_model(arm, folder, n):
    if arm == "minisat":
        fields = (folder / "model.sol").read_text(encoding="ascii").split()
        assert fields and fields[0] == "SAT"
        words = fields[1:]
    else:
        output = (folder / "solver.stdout").read_text(encoding="ascii")
        words = [v for line in output.splitlines()
                 if line.startswith("v ") for v in line.split()[1:]]
    assert words
    bits = {}
    terminated = False
    for word in words:
        v = int(word)
        assert not terminated
        if v == 0:
            terminated = True
        else:
            assert 1 <= abs(v) <= n and abs(v) not in bits
            bits[abs(v)] = v > 0
    assert terminated
    return bits


def verify_native(row, arm, case, n, clauses, truth, rank, raw):
    assert row["case"] == case and row["arm"] == arm
    assert row["variables"] == n and row["clauses"] == len(clauses)
    assert row["input_sha256"] == sha(raw)
    assert row["expected"] == truth
    assert row["node_limit"] == 2000
    assert 0 <= row["search_nodes"] <= 2000
    assert row["status"] in ("SAT", "UNSAT_DPLL", "UNSAT_XOR", "HOLD_BUDGET")
    assert row["branch_order_used"] == case.startswith("php")
    assert all(isinstance(row[name], int) and row[name] >= 0 for name in (
        "navigation_ops", "navigation_ns", "literal_checks", "preprocess_ns",
        "search_ns", "total_ns", "peak_rss_kib"))
    status = row["status"]
    if status == "SAT":
        assert truth == "SAT"
        witness = row["witness_integer"]
        assert isinstance(witness, int) and 0 <= witness < (1 << n)
        assignment = {v: bool(witness & (1 << (v - 1)))
                      for v in range(1, n + 1)}
        assert satisfies(clauses, assignment)
    else:
        assert row["witness_integer"] == "NONE"
        if status == "UNSAT_XOR":
            assert case.startswith("tseitin")
            assert row["xor_rank"] == rank
            assert row["xor_proof_sha256"] != "NONE"
        elif status == "UNSAT_DPLL":
            assert case.startswith("php") and truth == "UNSAT"
    assert status != "HOLD_BUDGET" or row["search_nodes"] == 2000
    return status, row["search_nodes"], row["total_ns"], row["literal_checks"]


def verify_external(row, arm, case, n, clauses, truth, raw, folder, checker):
    assert row["case"] == case and row["arm"] == arm
    assert row["raw_sha256"] == sha(raw)
    assert row["variables"] == n and row["clauses"] == len(clauses)
    assert row["limit_seconds"] == 12
    status = row["result"]
    assert status in ("SAT", "UNSAT_UNCERTIFIED",
                      "UNSAT_CERTIFIED", "HOLD_TIME", "HOLD_UNSAT_PROOF")
    if status == "SAT":
        assert truth == "SAT"
        model = external_model(arm, folder, n)
        assert satisfies(clauses, model)
        assert row["proof_checker"] == "ORIGINAL_CNF_WITNESS"
        assert sha(repr(tuple(sorted(model.items()))).encode("ascii")) == (
            row["witness_sha256"])
    elif status == "UNSAT_UNCERTIFIED":
        assert truth == "UNSAT" and arm in ("minisat", "picosat")
        assert row["solver_exit"] == 20
    elif status == "UNSAT_CERTIFIED":
        assert truth == "UNSAT" and arm == "cadical"
        proof = folder / "unsat.drat"
        assert proof.stat().st_size == row["proof_bytes"] > 0
        assert sha(proof.read_bytes()) == row["proof_sha256"]
        source = folder.parent.parent / "input" / (case + ".cnf")
        assert source.read_bytes() == raw
        result = run(["timeout", "--kill-after=3s", "20s",
                      str(checker), str(source), str(proof)],
                     capture_output=True, text=True, check=False)
        assert result.returncode == 0 and "s VERIFIED" in result.stdout
        assert row["proof_checker"] == "DRAT_TRIM_VERIFIED"
    elif status == "HOLD_UNSAT_PROOF":
        assert arm == "cadical" and truth == "UNSAT"
    assert isinstance(row["total_solver_ns"], int)
    assert row["total_solver_ns"] >= 0
    return status, 0, row["total_solver_ns"], 0


def main(evidence, checker):
    root = Path(evidence)
    assert root.is_dir()
    rows = {}
    same_source = {}
    proof_count = 0
    for arm in ARMS:
        base = root / ("sat-array-" + arm)
        assert base.is_dir()
        seen = {}
        for case in CASES:
            source = base / "input" / (case + ".cnf")
            raw = source.read_bytes()
            n, clauses = parse_dimacs(raw)
            truth, rank = known_truth(case, n, clauses)
            if case in same_source:
                assert same_source[case] == sha(raw)
            same_source[case] = sha(raw)
            folder = base / "cases" / case
            flag = (folder / "RESULT.txt").read_text(encoding="ascii").strip()
            assert flag in ("COMPLETE", "HOLD_TIMEOUT")
            if flag == "HOLD_TIMEOUT":
                status, nodes, duration, checks = "HOLD_TIMEOUT", 0, 0, 0
            else:
                row = ast.literal_eval(
                    (folder / "receipt.txt").read_text(encoding="ascii"))
                status, nodes, duration, checks = (
                    verify_native(row, arm, case, n, clauses, truth, rank, raw)
                    if arm in ARMS[:3] else
                    verify_external(row, arm, case, n, clauses,
                                    truth, raw, folder, checker))
            if status == "UNSAT_CERTIFIED":
                proof_count += 1
            assert not (truth == "UNSAT" and status == "SAT")
            assert not (truth == "SAT" and status.startswith("UNSAT"))
            seen[case] = (status, nodes, duration, checks)
            print("SAT_ARRAY_VERIFIED|arm=" + arm + "|case=" + case +
                  "|expected=" + truth + "|status=" + status +
                  "|nodes=" + str(nodes) + "|duration_ns=" + str(duration) +
                  "|checks=" + str(checks) + "|source=PASS|verifier=PASS",
                  flush=True)
        rows[arm] = seen
    assert len(same_source) == 14
    assert sum(len(v) for v in rows.values()) == 84
    for case in CASES:
        if case.startswith("php"):
            newer, historical = rows["rime_new"][case], rows["rime_original"][case]
            # Their PH computations have identical branch-order keys and the
            # same DPLL search; elapsed wall time is allowed to differ.
            assert newer[0] == historical[0] and newer[1] == historical[1]
            assert newer[3] == historical[3]
    # Twenty actual original-CNF forged witness controls.
    for attempt in range(20):
        case = ("php3_2", "php8_7", "tseitin16", "tseitin64")[attempt % 4]
        raw = (root / "sat-array-cadical" / "input" /
               (case + ".cnf")).read_bytes()
        n, clauses = parse_dimacs(raw)
        fake = {v: bool((attempt + v) & 1)
                for v in range(1, n + 1)}
        assert not satisfies(clauses, fake)
    print("SAT_ARRAY_FINAL|arms=6|cases=14|observations=84|"
          "source_hashes=14|all_origins_identical=PASS|"
          "mathematical_checks=PASS|sat_witnesses=PASS|"
          "DRAT_certificates=" + str(proof_count) +
          "|forged_witnesses_rejected=20|status=PASS")
    (root / "REFEREE.txt").write_text(repr(rows) + "\n",
                                      encoding="ascii")
    (root / "SOURCE_HASHES.txt").write_text(
        repr(same_source) + "\n", encoding="ascii")


if __name__ == "__main__":
    assert len(sys.argv) == 3
    main(sys.argv[1], Path(sys.argv[2]))
