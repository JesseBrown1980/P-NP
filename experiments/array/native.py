#!/usr/bin/env python3
"""Third-party test matrix wrappers for original, new and historical Fischer.

Original Yang--Mills engine and historical Rime bytes are never modified.
Each arm is a separate process, preventing collisions between old/new
rime_fischer module names. On PHP, all three use THE SAME exact
unit-propagating DPLL decision kernel, but different source-pinned orders.
On Tseitin, all share original independently watched XOR recognition;
historical PH performs an isolated, checked group navigation only.
"""
import hashlib
import resource
import sys
from pathlib import Path
from time import monotonic_ns

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "experiments" / "hardest"))
sys.path.insert(0, str(ROOT / "sat"))
import challenge
import native_arm
import engine
import suite


def source_identity(n, clauses, raw):
    assert challenge.parse(raw) == (n, clauses)
    return hashlib.sha256(raw.encode("ascii")).hexdigest()


def historic_order(n, clauses, sha):
    sys.path.insert(0, str(ROOT / "historical" /
                           "asolaria-2026-07-21"))
    import rime_fischer
    import rime_sphere
    p = 97
    g = rime_sphere.primitive_root(p)
    started = monotonic_ns()
    rows = []
    movements = 0
    for v in range(n):
        raw = hashlib.sha256((sha + ":" + str(v)).encode("ascii")).digest()
        target = 1 + int.from_bytes(raw[:8], "big") % (p - 1)
        exponent, towers = rime_fischer.rime_fischer(g, target, p)
        assert pow(g, exponent, p) == target
        coverage = sum(int(any(abs(lit) == v + 1 for lit in clause))
                       for clause in clauses)
        movements += sum(cost for _, cost in towers)
        rows.append((-coverage, exponent, v))
    return tuple(row[-1] for row in sorted(rows)), movements, monotonic_ns() - started


def historic_probe(source_sha):
    sys.path.insert(0, str(ROOT / "historical" /
                           "asolaria-2026-07-21"))
    import rime_fischer
    import rime_sphere
    p = 97
    g = rime_sphere.primitive_root(p)
    target = 1 + int.from_bytes(hashlib.sha256(
        source_sha.encode("ascii")).digest()[:8], "big") % (p - 1)
    t = monotonic_ns()
    exponent, towers = rime_fischer.rime_fischer(g, target, p)
    assert pow(g, exponent, p) == target
    return sum(cost for _, cost in towers), monotonic_ns() - t


def run_native(arm, case, raw):
    n, clauses, expected = suite.definition(case)
    sha = source_identity(n, clauses, raw)
    started = monotonic_ns()
    parse_ns = monotonic_ns() - started
    preparation_t0 = monotonic_ns()
    nav_ns = 0
    nav_ops = 0
    literal_checks = 0
    search_nodes = 0
    search_ns = 0
    status = "HOLD"
    proof_sha = "NONE"
    witness = None
    proof_kind = "NONE"
    branch_used = False
    rank = 0

    if case.startswith("php"):
        expr = tuple((1 << v, 0) for v in range(n))
        free = tuple(range(n))
        if arm == "ym_fischer":
            meter = {"fischer_literal_scores": 0}
            order = engine.branch_order(free, clauses, expr, meter,
                                        "fischer")
            nav_ops = meter["fischer_literal_scores"]
        elif arm == "rime_new":
            sys.path.insert(0, str(ROOT / "algebra"))
            import competition
            nav_t0 = monotonic_ns()
            order, receipts, nav_ops = competition.rank_variables(
                clauses, expr, free, engine.source_fingerprint(clauses))
            nav_ns = monotonic_ns() - nav_t0
            assert len(receipts) == n
        else:
            assert arm == "rime_original"
            order, nav_ops, nav_ns = historic_order(
                n, clauses, engine.source_fingerprint(clauses))
        preparation_ns = monotonic_ns() - preparation_t0
        branch_used = True
        status, witness, search_nodes, literal_checks, search_ns = (
            native_arm.bounded_decision(n, clauses, order))
        proof_kind = "GENERAL_CNF_DPLL"
    else:
        engine.validate_source(n, clauses)
        if arm == "ym_fischer":
            rec = engine.execute(case, n, clauses, limit=2000,
                                 injected=True)
            assert rec["status"] == "EXACT_UNSAT"
            rank = rec["rank"]
            proof_sha = rec["proof_sha256"]
            literal_checks = rec["watcher_checks"]
            search_nodes = rec["search_nodes"]
            proof_kind = "YANG_MILLS_WATCHED_XOR"
        elif arm == "rime_new":
            sys.path.insert(0, str(ROOT / "algebra"))
            import competition
            rec = competition.rime_challenger(case, n, clauses)
            assert rec["status"] == "EXACT_UNSAT"
            rank = rec["rank"]
            proof_sha = rec["proof_sha256"]
            search_nodes = rec["search_nodes"]
            literal_checks = rec["search_literal_checks"]
            nav_ns = rec["navigation_ns"]
            proof_kind = "NEW_RIME_WITH_WATCHED_XOR"
        else:
            assert arm == "rime_original"
            nav_ops, nav_ns = historic_probe(
                engine.source_fingerprint(clauses))
            metric = {name: 0 for name in (
                "group_inserts", "local_literal_checks",
                "watcher_clause_checks", "watcher_rejections",
                "accepted_transforms", "fischer_row_scores",
                "pivot_probes", "row_xors", "xor_bitwork_bound",
                "basis_peak_bits", "watcher_pivot_probes",
                "watcher_row_xors", "watcher_bitwork_bound",
                "backsub_xors", "watcher_expression_checks",
                "fischer_literal_scores", "search_nodes",
                "search_literal_checks", "witness_literal_checks")}
            rows, residual, trace, corrected = engine.compile_source(
                n, clauses, metric, tamper_first=True)
            basis = engine.right_eliminate(n, rows, metric)
            rank, contradiction = engine.left_rank(n, rows, metric)
            assert contradiction and basis is None and not residual
            proof_sha = trace
            literal_checks = metric["watcher_clause_checks"]
            proof_kind = "HISTORICAL_RIME_WITH_WATCHED_XOR"
        status = "UNSAT_XOR"
        preparation_ns = monotonic_ns() - preparation_t0
    if witness is not None:
        assert native_arm.satisfies(clauses, witness)
    assert engine.source_fingerprint(clauses) == (
        engine.source_fingerprint(suite.definition(case)[1]))
    return dict(case=case, arm=arm, variables=n, clauses=len(clauses),
                expected=expected, input_sha256=sha,
                status=status, proof_kind=proof_kind,
                witness_integer=witness if witness is not None else "NONE",
                xor_proof_sha256=proof_sha, xor_rank=rank,
                branch_order_used=branch_used,
                navigation_ops=nav_ops, navigation_ns=nav_ns,
                search_nodes=search_nodes, literal_checks=literal_checks,
                preprocess_ns=preparation_ns, parse_ns=parse_ns,
                search_ns=search_ns, total_ns=monotonic_ns() - started,
                peak_rss_kib=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
                node_limit=2000)


def main():
    assert len(sys.argv) == 4
    arm, case, path = sys.argv[1:]
    assert arm in suite.ARMS[:3] and case in suite.CASES
    raw = Path(path).read_text(encoding="ascii")
    rec = run_native(arm, case, raw)
    print("SAT_ARRAY_NATIVE|" + repr(rec) + "|json=0", flush=True)


if __name__ == "__main__":
    main()
