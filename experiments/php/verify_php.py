#!/usr/bin/env python3
"""Clean-room PHP source/witness/DRAT gate with exactly 100 false-verdict flips.

Never imports production php.py. The mathematical PHP count and clauses
are recomputed separately, from the original specification.
"""
import ast
import hashlib
import sys
from pathlib import Path
from subprocess import run
from time import monotonic_ns

CASES = ((2, 1), (2, 2), (3, 2), (3, 3), (4, 3))
VERSION = "php-basic-v1"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def construct(p, h, functional=False):
    alo, hole, pigeon = [], [], []
    for i in range(1, p + 1):
        alo.append(tuple((i - 1) * h + j for j in range(1, h + 1)))
    for j in range(1, h + 1):
        for i in range(1, p + 1):
            for k in range(i + 1, p + 1):
                hole.append((-((i - 1) * h + j), -((k - 1) * h + j)))
    if functional:
        for i in range(1, p + 1):
            for j in range(1, h + 1):
                for k in range(j + 1, h + 1):
                    pigeon.append((-((i - 1) * h + j),
                                   -((i - 1) * h + k)))
    return tuple(alo), tuple(hole), tuple(pigeon)


def parse(data):
    n, size = None, None
    clauses, queue = [], []
    for line in data.decode("ascii").splitlines():
        items = line.split()
        if not items or items[0] == "c":
            continue
        if items[0] == "p":
            assert n is None and len(items) == 4 and items[1] == "cnf"
            n, size = int(items[2]), int(items[3])
            assert n > 0 and size >= 0
        else:
            assert n is not None
            for item in items:
                value = int(item)
                if value:
                    assert abs(value) <= n
                    queue.append(value)
                else:
                    assert queue and len(set(map(abs, queue))) == len(queue)
                    clauses.append(tuple(queue))
                    queue = []
    assert n and not queue and len(clauses) == size
    return n, tuple(clauses)


def source(root, p, h, functional=False):
    name = "php-" + str(p) + "-" + str(h) + (
        "-functional" if functional else "")
    path = root / (name + ".cnf")
    data = path.read_bytes()
    assert data.startswith(("c P-NP original PHP source, generator " +
                            VERSION + "\n").encode("ascii"))
    n, actual = parse(data)
    expected_types = construct(p, h, functional)
    assert n == p * h
    assert actual == expected_types[0] + expected_types[1] + expected_types[2]
    lengths = tuple(len(part) for part in expected_types)
    assert lengths == (p, h * p * (p - 1) // 2,
                       p * h * (h - 1) // 2 if functional else 0)
    if (p, h) == (21, 20):
        assert n == 420
        assert lengths == ((21, 4200, 3990) if functional
                           else (21, 4200, 0))
        assert len(actual) == (8211 if functional else 4221)
    return name, path, n, actual, sha(data)


def parse_model(tokens, n):
    values = {}
    zero = False
    for token in tokens:
        literal = int(token)
        assert not zero
        if literal == 0:
            zero = True
        else:
            v = abs(literal)
            assert 1 <= v <= n and v not in values
            values[v] = literal > 0
    assert values and zero
    return values


def satisfied(clauses, values):
    # A partial assignment is accepted only if it satisfies every
    # original clause from an *assigned* literal, not by default.
    assert values
    return all(any(values.get(abs(x)) == (x > 0)
                   for x in clause if abs(x) in values)
               for clause in clauses)


def minisat_solution(path, n, expect):
    fields = path.read_text(encoding="ascii").split()
    assert fields and fields[0] == ("SAT" if expect == "SAT" else "UNSAT")
    if expect == "UNSAT":
        assert fields == ["UNSAT"]
        return None
    return parse_model(fields[1:], n)


def cadical_solution(path, n, expect):
    lines = path.read_text(encoding="ascii").splitlines()
    outcomes = [line[2:].strip() for line in lines if line.startswith("s ")]
    assert outcomes == [("SATISFIABLE" if expect == "SAT" else "UNSATISFIABLE")]
    literals = [x for line in lines if line.startswith("v ")
                for x in line.split()[1:]]
    if expect == "UNSAT":
        assert not literals
        return None
    return parse_model(literals, n)


def check_drat(binary, cnf, proof, transcript):
    start = monotonic_ns()
    result = run([str(binary), str(cnf), str(proof)],
                 capture_output=True, text=True, check=False)
    elapsed = monotonic_ns() - start
    if transcript is not None:
        transcript.write_text(result.stdout + result.stderr, encoding="ascii")
    return result.returncode == 0 and "s VERIFIED" in result.stdout, elapsed


def receipt_types(root):
    all_cases = []
    for p, h in CASES:
        name, path, n, clauses, digest = source(root, p, h)
        native = ast.literal_eval(
            (root / "native" / (name + ".txt")).read_text(encoding="ascii"))
        outcome = "SAT" if p <= h else "UNSAT"
        assert native["instance"] == name and native["input_sha256"] == digest
        assert native["verdict"] == outcome
        assert 0 < native["search_nodes"] <= 300000
        assert all(isinstance(native[k], int) and native[k] >= 0
                   for k in ("parse_ns", "preprocess_ns", "search_ns",
                             "witness_check_ns"))
        if outcome == "SAT":
            model = {k: bool(native["witness"] & (1 << (k - 1)))
                     for k in range(1, n + 1)}
            assert satisfied(clauses, model)
        else:
            assert native["witness"] is None
        logs = root / "solver" / name
        baseline = ast.literal_eval(
            (logs / "receipt.txt").read_text(encoding="ascii"))
        assert baseline["instance"] == name
        assert baseline["source_sha256"] == digest
        assert baseline["native_receipt_sha256"] == sha(
            (root / "native" / (name + ".txt")).read_bytes())
        assert baseline["minisat_verdict"] == baseline["cadical_verdict"]
        assert baseline["minisat_verdict"] == (10 if outcome == "SAT" else 20)
        assert all(isinstance(baseline[k], int) and baseline[k] >= 0
                   for k in ("minisat_search_ns", "cadical_search_ns",
                             "minisat_peak_kib", "cadical_peak_kib",
                             "proof_generation_ns", "proof_validation_ns",
                             "drat_bytes"))
        mini = minisat_solution(logs / "minisat.sol", n, outcome)
        cad = cadical_solution(logs / "cadical.log", n, outcome)
        if outcome == "SAT":
            assert satisfied(clauses, mini) and satisfied(clauses, cad)
            assert baseline["drat_bytes"] == 0
        else:
            assert mini is None and cad is None
            proof = logs / "cadical.drat"
            assert proof.is_file() and proof.stat().st_size == baseline["drat_bytes"] > 0
            verdict, elapsed = check_drat(
                DRAT_CHECKER, path, proof, logs / "drat-independent.log")
            assert verdict and elapsed > 0
        all_cases.append((name, outcome, mini, cad, path, clauses, baseline))
        print("PHP_AUDIT|case=" + name + "|decision=" + outcome +
              "|source_sha=PASS|native_vs_external=PASS|certificate=" +
              ("ORIGINAL_CNF_MODELS" if outcome == "SAT"
               else "DRAT_TRIM_VERIFIED") + "|status=PASS", flush=True)
    assert len(all_cases) == 5
    return all_cases


def reject_wrong_claim(claim, case, fake=False):
    name, correct, mini, cad, source_path, clauses, baseline = case
    if claim == "SAT":
        # A flipped UNSAT cannot create a valid witness by declaring it:
        # require each claimed independent model to satisfy original clauses.
        if fake:
            n = baseline["pigeons"] * baseline["holes"]
            mini = {v: False for v in range(1, n + 1)}
            cad = dict(mini)
        assert mini is not None and cad is not None
        assert satisfied(clauses, mini) and satisfied(clauses, cad)
    elif claim == "UNSAT":
        # Require a proof generated for *this* original CNF; replacing the
        # verdict with a word is not enough. Using another instance's
        # proof must be caught by drat-trim.
        candidate = source_path.parent / "solver" / name / "cadical.drat"
        if fake:
            candidate = source_path.parent / "solver" / "php-2-1" / "cadical.drat"
        assert candidate.is_file()
        verified, _ = check_drat(DRAT_CHECKER, source_path, candidate, None)
        assert verified
    else:
        raise AssertionError("unsupported verdict")


def flip_100(root, cases):
    rejected = 0
    for iteration in range(20):
        for case in cases:
            flipped = "UNSAT" if case[1] == "SAT" else "SAT"
            # Alternate bare flipped verdict versus forged evidence.
            forged = bool(iteration & 1)
            try:
                reject_wrong_claim(flipped, case, fake=forged)
            except (AssertionError, ValueError, FileNotFoundError):
                rejected += 1
            else:
                raise AssertionError("incorrect verdict was accepted: " +
                                     case[0] + ":" + str(iteration))
    assert rejected == 100
    print("PHP_FLIP|total=100|rejected=100|false_acceptances=0|"
          "bare_and_forged_evidence=PASS|status=PASS", flush=True)


def main(root):
    large = source(root, 21, 20)
    functional = source(root, 21, 20, True)
    assert len(large[3]) == 4221 and len(functional[3]) == 8211
    print("PHP_SPEC|php_21_20_variables=420|at_least_one=21|"
          "hole_at_most_one=4200|functional_extra=3990|"
          "basic_clauses=4221|functional_clauses=8211|"
          "independent_recount=PASS", flush=True)
    cases = receipt_types(root)
    flip_100(root, cases)
    print("PHP_INDEPENDENT_FINAL|source_large=PASS|five_cases=PASS|"
          "two_solver_verdicts=AGREE|original_models=PASS|"
          "checked_DRAT_proofs=PASS|100_flips=REJECTED|"
          "status=PASS", flush=True)


if __name__ == "__main__":
    assert len(sys.argv) == 3
    DRAT_CHECKER = Path(sys.argv[2]).resolve()
    assert DRAT_CHECKER.is_file()
    main(Path(sys.argv[1]))
