#!/usr/bin/env python3
"""Clean-room SAT arena referee; does not import either contestant.

Checks original CNF, every exact #SAT count and SAT witness, Rime
prime-power navigation receipts, independently rebuilt GF(2) rank,
same node budget, all cost and SHA records, and corruption rejection.
"""
import ast
import hashlib
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1].joinpath("sat")))
import verify_heldout as oracle


def equations_from_cnf(n, clauses):
    groups = {}
    for clause in clauses:
        scope = tuple(sorted(abs(v) for v in clause))
        groups.setdefault(scope, []).append(clause)
    equations = []
    rejected = []
    for scope, block in sorted(groups.items()):
        if len(block) == 4 and len({tuple(sorted(c, key=abs))
                                   for c in block}) == 4:
            permitted = []
            for assignment in range(8):
                bits = sum(((assignment >> j) & 1) << (v - 1)
                           for j, v in enumerate(scope))
                if all(oracle.check(c, bits) for c in block):
                    permitted.append(assignment)
            if len(permitted) == 4 and len(
                    {i.bit_count() & 1 for i in permitted}) == 1:
                rhs = permitted[0].bit_count() & 1
                equations.append((sum(1 << (v - 1) for v in scope), rhs))
                continue
        rejected.extend(block)
    return equations, tuple(rejected)


def independent_affine(n, clauses):
    equations, residual = equations_from_cnf(n, clauses)
    work = [[mask, rhs] for mask, rhs in equations]
    pivots = []
    rank = 0
    for bit in range(n):
        matching = next((j for j in range(rank, len(work))
                         if (work[j][0] >> bit) & 1), None)
        if matching is None:
            continue
        work[rank], work[matching] = work[matching], work[rank]
        for j in range(len(work)):
            if j != rank and work[j][0] & (1 << bit):
                work[j][0] ^= work[rank][0]
                work[j][1] ^= work[rank][1]
        pivots.append(bit)
        rank += 1
    contradiction = any(mask == 0 and rhs for mask, rhs in work)
    free = tuple(j for j in range(n) if j not in pivots)
    expr = [(1 << i, 0) for i in range(n)]
    if contradiction:
        return (), (), residual, rank, True
    for row, p in zip(work, pivots):
        mask, rhs = row
        assert mask & (1 << p)
        expr[p] = (mask ^ (1 << p), rhs)
    for mask, rhs in equations:
        actual_mask, actual_rhs = 0, rhs
        for i in range(n):
            if mask & (1 << i):
                actual_mask ^= expr[i][0]
                actual_rhs ^= expr[i][1]
        assert actual_mask == 0 and actual_rhs == 0
    return tuple(expr), free, residual, rank, False


def verify_rime_digit(row, clauses, n):
    expr, free, residual, rank, conflict = independent_affine(n, clauses)
    assert not conflict
    assert row["rank"] == rank
    assert tuple(sorted(row["variable_order"])) == free
    assert len(row["rime_digit_receipts"]) == len(free)
    source_sha = row["source_sha256"]
    records = []
    for v, target, exponent, proof, coverage, elapsed in row["rime_digit_receipts"]:
        assert v in free and isinstance(elapsed, int) and elapsed >= 0
        raw = hashlib.sha256((source_sha + ":" + str(v)).encode("ascii")).digest()
        expect = 1 + int.from_bytes(raw[:8], "big") % 96
        assert target == expect and 1 <= target < 97
        assert 0 <= exponent < 96 and pow(5, exponent, 97) == target
        # Law-14's two independent prime-power towers at p=97:
        # 2^5 and 3^1, with base-q digit witnesses and exact CRT.
        rows = []
        for q, e in ((2, 5), (3, 1)):
            modulus = pow(q, e)
            residue = exponent % modulus
            digits = tuple((residue // pow(q, j)) % q for j in range(e))
            assert sum(digit * pow(q, j)
                       for j, digit in enumerate(digits)) == residue
            rows.append((q, e, modulus, residue, digits))
        assert proof == oracle.sha((97, 5, target, exponent, tuple(rows)))
        recomputed_coverage = sum(int(any(expr[abs(lit) - 1][0] & (1 << v)
                                         for lit in c)) for c in residual)
        assert coverage == recomputed_coverage
        records.append((v, target, exponent, proof, coverage))
    expect_order = tuple(v for v, _, _ in sorted(
        (-covered, exponent, v) for v, target, exponent, proof, covered in records))
    assert row["variable_order"] == expect_order
    return tuple(records)


def original_count(kind, n, clauses):
    if kind == "xor":
        return oracle.independent_xor_count(n, clauses)
    if kind == "unsat":
        assert len({tuple(c) for c in clauses[-8:]}) == 8
        return 0
    return oracle.counted_dpll(n, clauses, 3000000)


def verify_row(row, n, kind, clauses, actual, independently_counted):
    assert row["variables"] == n and row["kind"] == kind
    source_sha = oracle.sha(clauses)
    assert row["source_sha256"] == source_sha
    proof, recognized, residual, rank, conflict = oracle.independent_reductions(
        n, clauses, source_sha)
    assert row["proof_sha256"] == proof
    assert row["recognized"] == recognized and row["residual"] == residual
    assert row["rank"] == rank
    assert row["free"] == (0 if conflict else n - rank)
    assert row["status"] in ("EXACT", "EXACT_UNSAT", "HOLD_BUDGET")
    if row["status"] == "HOLD_BUDGET":
        assert row["model_count"] == "HOLD"
        assert row["witness_integer"] == "NONE"
        assert row["search_nodes"] == 20000
        count, witness = None, None
    else:
        count = row["model_count"]
        assert isinstance(count, int) and count >= 0
        assert independently_counted is not None and count == independently_counted
        witness = row["witness_integer"]
        if count:
            assert isinstance(witness, int) and 0 <= witness < 1 << n
            assert all(oracle.check(c, witness) for c in clauses)
        else:
            assert witness == "NONE"
            witness = None
    assert row["witness_sha256"] == oracle.sha(witness)
    assert isinstance(row["source_generation_ns"], int) and row["source_generation_ns"] >= 0
    assert isinstance(row["total_process_ns"], int) and row["total_process_ns"] >= 0
    assert isinstance(row["process_launch_and_solve_ns"], int) and row["process_launch_and_solve_ns"] > 0
    assert isinstance(row["isolated_peak_rss_kib"], int) and row["isolated_peak_rss_kib"] > 0
    assert isinstance(row["metered_units"], int) and row["metered_units"] > 0
    assert isinstance(row["search_nodes"], int) and 0 <= row["search_nodes"] <= 20000
    if row["method"] == "YANG_MILLS_FROZEN":
        assert isinstance(row["fixed_comparison_nodes"], int)
        assert 0 <= row["fixed_comparison_nodes"] <= 20000
        assert row["trace_sha256"] == oracle.sha((
            kind, n, source_sha, count, rank, proof, row["status"]))
    else:
        assert row["method"] == "RIME_PH_BSGS_CRT"
        digits = verify_rime_digit(row, clauses, n)
        assert row["rime_navigation_ops"] > 0 or row["free"] == 0
        assert all(isinstance(row[k], int) and row[k] >= 0 for k in
                   ("preprocess_ns", "navigation_ns", "search_ns", "verification_ns"))
        assert row["trace_sha256"] == oracle.sha((
            source_sha, proof, tuple(row["variable_order"]), digits,
            row["status"], count))


def tamper_controls(pair, n, kind, clauses, expected):
    row = pair["rime"]
    bad_fields = {
        "model_count": -1,
        "source_sha256": "0" * 64,
        "proof_sha256": "0" * 64,
        "trace_sha256": "0" * 64,
        "rank": row["rank"] + 1,
        "residual": row["residual"] + 1,
        "free": row["free"] + 1,
        "metered_units": -1,
        "isolated_peak_rss_kib": -1,
        "search_nodes": 20001,
        "witness_sha256": "0" * 64,
    }
    caught = 0
    for key, corrupted in bad_fields.items():
        bad = dict(row)
        bad[key] = corrupted
        try:
            verify_row(bad, n, kind, clauses, expected, expected)
        except (AssertionError, ValueError):
            caught += 1
        else:
            raise AssertionError("tampered competitor accepted: " + key)
    assert caught == 11
    return caught


def main(folder):
    root = Path(folder)
    expected = ([("xor", n) for n in (24, 48, 96)] +
                [("mixed", n) for n in (12, 24, 36)] +
                [("planted", n) for n in (12, 18, 24, 30)] +
                [("random", n) for n in (12, 18, 24, 30)] +
                [("hub", n) for n in (12, 18, 24, 30)] +
                [("unsat", n) for n in (12, 24, 36)])
    exact, hold = {"legacy": 0, "rime": 0}, {"legacy": 0, "rime": 0}
    checksum = 0
    for idx, (kind, n) in enumerate(expected):
        name = kind + "-" + str(n)
        location = root.joinpath("cases", name)
        original = location.joinpath("source.cnf").read_bytes()
        parsed_n, clauses = oracle.independent_dimacs(original.decode("ascii"))
        assert parsed_n == n
        seed = "frozen-" + kind + "-" + str(n) + "-v1"
        assert clauses == oracle.expected_fixture(kind, n, seed)
        pair = ast.literal_eval(location.joinpath("PAIR.txt").read_text(encoding="ascii"))
        assert pair["case"] == name and pair["variables"] == n
        assert pair["seed"] == seed and pair["kind"] == kind
        assert pair["original_dimacs_sha256"] == hashlib.sha256(original).hexdigest()
        assert pair["alternated_execution_order"] == (
            ("legacy", "rime") if idx % 2 == 0 else ("rime", "legacy"))
        independently_counted = original_count(kind, n, clauses)
        for arm in ("legacy", "rime"):
            row = pair[arm]
            assert row["seed"] == seed
            verify_row(row, n, kind, clauses, independently_counted,
                       independently_counted)
            receipt = location.joinpath(arm + ".receipt").read_text(encoding="ascii")
            expected_line = "PNP_MATCH|" + repr({
                k: v for k, v in row.items() if k not in (
                    "process_launch_and_solve_ns", "isolated_peak_rss_kib")}) + "|json=0"
            assert expected_line in receipt
            if row["status"] == "HOLD_BUDGET":
                hold[arm] += 1
            else:
                exact[arm] += 1
        if idx == 0:
            checksum = tamper_controls(pair, n, kind, clauses, independently_counted)
        print("PNP_COMPETITION_REFEREE|case=" + name +
              "|source=PASS|both_model_counts=PASS|"
              "both_receipts=PASS|status=PASS", flush=True)
    assert exact["legacy"] + hold["legacy"] == len(expected) == 21
    assert exact["rime"] + hold["rime"] == len(expected)
    print("PNP_COMPETITION_FINAL|cases=21|legacy_exact=" +
          str(exact["legacy"]) + "|legacy_hold=" + str(hold["legacy"]) +
          "|rime_exact=" + str(exact["rime"]) + "|rime_hold=" +
          str(hold["rime"]) + "|source_equal=PASS|"
          "original_CNF_model_counts=PASS|"
          "rime_prime_power_dlogs=PASS|"
          "corrupt_receipts_rejected=" + str(checksum) +
          "|full_source_and_receipt_verification=PASS|status=PASS")


if __name__ == "__main__":
    assert len(sys.argv) == 2
    main(sys.argv[1])
