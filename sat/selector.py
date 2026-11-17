#!/usr/bin/env python3
"""Exact SAT representation selector pilot.

The original authenticated clauses are the only source of truth. Evaluate
bounded probes of several exact representations, score their observed work,
then fully execute the selected candidate (with metered fallbacks). Both
failed probes and independent watchers are charged to total work.
"""
import hashlib
import sys
from pathlib import Path
from time import monotonic_ns

import engine


METRIC_KEYS = (
    "group_inserts", "local_literal_checks", "watcher_clause_checks",
    "watcher_rejections", "accepted_transforms", "fischer_row_scores",
    "pivot_probes", "row_xors", "xor_bitwork_bound", "basis_peak_bits",
    "watcher_pivot_probes", "watcher_row_xors", "watcher_bitwork_bound",
    "backsub_xors", "watcher_expression_checks", "fischer_literal_scores",
    "search_nodes", "search_literal_checks", "witness_literal_checks",
)
WORK_KEYS = tuple(key for key in METRIC_KEYS if key != "basis_peak_bits")


def metrics():
    return {key: 0 for key in METRIC_KEYS}


def work_used(meter):
    return sum(meter[key] for key in WORK_KEYS)


def identity(n):
    return tuple((1 << i, 0) for i in range(n)), tuple(range(n))


def candidate_data(name, clauses, residual, expressions, free, n):
    if name.startswith("affine"):
        return residual, expressions, free, name.split("_")[1]
    assert name.startswith("raw")
    raw_expr, raw_free = identity(n)
    return clauses, raw_expr, raw_free, name.split("_")[1]


def apply_candidate(name, n, clauses, residual, expr, free, cap):
    search_metric = metrics()
    required, representation, variables, ordering = candidate_data(
        name, clauses, residual, expr, free, n)
    count, witness, status = engine.bounded_exact(
        n, clauses, required, representation, variables,
        cap, search_metric, ordering)
    assert status in ("EXACT", "HOLD_BUDGET")
    if count is not None:
        assert status == "EXACT"
        if count:
            assert witness is not None
            assert engine.clauses_hold(clauses, witness)
        else:
            assert witness is None
    return count, witness, status, search_metric


def prepare(n, clauses):
    frozen = engine.sha(clauses)
    m = metrics()
    equations, residual, proof, corrected = engine.compile_source(
        n, clauses, m, tamper_first=True)
    basis = engine.right_eliminate(n, equations, m)
    rank, contradiction = engine.left_rank(n, equations, m)
    assert (basis is None) == contradiction
    assert engine.sha(clauses) == frozen
    if contradiction:
        return frozen, proof, corrected, equations, residual, rank, None, (), m, True
    assert len(basis) == rank
    expr, free = engine.affine_expressions(n, basis, equations, m)
    return frozen, proof, corrected, equations, residual, rank, expr, free, m, False


def solve(n, clauses, probe_limit=96, full_limit=20000):
    engine.validate_source(n, clauses)
    started = monotonic_ns()
    source, proof, corrected, rows, residual, rank, expr, free, prep, impossible = prepare(
        n, clauses)
    if impossible:
        count, witness, status = 0, None, "EXACT_UNSAT"
        chosen, initial = "parity_contradiction", "parity_contradiction"
        scores = ()
        probes = []
        executions = []
    else:
        candidate_names = (
            ("affine_fischer", "affine_fixed", "raw_fixed")
            if rows else ("raw_fischer", "raw_fixed")
        )
        probes = []
        ranked = []
        for name in candidate_names:
            count0, witness0, state0, meter = apply_candidate(
                name, n, clauses, residual, expr, free, probe_limit)
            cost = work_used(meter)
            # An exact completed probe ranks ahead of an incomplete probe.
            # Otherwise compare actual per-probe work; this is a heuristic,
            # not an oracle for subsequent full search complexity.
            score = (0 if state0 == "EXACT" else 1, cost, name)
            probes.append((name, state0, meter["search_nodes"],
                           meter["search_literal_checks"], cost))
            ranked.append(score)
        ranked.sort()
        initial = ranked[0][2]
        count, witness, status = None, None, "HOLD_BUDGET"
        chosen = "NONE"
        executions = []
        for _, score, name in ranked:
            result, candidate_witness, state, meter = apply_candidate(
                name, n, clauses, residual, expr, free, full_limit)
            executions.append((name, state, meter["search_nodes"],
                               meter["search_literal_checks"], work_used(meter)))
            if state == "EXACT":
                count, witness, status, chosen = result, candidate_witness, state, name
                break
        if status == "HOLD_BUDGET":
            assert count is None and witness is None
    assert engine.sha(clauses) == source
    if count is not None and witness is not None:
        assert engine.clauses_hold(clauses, witness)
    spent = len(repr(clauses).encode("ascii")) + len(clauses) + work_used(prep)
    spent += sum(row[-1] for row in probes)
    spent += sum(row[-1] for row in executions)
    trace_sha = engine.sha((source, proof, tuple(probes), tuple(executions)))
    receipt_sha = engine.sha((source, proof, n, rank, count, status,
                             initial, chosen, trace_sha, engine.sha(witness)))
    return {
        "variables": n,
        "clauses": len(clauses),
        "recognized": len(rows),
        "residual": len(residual),
        "rank": rank,
        "free": n - rank if not impossible else 0,
        "status": status,
        "model_count": count if count is not None else "HOLD",
        "witness_integer": witness if witness is not None else "NONE",
        "witness_sha256": engine.sha(witness),
        "source_sha256": source,
        "proof_sha256": proof,
        "trace_sha256": trace_sha,
        "receipt_sha256": receipt_sha,
        "corrected_proposals": corrected,
        "probed_candidates": tuple(probes),
        "candidate_executions": tuple(executions),
        "initial_selection": initial,
        "selected_representation": chosen,
        "source_bytes": len(repr(clauses).encode("ascii")),
        "preprocessing_metered_units": work_used(prep),
        "all_probes_metered_units": sum(row[-1] for row in probes),
        "all_executions_metered_units": sum(row[-1] for row in executions),
        "total_metered_units": spent,
        "probe_limit": probe_limit,
        "full_limit": full_limit,
        "elapsed_ns": monotonic_ns() - started,
        "peak_rss_kib": engine.peak_kib(),
    }


def controls():
    conflict = engine.xor_clauses((1, 2, 3), 0) + engine.xor_clauses((1, 2, 3), 1)
    result = solve(3, conflict)
    assert result["model_count"] == 0
    assert result["status"] == "EXACT"
    missing = engine.xor_clauses((1, 2, 3), 0)[:3]
    result = solve(3, missing)
    assert result["recognized"] == 0
    assert engine.sha(conflict) != engine.sha(missing)
    print("PNP_SELECTOR_CONTROL|opposite_parity=UNSAT|incomplete_motif=REJECTED|"
          "source_mutation_sha=PASS|status=PASS|json=0")


def main(folder):
    root = Path(folder)
    root.mkdir(parents=True, exist_ok=True)
    controls()
    statuses = []
    for kind, n, seed in engine.heldout_cases():
        started = monotonic_ns()
        clauses = engine.heldout_source(kind, n, seed)
        raw = engine.dimacs(n, clauses)
        generation_ns = monotonic_ns() - started
        assert engine.parse_dimacs(raw) == (n, clauses)
        root.joinpath(kind + "-" + str(n) + ".cnf").write_text(raw, encoding="ascii")
        result = solve(n, clauses)
        result["kind"] = kind
        result["seed"] = seed
        result["raw_dimacs_sha256"] = hashlib.sha256(raw.encode("ascii")).hexdigest()
        result["generation_ns"] = generation_ns
        result["elapsed_ns"] += generation_ns
        result["total_metered_units"] += len(raw)
        statuses.append(result["status"])
        print("PNP_SELECTOR|" + "|".join(k + "=" + str(v)
              for k, v in result.items()) + "|json=0", flush=True)
    assert len(statuses) == 21
    print("PNP_SELECTOR_FINAL|cases=21|exact=" + str(statuses.count("EXACT")) +
          "|hold=" + str(statuses.count("HOLD_BUDGET")) +
          "|source_frozen=PASS|watcher_replay=REQUIRED|"
          "all_candidate_costs_metered=PASS|P_equals_NP=OPEN|status=RESEARCH_PILOT|json=0")


if __name__ == "__main__":
    assert len(sys.argv) == 2
    main(sys.argv[1])
