#!/usr/bin/env python3
"""Frozen hard SAT challenges: canonical PHP and odd-charge 3-regular Tseitin.

All cases are UNSAT by independent mathematical invariants:
  PHP(P,H), P>H: injection from P objects into H slots impossible;
  Tseitin: XOR of all local vertex equations has left side zero
           (every edge twice), right side odd.
These mathematical invariants are not substitutes for per-solver proof logs.
"""
import hashlib
import random
import sys
from pathlib import Path

SEED = 20261117


def source_sha(raw):
    return hashlib.sha256(raw.encode("ascii")).hexdigest()


def php(p, h):
    assert p in (h, h + 1)
    clauses = [tuple(i * h + j + 1 for j in range(h))
               for i in range(p)]
    clauses.extend((-(i * h + j + 1), -(k * h + j + 1))
                   for j in range(h)
                   for i in range(p)
                   for k in range(i + 1, p))
    assert len(clauses) == p + h * p * (p - 1) // 2
    return p * h, tuple(clauses)


def random_matchings(vertices=128):
    assert vertices > 4 and vertices % 2 == 0
    generator = random.Random(SEED)
    for attempt in range(1, 10001):
        edges = set()
        good = True
        for _ in range(3):
            permutation = list(range(vertices))
            generator.shuffle(permutation)
            proposed = [tuple(sorted((permutation[j], permutation[j + 1])))
                        for j in range(0, vertices, 2)]
            if any(edge in edges for edge in proposed):
                good = False
                break
            edges.update(proposed)
        if not good:
            continue
        assert len(edges) == (3 * vertices // 2)
        adjacency = [[] for _ in range(vertices)]
        for a, b in edges:
            adjacency[a].append(b)
            adjacency[b].append(a)
        assert all(len(part) == 3 for part in adjacency)
        reached = {0}
        frontier = [0]
        while frontier:
            for v in adjacency[frontier.pop()]:
                if v not in reached:
                    reached.add(v)
                    frontier.append(v)
        if len(reached) == vertices:
            return tuple(sorted(edges)), attempt
    raise AssertionError("no connected three-regular matching graph in 10k trials")


def tseitin(vertices=128):
    edges, attempts = random_matchings(vertices)
    incidence = [[] for _ in range(vertices)]
    for i, (a, b) in enumerate(edges, 1):
        incidence[a].append(i)
        incidence[b].append(i)
    assert all(len(row) == 3 for row in incidence)
    clauses = []
    for vertex, row in enumerate(incidence):
        row = tuple(sorted(row))
        charge = int(vertex == 0)
        for pattern in range(8):
            if pattern.bit_count() % 2 != charge:
                clauses.append(tuple(
                    -v if (pattern >> j) & 1 else v
                    for j, v in enumerate(row)))
    assert len(clauses) == 4 * vertices
    return len(edges), tuple(clauses), edges, attempts


def original(case):
    if case == "php21":
        variables, clauses = php(21, 20)
    elif case == "php8":
        variables, clauses = php(8, 7)
    elif case == "php4sat":
        variables, clauses = php(4, 4)
    else:
        assert case == "tseitin128"
        variables, clauses, _, _ = tseitin()
    return variables, clauses


def dimacs(case):
    n, clauses = original(case)
    lines = [
        "c P-NP hard comparison fixture v1 " + case,
        "p cnf " + str(n) + " " + str(len(clauses))
    ]
    lines.extend(" ".join(str(v) for v in c) + " 0" for c in clauses)
    return "\n".join(lines) + "\n"


def parse(raw):
    n, announced = None, None
    clauses = []
    pending = []
    for line in raw.splitlines():
        f = line.split()
        if not f or f[0] == "c":
            continue
        if f[0] == "p":
            assert n is None and len(f) == 4 and f[1] == "cnf"
            n, announced = int(f[2]), int(f[3])
            assert n > 0 and announced > 0
            continue
        assert n is not None
        for word in f:
            lit = int(word)
            if lit:
                assert abs(lit) <= n
                pending.append(lit)
            else:
                assert pending and len(set(map(abs, pending))) == len(pending)
                clauses.append(tuple(pending))
                pending = []
    assert n and announced == len(clauses) and not pending
    return n, tuple(clauses)


def build(out):
    path = Path(out)
    path.mkdir(parents=True, exist_ok=True)
    for case in ("php21", "php8", "tseitin128", "php4sat"):
        raw = dimacs(case)
        actual = parse(raw)
        assert actual == original(case)
        p = path / (case + ".cnf")
        p.write_text(raw, encoding="ascii")
        print("HARD_SOURCE|case=" + case + "|variables=" + str(actual[0]) +
              "|clauses=" + str(len(actual[1])) +
              "|sha256=" + source_sha(raw) + "|status=FROZEN", flush=True)
    _, c = php(21, 20)
    assert len(c) == 4221
    _, c = php(8, 7)
    assert len(c) == 204
    _, c = php(4, 4)
    assert len(c) == 28
    n, c, edges, attempts = tseitin()
    assert n == 192 and len(c) == 512 and len(edges) == 192
    print("HARD_GENERATOR|php21_pigeon_ALO=21|php21_hole_AMO=4200|"
          "php21_clauses=4221|tseitin_vertices=128|tseitin_edges=192|"
          "tseitin_clauses=512|tseitin_graph_attempts=" + str(attempts) +
          "|odd_charge_invariant=PASS|status=PASS")


if __name__ == "__main__":
    assert len(sys.argv) == 2
    build(sys.argv[1])
