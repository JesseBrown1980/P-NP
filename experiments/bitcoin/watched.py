#!/usr/bin/env python3
"""Common source-independent watched-literal exact CNF SAT decision kernel.

Shared by the three historical research ordering adapters for this Bitcoin
stress test. Original mathematical engines are never modified. This kernel
implements complete DPLL search with unit propagation; it is exact when
finished and returns HOLD whenever the declared node cap is reached.
"""
from time import monotonic_ns


class Budget(Exception):
    pass


def solve(n, clauses, priority=(), limit=1024):
    started = monotonic_ns()
    assert n > 0 and limit >= 1
    assert all(c and all(0 < abs(v) <= n for v in c)
               for c in clauses)
    order = tuple(dict.fromkeys(
        tuple(priority) + tuple(range(1, n + 1))))
    assert len(order) == n and set(order) == set(range(1, n + 1))
    value = [0] * (n + 1)
    trail = []
    watch0 = []
    watch1 = []
    watchers = {}
    nodes = 0
    checks = 0
    propagations = 0
    head = 0

    def litval(lit):
        val = value[abs(lit)]
        return val if lit > 0 else -val

    def enqueue(lit):
        nonlocal propagations
        var = abs(lit)
        expected = 1 if lit > 0 else -1
        actual = value[var]
        if actual:
            return actual == expected
        value[var] = expected
        trail.append(lit)
        propagations += 1
        return True

    for index, clause in enumerate(clauses):
        watch0.append(0)
        watch1.append(0 if len(clause) == 1 else 1)
        if len(clause) == 1:
            if not enqueue(clause[0]):
                return dict(status="UNSAT_DPLL", witness=None,
                            nodes=0, literal_checks=0,
                            propagations=propagations,
                            search_ns=monotonic_ns() - started)
        else:
            watchers.setdefault(clause[0], []).append(index)
            watchers.setdefault(clause[1], []).append(index)

    def propagate():
        nonlocal head, checks
        while head < len(trail):
            falsified = -trail[head]
            head += 1
            candidates = watchers.get(falsified)
            if candidates is None:
                continue
            i = 0
            while i < len(candidates):
                ci = candidates[i]
                clause = clauses[ci]
                j0, j1 = watch0[ci], watch1[ci]
                if clause[j0] == falsified:
                    found_index, opposite = 0, j1
                elif clause[j1] == falsified:
                    found_index, opposite = 1, j0
                else:
                    # Watchlists are maintained exactly even across backtracks.
                    raise AssertionError("stale watched literal")
                other = clause[opposite]
                checks += 1
                if litval(other) == 1:
                    i += 1
                    continue
                moved = False
                for j, lit in enumerate(clause):
                    if j == j0 or j == j1:
                        continue
                    checks += 1
                    if litval(lit) != -1:
                        if found_index == 0:
                            watch0[ci] = j
                        else:
                            watch1[ci] = j
                        candidates[i] = candidates[-1]
                        candidates.pop()
                        watchers.setdefault(lit, []).append(ci)
                        moved = True
                        break
                if moved:
                    continue
                if litval(other) == -1 or not enqueue(other):
                    return False
                i += 1
        return True

    if not propagate():
        return dict(status="UNSAT_DPLL", witness=None, nodes=0,
                    literal_checks=checks, propagations=propagations,
                    search_ns=monotonic_ns() - started)

    def backtrack(size):
        nonlocal head
        while len(trail) > size:
            lit = trail.pop()
            value[abs(lit)] = 0
        head = min(head, size)

    def recurse():
        nonlocal nodes
        if nodes >= limit:
            raise Budget
        nodes += 1
        decision_var = next((v for v in order if value[v] == 0), None)
        if decision_var is None:
            return True
        before = len(trail)
        for lit in (decision_var, -decision_var):
            if enqueue(lit) and propagate() and recurse():
                return True
            backtrack(before)
        return False

    try:
        sat = recurse()
    except Budget:
        return dict(status="HOLD_BUDGET", witness=None, nodes=nodes,
                    literal_checks=checks, propagations=propagations,
                    search_ns=monotonic_ns() - started)
    witness = None
    if sat:
        witness = tuple(v for v in range(1, n + 1) if value[v] == 1)
    return dict(status="SAT" if sat else "UNSAT_DPLL", witness=witness,
                nodes=nodes, literal_checks=checks,
                propagations=propagations,
                search_ns=monotonic_ns() - started)
