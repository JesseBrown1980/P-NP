#!/usr/bin/env python3
"""Fair-task P-NP SAT contest: frozen Yang-Mills kernel vs Rime Fischer adapter.

Same original 21 held-out CNFs, same 20,000 exact #SAT search-node cap,
same separate process per arm. The incumbent is the upstream engine
unchanged. The challenger uses the same independently watched XOR
representation but replaces residual variable-order scoring with a
Pohlig--Hellman/BSGS/CRT-derived ordering. No transfer theorem is assumed.
"""
import hashlib
import sys
from pathlib import Path
from time import monotonic_ns

import engine

sys.path.insert(0, str(Path(__file__).resolve().parents[1].joinpath("algebra")))
import rime_fischer


METRICS = (
    "group_inserts", "local_literal_checks", "watcher_clause_checks",
    "watcher_rejections", "accepted_transforms", "fischer_row_scores",
    "pivot_probes", "row_xors", "xor_bitwork_bound", "basis_peak_bits",
    "watcher_pivot_probes", "watcher_row_xors", "watcher_bitwork_bound",
    "backsub_xors", "watcher_expression_checks", "fischer_literal_scores",
    "search_nodes", "search_literal_checks", "witness_literal_checks",
)


def metrics():
    return {key: 0 for key in METRICS}


def rank_variables(clauses, expr, free, source_sha):
    """Original-CNF influence then verified prime-tower digit address.

    The discrete-log target is derived from original source SHA and variable
    identity, not a planted SAT solution. It only breaks branch-order ties
    and adds a precisely measured computational expense.
    """
    p = 97
    g = rime_fischer.primitive_root(p)
    scores = []
    receipts = []
    operations = 0
    for v in free:
        start = monotonic_ns()
        raw = hashlib.sha256((source_sha + ":" + str(v)).encode("ascii")).digest()
        target = 1 + (int.from_bytes(raw[:8], "big") % (p - 1))
        proof = rime_fischer.recover(p, g, target)
        assert pow(g, proof["exponent"], p) == target
        covered = sum(int(any(expr[abs(lit) - 1][0] & (1 << v)
                              for lit in clause)) for clause in clauses)
        operations += proof["baby_steps"] + proof["giant_steps"]
        operations += proof["modular_multiplies"] + proof["modular_pow_calls"]
        operations += proof["digit_verifications"] + len(clauses) * 3
        scores.append((-covered, proof["exponent"], v))
        receipts.append((v, target, proof["exponent"],
                         proof["result_sha256"], covered,
                         monotonic_ns() - start))
    order = tuple(v for _, _, v in sorted(scores))
    assert set(order) == set(free)
    return order, tuple(receipts), operations


def bounded_count(original, residual, expr, order, limit=20000):
    nodes = [0]
    literals = [0]
    first = [None]

    def visit(known, free_value, depth):
        if nodes[0] >= limit:
            raise OverflowError("declared node budget")
        nodes[0] += 1
        completed = True
        for clause in residual:
            sat = False
            pending = False
            for lit in clause:
                mask, rhs = expr[abs(lit) - 1]
                literals[0] += 1
                if mask & ~known:
                    pending = True
                elif (((mask & free_value).bit_count() & 1) ^ rhs) == int(lit > 0):
                    sat = True
                    break
            if not sat and not pending:
                return 0
            if not sat:
                completed = False
        if completed:
            if first[0] is None:
                first[0] = engine.expand(expr, free_value)
                assert engine.clauses_hold(original, first[0])
            return 1 << (len(order) - depth)
        assert depth < len(order)
        bit = 1 << order[depth]
        return visit(known | bit, free_value, depth + 1) + visit(
            known | bit, free_value | bit, depth + 1)

    try:
        return visit(0, 0, 0), first[0], "EXACT", nodes[0], literals[0]
    except OverflowError:
        return None, None, "HOLD_BUDGET", nodes[0], literals[0]


def rime_challenger(kind, n, clauses):
    sha = engine.source_fingerprint(clauses)
    t0 = monotonic_ns()
    meter = metrics()
    rows, residual, proof, corrected = engine.compile_source(
        n, clauses, meter, tamper_first=True)
    basis = engine.right_eliminate(n, rows, meter)
    independent_rank, inconsistent = engine.left_rank(n, rows, meter)
    assert (basis is None) == inconsistent
    if inconsistent:
        expr, free = (), ()
    else:
        assert len(basis) == independent_rank
        expr, free = engine.affine_expressions(n, basis, rows, meter)
    preprocess_ns = monotonic_ns() - t0

    t1 = monotonic_ns()
    if inconsistent:
        order, digit_receipts, digit_ops = (), (), 0
    else:
        order, digit_receipts, digit_ops = rank_variables(
            residual, expr, free, sha)
    navigation_ns = monotonic_ns() - t1

    t2 = monotonic_ns()
    if inconsistent:
        count, witness, status, nodes, literals = 0, None, "EXACT_UNSAT", 0, 0
    else:
        count, witness, status, nodes, literals = bounded_count(
            clauses, residual, expr, order)
    search_ns = monotonic_ns() - t2

    t3 = monotonic_ns()
    assert engine.source_fingerprint(clauses) == sha
    if count is not None and witness is not None:
        assert engine.clauses_hold(clauses, witness)
    verification_ns = monotonic_ns() - t3

    trace = (sha, proof, tuple(order),
             tuple((v, target, x, result_sha, cover)
                   for v, target, x, result_sha, cover, _ in digit_receipts),
             status, count)
    return dict(method="RIME_PH_BSGS_CRT", kind=kind, variables=n,
                source_sha256=sha, proof_sha256=proof, rank=independent_rank,
                recognized=len(rows), residual=len(residual), free=len(free),
                corrected_proposals=corrected, variable_order=order,
                rime_digit_receipts=digit_receipts,
                rime_navigation_ops=digit_ops, status=status,
                model_count=count if count is not None else "HOLD",
                witness_integer=witness if witness is not None else "NONE",
                witness_sha256=engine.sha(witness),
                search_nodes=nodes, search_literal_checks=literals,
                preprocess_ns=preprocess_ns, navigation_ns=navigation_ns,
                search_ns=search_ns, verification_ns=verification_ns,
                metered_units=len(repr(clauses).encode("ascii")) +
                sum(meter.values()) + digit_ops + nodes + literals,
                trace_sha256=engine.sha(trace))


def incumbent(kind, n, clauses):
    result = engine.execute(kind, n, clauses, limit=20000, injected=True)
    # Do not alter source file or normalize away its legitimate baseline cost.
    return dict(method="YANG_MILLS_FROZEN", kind=kind, variables=n,
                source_sha256=result["source_sha256"],
                proof_sha256=result["proof_sha256"], rank=result["rank"],
                recognized=result["recognized"], residual=result["residual"],
                free=result["free_variables"], status=result["status"],
                model_count=result["model_count"],
                witness_integer=result["witness_integer"],
                witness_sha256=result["witness_sha256"],
                search_nodes=result["search_nodes"],
                fixed_comparison_nodes=result["baseline_nodes"],
                selected_order=result["selected_order"],
                solver_elapsed_ns=result["elapsed_ns"],
                metered_units=result["total_metered_units"],
                trace_sha256=result["result_sha256"])


def main():
    assert len(sys.argv) == 5
    method, kind, n_text, seed = sys.argv[1:]
    n = int(n_text)
    assert (kind, n, seed) in tuple(engine.heldout_cases())
    started = monotonic_ns()
    source = engine.heldout_source(kind, n, seed)
    original_source_ns = monotonic_ns() - started
    engine.validate_source(n, source)
    assert engine.parse_dimacs(engine.dimacs(n, source)) == (n, source)
    if method == "legacy":
        result = incumbent(kind, n, source)
    else:
        assert method == "rime"
        result = rime_challenger(kind, n, source)
    result["seed"] = seed
    result["source_generation_ns"] = original_source_ns
    result["total_process_ns"] = monotonic_ns() - started
    assert result["source_sha256"] == engine.source_fingerprint(source)
    print("PNP_MATCH|" + repr(result) + "|json=0", flush=True)


if __name__ == "__main__":
    main()
