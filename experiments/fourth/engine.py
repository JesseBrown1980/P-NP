#!/usr/bin/env python3
"""Fourth contender: source-blind structural certificates, plus bounded DPLL.

HALL: reconstruct independent positive requirements and mutually exclusive
negative cliques from the actual CNF. Prove UNSAT with an explicit Hall
deficiency, or construct a directly checked satisfying assignment.
XOR: reconstruct truth tables from 3-CNF clause groups; derive a GF(2)
contradiction with independently replayable row-combination indices,
or construct a satisfying assignment for a consistent affine system.
Fallback: original-CNF bounded exact unit-propagating DPLL.

No instance name, class label, expected outcome or generator is consulted.
Original three competing engines remain unchanged.
"""
import hashlib
import sys
from pathlib import Path
from time import monotonic_ns


def digest(clauses):
    return hashlib.sha256(repr(clauses).encode("ascii")).hexdigest()


def holds(clauses, bits):
    return all(any(bool(bits & (1 << (abs(v) - 1))) == (v > 0)
                   for v in clause) for clause in clauses)


def parse(raw):
    n = None
    announced = None
    clauses, pending = [], []
    for line in raw.splitlines():
        cells = line.split()
        if not cells or cells[0] == "c":
            continue
        if cells[0] == "p":
            assert n is None and len(cells) == 4 and cells[1] == "cnf"
            n, announced = int(cells[2]), int(cells[3])
            continue
        assert n is not None
        for cell in cells:
            v = int(cell)
            if v == 0:
                assert pending and len(set(map(abs, pending))) == len(pending)
                clauses.append(tuple(pending))
                pending = []
            else:
                assert 1 <= abs(v) <= n
                pending.append(v)
    assert n and not pending and announced == len(clauses)
    return n, tuple(clauses)


def hall(variables, clauses):
    """Use an independently checkable clique partition and Hall witness.

    A positive clause is a requirement of at least one item from a group.
    A negative binary clause forbids simultaneous items from two groups.
    Every negative connected component must be a complete clique, giving
    a set of mutually exclusive slots. Matching picks one item per group
    from a different slot. If there is no full matching, Hall's theorem
    yields an explicit group subset with fewer neighboring slots.
    """
    positive = []
    negative = []
    membership = {}
    used = set()
    for clause in clauses:
        if len(clause) >= 2 and all(v > 0 for v in clause):
            index = len(positive)
            positive.append(tuple(sorted(clause)))
            for v in clause:
                if v in membership:
                    return None
                membership[v] = index
                used.add(v)
        elif len(clause) == 2 and all(v < 0 for v in clause):
            a, b = sorted((-clause[0], -clause[1]))
            negative.append((a, b))
            used.update((a, b))
        else:
            return None
    if not positive or not negative or len(used) != variables:
        return None
    if set(membership) != used or len(set(negative)) != len(negative):
        return None
    parent = {v: v for v in used}

    def root(v):
        while parent[v] != v:
            parent[v] = parent[parent[v]]
            v = parent[v]
        return v

    for a, b in negative:
        if membership[a] == membership[b]:
            return None
        ra, rb = root(a), root(b)
        if ra != rb:
            parent[max(ra, rb)] = min(ra, rb)
    comp = {}
    for v in sorted(used):
        comp.setdefault(root(v), []).append(v)
    slots = tuple(tuple(members) for _, members in sorted(comp.items()))
    slot_of = {v: j for j, slot in enumerate(slots) for v in slot}
    all_edges = set(negative)
    for slot in slots:
        for j, a in enumerate(slot):
            for b in slot[j + 1:]:
                if (a, b) not in all_edges:
                    return None
    if len(all_edges) != sum(len(c) * (len(c) - 1) // 2 for c in slots):
        return None
    neighbors = tuple(tuple(sorted({slot_of[v] for v in req}))
                      for req in positive)
    match_slot = {}
    match_group = {}

    def augment(group, seen):
        for slot in neighbors[group]:
            if slot in seen:
                continue
            seen.add(slot)
            previous = match_slot.get(slot)
            if previous is None or augment(previous, seen):
                match_slot[slot] = group
                match_group[group] = slot
                return True
        return False

    for group in range(len(positive)):
        augment(group, set())
    if len(match_group) == len(positive):
        selected = []
        for group, slot in sorted(match_group.items()):
            candidates = sorted(v for v in positive[group]
                                if slot_of[v] == slot)
            assert candidates
            selected.append(candidates[0])
        witness = sum(1 << (v - 1) for v in selected)
        assert holds(clauses, witness)
        return dict(status="SAT", witness=witness,
                    proof_kind="HALL_MATCHED_MODEL",
                    hall_groups=positive, hall_slots=slots,
                    hall_subset=(), hall_neighborhood=(),
                    hall_matching=tuple(sorted(match_group.items())),
                    matching_size=len(match_group))
    # Alternating reachability from all unmatched left vertices.
    reached_groups = set(range(len(positive))) - set(match_group)
    todo = list(sorted(reached_groups))
    reached_slots = set()
    while todo:
        group = todo.pop()
        for slot in neighbors[group]:
            if match_group.get(group) == slot:
                continue
            if slot not in reached_slots:
                reached_slots.add(slot)
                previous = match_slot.get(slot)
                if previous is not None and previous not in reached_groups:
                    reached_groups.add(previous)
                    todo.append(previous)
    actual_neighbors = {slot for g in reached_groups
                        for slot in neighbors[g]}
    assert len(actual_neighbors) < len(reached_groups)
    assert actual_neighbors == reached_slots
    return dict(status="UNSAT_HALL", witness=None,
                proof_kind="HALL_DEFICIENT_SUBSET",
                hall_groups=positive, hall_slots=slots,
                hall_subset=tuple(sorted(reached_groups)),
                hall_neighborhood=tuple(sorted(actual_neighbors)),
                hall_matching=tuple(sorted(match_group.items())),
                matching_size=len(match_group))


def xor(variables, clauses):
    """Check every local 3-CNF truth table before recognizing its parity."""
    groups = {}
    for clause in clauses:
        if len(clause) != 3 or len(set(map(abs, clause))) != 3:
            return None
        scope = tuple(sorted(map(abs, clause)))
        groups.setdefault(scope, []).append(clause)
    rows = []
    for scope, parts in sorted(groups.items()):
        if len(parts) != 4:
            return None
        permitted = []
        for bits in range(8):
            truth = {v: bool((bits >> i) & 1)
                     for i, v in enumerate(scope)}
            if all(any(truth[abs(v)] == (v > 0)
                       for v in clause) for clause in parts):
                permitted.append(bits)
        assert len(permitted) <= 8
        if len(permitted) != 4:
            return None
        parity = {bits.bit_count() & 1 for bits in permitted}
        if len(parity) != 1:
            return None
        rhs = next(iter(parity))
        mask = sum(1 << (v - 1) for v in scope)
        rows.append((mask, rhs))
    basis = {}
    for index, (mask, rhs) in enumerate(rows):
        provenance = 1 << index
        while mask:
            pivot = mask.bit_length() - 1
            if pivot not in basis:
                basis[pivot] = (mask, rhs, provenance)
                break
            old_mask, old_rhs, old_provenance = basis[pivot]
            mask ^= old_mask
            rhs ^= old_rhs
            provenance ^= old_provenance
        else:
            if rhs:
                combined = tuple(i for i in range(len(rows))
                                 if (provenance >> i) & 1)
                assert combined
                return dict(status="UNSAT_XOR", witness=None,
                            proof_kind="EXPLICIT_XOR_COMBINATION",
                            xor_rows=tuple(rows), xor_subset=combined,
                            xor_rank=len(basis))
    # Solve in ascending pivots with all free bits zero.
    witness = 0
    for pivot in sorted(basis):
        mask, rhs, _ = basis[pivot]
        rest = mask & ~(1 << pivot)
        val = rhs ^ ((rest & witness).bit_count() & 1)
        if val:
            witness |= 1 << pivot
    assert witness < (1 << variables) and holds(clauses, witness)
    return dict(status="SAT", witness=witness,
                proof_kind="EXACT_XOR_MODEL",
                xor_rows=tuple(rows), xor_subset=(),
                xor_rank=len(basis))


class Budget(Exception):
    pass


def generic(n, clauses, limit):
    """Sound exact decision fallback; incomplete only at explicit node cap."""
    degree = [0] * n
    for clause in clauses:
        for v in clause:
            degree[abs(v) - 1] += 1
    order = sorted(range(n), key=lambda v: (-degree[v], v))
    nodes = [0]
    checks = [0]
    sys.setrecursionlimit(max(2000, n * 4 + 100))

    def visit(initial):
        if nodes[0] >= limit:
            raise Budget
        nodes[0] += 1
        state = dict(initial)
        while True:
            change = False
            satisfied_all = True
            for clause in clauses:
                remaining = []
                satisfied = False
                for lit in clause:
                    checks[0] += 1
                    val = state.get(abs(lit) - 1)
                    if val is None:
                        remaining.append(lit)
                    elif val == (lit > 0):
                        satisfied = True
                        break
                if satisfied:
                    continue
                if not remaining:
                    return None
                satisfied_all = False
                if len(remaining) == 1:
                    lit = remaining[0]
                    var = abs(lit) - 1
                    val = lit > 0
                    if var in state and state[var] != val:
                        return None
                    if var not in state:
                        state[var] = val
                        change = True
            if satisfied_all:
                return state
            if not change:
                break
        var = next(v for v in order if v not in state)
        for value in (False, True):
            child = dict(state)
            child[var] = value
            result = visit(child)
            if result is not None:
                return result
        return None

    try:
        result = visit({})
    except Budget:
        return dict(status="HOLD_BUDGET", witness=None,
                    proof_kind="GENERIC_BOUNDED_DPLL",
                    search_nodes=nodes[0], literal_checks=checks[0])
    witness = (sum(1 << v for v, b in result.items() if b)
               if result is not None else None)
    assert witness is None or holds(clauses, witness)
    return dict(status="SAT" if result is not None else "UNSAT_DPLL",
                witness=witness, proof_kind="GENERIC_BOUNDED_DPLL",
                search_nodes=nodes[0], literal_checks=checks[0])


def run(n, clauses, cap=2000):
    start = monotonic_ns()
    assert isinstance(n, int) and n > 0
    assert all(isinstance(c, tuple) and len(c) > 0
               and len(set(map(abs, c))) == len(c)
               and all(1 <= abs(v) <= n for v in c) for c in clauses)
    source = digest(clauses)
    preprocess = monotonic_ns()
    solution = hall(n, clauses)
    if solution is None:
        solution = xor(n, clauses)
    prep_ns = monotonic_ns() - preprocess
    if solution is None:
        outcome = generic(n, clauses, cap)
        method = "GENERIC"
    else:
        outcome = solution
        method = "HALL" if "hall" in outcome["proof_kind"].lower() else "XOR"
    witness = outcome["witness"]
    assert witness is None or holds(clauses, witness)
    assert source == digest(clauses)
    receipt = dict(method="FOURTH_STRUCTURAL_ENGINE", family=method,
                   variables=n, clauses=len(clauses), source_sha256=source,
                   status=outcome["status"], proof_kind=outcome["proof_kind"],
                   witness_integer=witness if witness is not None else "NONE",
                   search_nodes=outcome.get("search_nodes", 0),
                   literal_checks=outcome.get("literal_checks", 0),
                   preprocess_ns=prep_ns,
                   total_ns=monotonic_ns() - start, node_cap=cap)
    for key in ("hall_groups", "hall_slots", "hall_subset",
                "hall_neighborhood", "hall_matching", "matching_size",
                "xor_rows", "xor_subset", "xor_rank"):
        if key in outcome:
            receipt[key] = outcome[key]
    return receipt


def main():
    assert len(sys.argv) == 2
    raw = Path(sys.argv[1]).read_text(encoding="ascii")
    n, clauses = parse(raw)
    result = run(n, clauses)
    print("FOURTH_RESULT|" + repr(result) + "|json=0", flush=True)


if __name__ == "__main__":
    main()
