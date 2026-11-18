#!/usr/bin/env python3
"""Reversible RIME/Fischer source-blind SAT representation.

RIME transforms 27 source-derived bytes through the original 27-point
NTT (p=1000081, g=7, w=951846). Each resulting glyph is wound to
the original Fischer smooth sphere (p=103681, N=20736), unwound exactly,
and used to choose a signed bijection of each 27-variable block.
The paired random control permutes the same 27-variable block and
has EXACTLY THE SAME NUMBER of sign reversals per block.
No expected decision, satisfying assignment, Bitcoin nonce, or oracle
is used in choosing the transformations.

Mathematically this is a reversible SIGNED VARIABLE RELABELING of CNF.
It is a first controlled test of winding as a representation/ordering
heuristic, not a nonlinear RIME constraint solver.
"""
import hashlib
import random
import sys
from pathlib import Path
from time import monotonic_ns

P = 1000081
G = 7
N = 27
W = 951846
SPHERE_P = 103681
SPHERE_N = 20736
MODES = ("plain", "rime_wound", "random_gauge")


def factors(value):
    out = []
    d = 2
    while d * d <= value:
        if value % d == 0:
            out.append(d)
            while value % d == 0:
                value //= d
        d += 1
    if value > 1:
        out.append(value)
    return out


def primitive_root(p):
    qs = factors(p - 1)
    for root in range(2, p):
        if all(pow(root, (p - 1) // q, p) != 1 for q in qs):
            return root
    raise AssertionError("primitive root missing")


assert pow(G, (P - 1) // N, P) == W
assert pow(W, N, P) == 1 and all(pow(W, N // q, P) != 1
                                 for q in factors(N))
ROOT = primitive_root(SPHERE_P)
SPHERE_W = pow(ROOT, (SPHERE_P - 1) // SPHERE_N, SPHERE_P)
assert pow(SPHERE_W, SPHERE_N, SPHERE_P) == 1
assert all(pow(SPHERE_W, SPHERE_N // q, SPHERE_P) != 1
           for q in factors(SPHERE_N))
WEIGHTS = tuple(tuple(pow(W, (j * k) % N, P)
                      for j in range(N)) for k in range(N))
IW = pow(W, -1, P)
INVERSE_WEIGHTS = tuple(tuple(pow(IW, (j * k) % N, P)
                              for j in range(N)) for k in range(N))
INV27 = pow(N, -1, P)
SPHERE = tuple(pow(SPHERE_W, k, SPHERE_P) for k in range(SPHERE_N))
UNWIND = {v: k for k, v in enumerate(SPHERE)}
assert len(UNWIND) == SPHERE_N


def ntt(block):
    assert len(block) == N
    return tuple(sum(x * weight for x, weight in zip(block, row)) % P
                 for row in WEIGHTS)


def intt(spectrum):
    assert len(spectrum) == N
    return tuple(INV27 * (
        sum(x * weight for x, weight in zip(spectrum, row)) % P) % P
        for row in INVERSE_WEIGHTS)


def parse(raw):
    """Independent full DIMACS parsing, no solver imports."""
    variables = announced = None
    clauses = []
    buffer = []
    for line in raw.decode("ascii").splitlines():
        cells = line.split()
        if not cells or cells[0] == "c":
            continue
        if cells[0] == "p":
            assert variables is None and len(cells) == 4 and cells[1] == "cnf"
            variables, announced = int(cells[2]), int(cells[3])
            continue
        assert variables is not None
        for entry in cells:
            value = int(entry)
            if value == 0:
                assert buffer and len(set(map(abs, buffer))) == len(buffer)
                clauses.append(tuple(buffer))
                buffer = []
            else:
                assert 1 <= abs(value) <= variables
                buffer.append(value)
    assert variables and not buffer and len(clauses) == announced
    return variables, tuple(clauses)


def encode(variables, clauses):
    lines = ["p cnf " + str(variables) + " " + str(len(clauses))]
    lines.extend(" ".join(str(lit) for lit in clause) + " 0"
                 for clause in clauses)
    return ("\n".join(lines) + "\n").encode("ascii")


def signed_map(variables, original_sha, mode):
    assert mode in MODES and variables > 0 and len(original_sha) == 64
    if mode == "plain":
        return tuple(range(variables + 1)), 0
    result = [0] * (variables + 1)
    reverse_count = 0
    for start in range(0, variables, N):
        width = min(N, variables - start)
        group = start // N
        digest = hashlib.sha256(
            bytes.fromhex(original_sha) +
            group.to_bytes(8, "little")).digest()
        source = tuple(digest[:N])
        spectrum = ntt(source)
        assert intt(spectrum) == source
        points = []
        for k, glyph in enumerate(spectrum):
            address = (glyph + 256 * (k % 3) +
                       768 * (group % 27)) % SPHERE_N
            point = SPHERE[address]
            assert UNWIND[point] == address
            points.append(point)
        by_winding = tuple(sorted(range(width),
                                  key=lambda k: (points[k], spectrum[k], k)))
        if mode == "rime_wound":
            permutation = by_winding
            flipped = {k for k in range(width)
                       if (spectrum[k] ^ points[k]) & 1}
        else:
            rng = random.Random(int.from_bytes(digest, "big"))
            shuffled = list(range(width))
            rng.shuffle(shuffled)
            permutation = tuple(shuffled)
            same_number = sum(1 for k in range(width)
                              if (spectrum[k] ^ points[k]) & 1)
            flipped = set(rng.sample(range(width), same_number))
        # sorted indices carry original variable numbers; each variable
        # gets one distinct target variable within this 27-variable block.
        for target_local, original_local in enumerate(permutation):
            variable = start + original_local + 1
            target = start + target_local + 1
            result[variable] = -target if original_local in flipped else target
            reverse_count += int(original_local in flipped)
    assert sorted(abs(v) for v in result[1:]) == list(
        range(1, variables + 1))
    return tuple(result), reverse_count


def change(clauses, mapping):
    return tuple(tuple(mapping[abs(lit)] if lit > 0
                       else -mapping[abs(lit)]
                       for lit in clause) for clause in clauses)


def unchange(clauses, mapping):
    inverse = [0] * len(mapping)
    for old in range(1, len(mapping)):
        converted = mapping[old]
        inverse[abs(converted)] = old if converted > 0 else -old
    return change(clauses, inverse)


def transfer_witness(model, mapping):
    """Map a transformed true-variable set back into source variable bits."""
    original = 0
    for v in range(1, len(mapping)):
        m = mapping[v]
        bit = (abs(m) in model)
        if m < 0:
            bit = not bit
        if bit:
            original |= 1 << (v - 1)
    return original


def valid_model(clauses, bits):
    return all(any(((bits >> (abs(lit) - 1)) & 1) == int(lit > 0)
                   for lit in clause) for clause in clauses)


def make(original_path, mode, output_path, mapping_path):
    raw = Path(original_path).read_bytes()
    original_sha = hashlib.sha256(raw).hexdigest()
    variables, clauses = parse(raw)
    began = monotonic_ns()
    mapping, flips = signed_map(variables, original_sha, mode)
    transformed = change(clauses, mapping)
    assert unchange(transformed, mapping) == clauses
    out = encode(variables, transformed)
    Path(output_path).write_bytes(out)
    map_file = Path(mapping_path)
    map_file.write_text(" ".join(map(str, mapping)) + "\n",
                        encoding="ascii")
    payload = dict(variables=variables, clauses=len(clauses),
                   mode=mode, original_sha256=original_sha,
                   transformed_sha256=hashlib.sha256(out).hexdigest(),
                   map_sha256=hashlib.sha256(
                       map_file.read_bytes()).hexdigest(),
                   flips=flips,
                   rime_modulus=P, rime_root=W,
                   fischer_modulus=SPHERE_P,
                   fischer_tower=SPHERE_N,
                   elapsed_ns=monotonic_ns() - began,
                   inverse_clauses=len(clauses),
                   identity=transformed == clauses)
    print("RIME_SOURCE|" + repr(payload) + "|json=0", flush=True)
    return payload


def main():
    assert len(sys.argv) == 5
    make(*sys.argv[1:])


if __name__ == "__main__":
    main()
