#!/usr/bin/env python3
"""Independent receipt and exact-count checker for the P-NP selector pilot.

Reads the original frozen CNF files, not the engine module. Cross-checks
representation selection, every recorded candidate, original witnesses,
count certification, SHA chains and independently reconstructed XOR rank.
"""
import ast
import hashlib
import sys
from pathlib import Path
import verify_heldout as oracle


def decode(line):
    result = {}
    for field in line.strip().split("|")[1:]:
        assert "=" in field
        key, value = field.split("=", 1)
        assert key not in result
        result[key] = value
    assert result.pop("json") == "0"
    return result


def inspect_receipt(row, root, check_count=True):
    name = row["kind"] + "-" + row["variables"]
    raw_bytes = root.joinpath(name + ".cnf").read_bytes()
    n, clauses = oracle.independent_dimacs(raw_bytes.decode("ascii"))
    assert n == int(row["variables"])
    assert row["seed"] == ("frozen-" + row["kind"] + "-" + str(n) + "-v1")
    assert clauses == oracle.expected_fixture(row["kind"], n, row["seed"])
    assert hashlib.sha256(raw_bytes).hexdigest() == row["raw_dimacs_sha256"]
    frozen = oracle.sha(clauses)
    assert row["source_sha256"] == frozen
    assert int(row["clauses"]) == len(clauses)
    proof, recognized, residual, rank, inconsistent = oracle.independent_reductions(
        n, clauses, frozen)
    assert row["proof_sha256"] == proof
    assert int(row["recognized"]) == recognized
    assert int(row["residual"]) == residual
    assert int(row["rank"]) == rank
    assert int(row["free"]) == (0 if inconsistent else n - rank)
    assert int(row["source_bytes"]) == len(repr(clauses).encode("ascii"))
    assert int(row["probe_limit"]) == 96
    assert int(row["full_limit"]) == 20000
    assert int(row["generation_ns"]) >= 0
    assert int(row["elapsed_ns"]) >= int(row["generation_ns"])
    probes = ast.literal_eval(row["probed_candidates"])
    executions = ast.literal_eval(row["candidate_executions"])
    assert isinstance(probes, tuple) and isinstance(executions, tuple)
    if recognized:
        expected_names = {"affine_fischer", "affine_fixed", "raw_fixed"}
    else:
        expected_names = {"raw_fischer", "raw_fixed"}
    assert {p[0] for p in probes} == expected_names
    assert len(probes) == len(expected_names)
    for name, state, nodes, literals, cost in probes:
        assert name in expected_names
        assert state in ("EXACT", "HOLD_BUDGET")
        assert 0 < nodes <= 96 and 0 <= literals <= cost
    ranked = sorted((0 if state == "EXACT" else 1, cost, name)
                    for name, state, nodes, literals, cost in probes)
    assert row["initial_selection"] == ranked[0][2]
    assert len(executions) >= 1 and len(executions) <= len(probes)
    assert [r[0] for r in executions] == [r[2] for r in ranked[:len(executions)]]
    for name, state, nodes, literals, cost in executions:
        assert name in expected_names and state in ("EXACT", "HOLD_BUDGET")
        assert 0 < nodes <= 20000 and 0 <= literals <= cost
    if executions[-1][1] == "EXACT":
        assert all(r[1] == "HOLD_BUDGET" for r in executions[:-1])
        assert row["status"] == "EXACT"
        assert row["selected_representation"] == executions[-1][0]
    else:
        assert len(executions) == len(probes)
        assert row["status"] == "HOLD_BUDGET"
        assert row["selected_representation"] == "NONE"
    assert int(row["all_probes_metered_units"]) == sum(r[-1] for r in probes)
    assert int(row["all_executions_metered_units"]) == sum(r[-1] for r in executions)
    total = int(row["source_bytes"]) + len(clauses)
    total += int(row["preprocessing_metered_units"])
    total += int(row["all_probes_metered_units"])
    total += int(row["all_executions_metered_units"])
    total += len(raw_bytes)
    assert int(row["total_metered_units"]) == total
    assert row["trace_sha256"] == oracle.sha((frozen, proof, probes, executions))
    if row["status"] == "HOLD_BUDGET":
        assert row["model_count"] == "HOLD" and row["witness_integer"] == "NONE"
        declared, witness = None, None
    else:
        declared = int(row["model_count"])
        assert declared >= 0
        if declared:
            witness = int(row["witness_integer"])
            assert 0 <= witness < (1 << n)
            assert all(oracle.check(c, witness) for c in clauses)
        else:
            assert row["witness_integer"] == "NONE"
            witness = None
    assert row["witness_sha256"] == oracle.sha(witness)
    assert row["receipt_sha256"] == oracle.sha((
        frozen, proof, n, rank, declared, row["status"],
        row["initial_selection"], row["selected_representation"],
        row["trace_sha256"], oracle.sha(witness)))
    if check_count:
        if row["kind"] == "xor":
            expected = oracle.independent_xor_count(n, clauses)
        elif row["kind"] == "unsat":
            expected = 0
        else:
            expected = oracle.counted_dpll(n, clauses, 3000000)
        if declared is not None:
            assert expected is not None and declared == expected
        return declared
    return declared


def corruption_tests(rows, folder):
    row = next(r for r in rows if r["kind"] == "xor")
    assert inspect_receipt(row, folder, check_count=False) is not None
    mutations = {
        "source_sha256": "0" * 64,
        "proof_sha256": "0" * 64,
        "trace_sha256": "0" * 64,
        "receipt_sha256": "0" * 64,
        "witness_sha256": "0" * 64,
        "recognized": str(int(row["recognized"]) + 1),
        "free": str(int(row["free"]) + 1),
        "total_metered_units": str(int(row["total_metered_units"]) + 1),
        "selected_representation": "raw_fixed",
        "model_count": str(int(row["model_count"]) + 1),
    }
    for name, replacement in mutations.items():
        changed = dict(row)
        changed[name] = replacement
        try:
            inspect_receipt(changed, folder, check_count=False)
        except (AssertionError, ValueError):
            pass
        else:
            raise AssertionError("selector receipt tamper accepted: " + name)
    print("PNP_SELECTOR_TAMPER|altered_receipts_rejected=10|status=PASS|json=0")


def main(receipt_path, folder):
    lines = Path(receipt_path).read_text(encoding="ascii").splitlines()
    assert any(line.startswith("PNP_SELECTOR_CONTROL|") for line in lines)
    assert any(line.startswith("PNP_SELECTOR_FINAL|cases=21|") for line in lines)
    rows = [decode(line) for line in lines if line.startswith("PNP_SELECTOR|")]
    expected = ([("xor", n) for n in (24, 48, 96)] +
                [("mixed", n) for n in (12, 24, 36)] +
                [("planted", n) for n in (12, 18, 24, 30)] +
                [("random", n) for n in (12, 18, 24, 30)] +
                [("hub", n) for n in (12, 18, 24, 30)] +
                [("unsat", n) for n in (12, 24, 36)])
    assert len(rows) == len(expected)
    assert [(r["kind"], int(r["variables"])) for r in rows] == expected
    root = Path(folder)
    assert len(tuple(root.glob("*.cnf"))) == len(rows)
    corruption_tests(rows, root)
    values = [inspect_receipt(r, root) for r in rows]
    exact = sum(value is not None for value in values)
    holds = len(values) - exact
    assert exact >= 1
    assert holds >= 0
    print("PNP_SELECTOR_INDEPENDENT|cases=21|exact=" + str(exact) +
          "|hold=" + str(holds) + "|source_proof_rank=PASS|"
          "candidate_score_and_total_cost=PASS|original_CNF_counts=PASS|"
          "original_CNF_witnesses=PASS|SHA_trace_replay=PASS|"
          "corruption_rejection=PASS|status=PASS|json=0")


if __name__ == "__main__":
    assert len(sys.argv) == 3
    main(sys.argv[1], sys.argv[2])
