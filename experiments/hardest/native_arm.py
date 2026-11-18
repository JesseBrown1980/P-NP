#!/usr/bin/env python3
"""General-CNF SAT adapters and exact 3-CNF algorithm for all research arms.

'ym_fischer' is byte-identical original engine.execute on 3-CNF, and the
original byte-identical engine.branch_order on general CNF with a common
bounded general-CNF DPLL decision kernel.

'rime_new' uses the verified newer Rime PH tie-break.
'rime_original' imports and executes the byte-identical historical ASOLARIA
rime_fischer.py as its discrete-log tie-break.
No history file or Yang--Mills source is changed. These are SAT *adapters*
on PHP, not claims that a discrete log solves SAT.
"""
import hashlib
import sys
from pathlib import Path
from time import monotonic_ns

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT.joinpath("sat")))
sys.path.insert(0, str(ROOT.joinpath("algebra")))
sys.path.insert(0, str(ROOT.joinpath("historical/asolaria-2026-07-21")))

import engine
import competition
import rime_fischer as historical_fischer
import rime_sphere as historical_sphere
import challenge


LIMIT = 2000
P = 97


class Budget(Exception):
    pass


def satisfies(clauses, bits):
    return all(any(((bits >> (abs(v) - 1)) & 1) == int(v > 0)
                   for v in clause) for clause in clauses)


def bounded_decision(n, clauses, order):
    """Exhaustive unit-propagating DPLL; no shortcut from fixture metadata."""
    assert len(order) == n and set(order) == set(range(n))
    started = monotonic_ns()
    nodes = [0]
    checked = [0]
    sys.setrecursionlimit(5000)

    def search(previous):
        if nodes[0] >= LIMIT:
            raise Budget
        nodes[0] += 1
        assign = dict(previous)
        while True:
            all_true = True
            forced = False
            for clause in clauses:
                true = False
                leftover = []
                for lit in clause:
                    checked[0] += 1
                    v = abs(lit) - 1
                    val = assign.get(v)
                    if val is None:
                        leftover.append(lit)
                    elif val == (lit > 0):
                        true = True
                        break
                if true:
                    continue
                if not leftover:
                    return None, False
                all_true = False
                if len(leftover) == 1:
                    lit = leftover[0]
                    v = abs(lit) - 1
                    val = lit > 0
                    old = assign.get(v)
                    if old is not None and old != val:
                        return None, False
                    if old is None:
                        assign[v] = val
                        forced = True
            if all_true:
                return assign, True
            if not forced:
                break
        v = next(v for v in order if v not in assign)
        for polarity in (False, True):
            current = dict(assign)
            current[v] = polarity
            witness, sat = search(current)
            if sat:
                return witness, True
        return None, False

    try:
        assignment, satisfiable = search({})
    except Budget:
        return "HOLD_BUDGET", None, nodes[0], checked[0], monotonic_ns() - started
    if satisfiable:
        bits = sum((1 << v) for v, val in assignment.items() if val)
        assert satisfies(clauses, bits)
        return "SAT", bits, nodes[0], checked[0], monotonic_ns() - started
    return "UNSAT_DPLL", None, nodes[0], checked[0], monotonic_ns() - started


def original_phi_rank(n, clauses, sha):
    """Historical byte-original PH per variable; same input and p as new."""
    g = historical_sphere.primitive_root(P)
    assert g == 5
    started = monotonic_ns()
    scores = []
    steps = 0
    for v in range(n):
        raw = hashlib.sha256((sha + ":" + str(v)).encode("ascii")).digest()
        target = 1 + int.from_bytes(raw[:8], "big") % (P - 1)
        exponent, towers = historical_fischer.rime_fischer(g, target, P)
        assert pow(g, exponent, P) == target
        coverage = sum(int(any(abs(lit) == v + 1 for lit in clause))
                       for clause in clauses)
        scores.append((-coverage, exponent, v))
        steps += sum(moves for _, moves in towers)
    return tuple(v for _, _, v in sorted(scores)), steps, monotonic_ns() - started


def run(case, arm, raw):
    started = monotonic_ns()
    n, clauses = challenge.parse(raw)
    assert (n, clauses) == challenge.original(case)
    parse_ns = monotonic_ns() - started
    source = engine.source_fingerprint(clauses)
    original_sha = hashlib.sha256(raw.encode("ascii")).hexdigest()
    navigation_ns = 0
    preprocessing_ns = 0
    navigation_ops = 0
    search_nodes = 0
    literal_checks = 0
    proof_kind = "NONE"
    witness = None
    status = "HOLD"
    exact_proof_sha = "NONE"
    rank = 0
    t0 = monotonic_ns()

    if case == "tseitin128":
        engine.validate_source(n, clauses)
        if arm == "ym_fischer":
            original = engine.execute("tseitin128", n, clauses,
                                      limit=LIMIT, injected=True)
            assert original["source_sha256"] == source
            assert original["status"] == "EXACT_UNSAT"
            status = "UNSAT_XOR"
            rank = original["rank"]
            exact_proof_sha = original["proof_sha256"]
            proof_kind = "ORIGINAL_YM_XOR"
            search_nodes = original["search_nodes"]
            literal_checks = original["watcher_checks"]
        elif arm == "rime_new":
            new = competition.rime_challenger("tseitin128", n, clauses)
            assert new["source_sha256"] == source
            assert new["status"] == "EXACT_UNSAT"
            status = "UNSAT_XOR"
            rank = new["rank"]
            exact_proof_sha = new["proof_sha256"]
            proof_kind = "NEW_RIME_WITH_YM_XOR"
            search_nodes = new["search_nodes"]
            literal_checks = new["search_literal_checks"]
            navigation_ns = new["navigation_ns"]
        else:
            assert arm == "rime_original"
            # Run original byte-identical discrete-log code on a frozen,
            # source-derived target even though XOR algebra already detects
            # contradiction before branch ordering is needed.
            g = historical_sphere.primitive_root(P)
            target = 1 + int.from_bytes(
                hashlib.sha256(source.encode("ascii")).digest()[:8],
                "big") % (P - 1)
            t = monotonic_ns()
            exponent, towers = historical_fischer.rime_fischer(g, target, P)
            navigation_ns = monotonic_ns() - t
            assert pow(g, exponent, P) == target
            navigation_ops = sum(moves for _, moves in towers)
            m = {key: 0 for key in competition.METRICS}
            rows, residual, proof, corrected = engine.compile_source(
                n, clauses, m, tamper_first=True)
            basis = engine.right_eliminate(n, rows, m)
            rank, conflict = engine.left_rank(n, rows, m)
            assert conflict and basis is None and not residual
            status = "UNSAT_XOR"
            exact_proof_sha = proof
            proof_kind = "HISTORICAL_RIME_WITH_YM_XOR"
            literal_checks = m["watcher_clause_checks"]
        preprocessing_ns = monotonic_ns() - t0
    else:
        assert case in ("php21", "php8", "php4sat")
        expr = tuple((1 << v, 0) for v in range(n))
        free = tuple(range(n))
        if arm == "ym_fischer":
            metric = {"fischer_literal_scores": 0}
            order = engine.branch_order(free, clauses, expr, metric, "fischer")
            navigation_ops = metric["fischer_literal_scores"]
        elif arm == "rime_new":
            t = monotonic_ns()
            order, receipts, navigation_ops = competition.rank_variables(
                clauses, expr, free, source)
            navigation_ns = monotonic_ns() - t
            assert len(receipts) == n
        else:
            assert arm == "rime_original"
            order, navigation_ops, navigation_ns = original_phi_rank(
                n, clauses, source)
        preprocessing_ns = monotonic_ns() - t0
        status, witness, search_nodes, literal_checks, search_ns = (
            bounded_decision(n, clauses, order))
        proof_kind = "GENERAL_CNF_DPLL"
    if case == "tseitin128":
        search_ns = 0
    tv = monotonic_ns()
    if witness is not None:
        assert satisfies(clauses, witness)
    verify_ns = monotonic_ns() - tv
    assert engine.source_fingerprint(clauses) == source
    receipt = dict(case=case, arm=arm, variables=n, clauses=len(clauses),
                   raw_sha256=original_sha, source_sha256=source,
                   status=status, proof_kind=proof_kind,
                   witness_integer=witness if witness is not None else "NONE",
                   witness_sha256=engine.sha(witness),
                   xor_proof_sha256=exact_proof_sha, xor_rank=rank,
                   search_nodes=search_nodes, search_literal_checks=literal_checks,
                   navigation_ops=navigation_ops, parse_ns=parse_ns,
                   preprocess_ns=preprocessing_ns, navigation_ns=navigation_ns,
                   search_ns=search_ns, certificate_ns=verify_ns,
                   total_ns=monotonic_ns() - started, node_cap=LIMIT)
    return receipt


if __name__ == "__main__":
    assert len(sys.argv) == 4
    arm, case, source_path = sys.argv[1:]
    assert arm in ("ym_fischer", "rime_new", "rime_original")
    assert case in ("php21", "php8", "tseitin128", "php4sat")
    raw = Path(source_path).read_text(encoding="ascii")
    receipt = run(case, arm, raw)
    print("HARD_NATIVE|" + repr(receipt) + "|json=0", flush=True)
