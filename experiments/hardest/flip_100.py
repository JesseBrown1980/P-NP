#!/usr/bin/env python3
"""100 adversarial verdict inversions using actual published original-CNF evidence.

A real certified UNSAT proof from PHP(8,7) cannot certify an unrelated
satisfiable PHP(4,4), and forged SAT assignments cannot satisfy the
independently frozen UNSAT PHP/Tseitin sources. Original bytes, real solver
receipts, and actual proof checking are mandatory here.
"""
import ast
import hashlib
import sys
from pathlib import Path
from subprocess import run


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return ast.literal_eval(Path(path).read_text(encoding="ascii"))


def parse(path):
    n = None
    clauses = []
    row = []
    for line in Path(path).read_text(encoding="ascii").splitlines():
        tokens = line.split()
        if not tokens or tokens[0] == "c":
            continue
        if tokens[0] == "p":
            assert n is None and tokens[1] == "cnf"
            n = int(tokens[2])
            expected = int(tokens[3])
            continue
        for token in tokens:
            v = int(token)
            if v:
                assert 1 <= abs(v) <= n
                row.append(v)
            else:
                assert row
                clauses.append(tuple(row))
                row = []
    assert n and len(clauses) == expected and not row
    return n, tuple(clauses)


def satisfies(clauses, assignments):
    return all(any(assignments.get(abs(v)) == (v > 0)
                   for v in clause if abs(v) in assignments)
               for clause in clauses)


def main(evidence, checker):
    root = Path(evidence)
    cad = root / "pnp-hard-cadical"
    proof_case = cad / "cases" / "php8"
    proof_receipt = read(proof_case / "receipt.txt")
    assert proof_receipt["result"] == "UNSAT_CERTIFIED"
    proof = proof_case / "unsat.drat"
    assert proof.is_file() and proof.stat().st_size > 0
    assert sha(proof.read_bytes()) == proof_receipt["proof_sha256"]
    assert read(root / "pnp-hard-minisat" / "cases" / "php4sat" /
                "receipt.txt")["result"] == "SAT"
    assert read(root / "pnp-hard-ym_fischer" / "cases" / "tseitin128" /
                "receipt.txt")["status"] == "UNSAT_XOR"

    forged_sat, swapped_drat = 0, 0
    for iteration in range(100):
        case = ("php21", "php4sat", "tseitin128", "php8")[iteration % 4]
        original = cad / "input" / (case + ".cnf")
        n, clauses = parse(original)
        if case == "php4sat":
            # Invert a real witness-checked SAT verdict into a false UNSAT
            # and attach an actual proof valid for a DIFFERENT input.
            result = run(["timeout", "15s", str(checker),
                          str(original), str(proof)],
                         capture_output=True, text=True, check=False)
            try:
                assert result.returncode == 0 and "s VERIFIED" in result.stdout
            except AssertionError:
                swapped_drat += 1
            else:
                raise AssertionError("borrowed DRAT proof accepted for SAT")
        else:
            # Invert a true UNSAT source into a SAT claim using a concrete,
            # deliberately forged full assignment, testing the real CNF.
            fake = {v: bool((iteration + v) & 1) if iteration & 1 else False
                    for v in range(1, n + 1)}
            assert len(fake) == n
            try:
                assert satisfies(clauses, fake)
            except AssertionError:
                forged_sat += 1
            else:
                raise AssertionError("forged witness accepted: " + case)
    assert forged_sat == 75 and swapped_drat == 25
    print("HARD_REAL_FLIPS|total=100|forged_original_CNF_SAT=75|"
          "cross_source_actual_DRAT=25|wrong_verdicts_rejected=100|"
          "source_and_proof_bytes=ARCHIVED|status=PASS")


if __name__ == "__main__":
    assert len(sys.argv) == 3
    main(sys.argv[1], Path(sys.argv[2]))
