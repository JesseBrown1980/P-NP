#!/usr/bin/env python3
"""Run fourth HXF-1, frozen Yang--Mills, modern PH and historical PH on
identical Bitcoin SHA256d SAT CNFs, with source-separated processes.

All Fischer source files remain byte-identical. The three Fischer routes
are explicitly SAT-order ADAPTERS using a common new watched-DPLL kernel,
because their original research interfaces are not native SHA bitblast
solvers. Each receives the same nonce variable labels and search budget.
HXF-1 is run unchanged, with its ordinary general-CNF fallback.

Solutions are accepted only after checking the full original CNF and
independently recomputing the original Bitcoin block's double SHA256.
"""
import ast
import hashlib
import os
import sys
from pathlib import Path
from time import monotonic_ns

import genesis
import watched

ROOT = Path(__file__).resolve().parents[2]
ARMS = ("hxf4", "ym_fischer", "rime_new", "rime_original", "constellation")


def parse(raw):
    variables = announced = None
    clauses, waiting = [], []
    for line in raw.decode("ascii").splitlines():
        parts = line.split()
        if not parts or parts[0] == "c":
            continue
        if parts[0] == "p":
            assert variables is None and len(parts) == 4 and parts[1] == "cnf"
            variables, announced = int(parts[2]), int(parts[3])
            continue
        assert variables is not None
        for word in parts:
            value = int(word)
            if value:
                assert 1 <= abs(value) <= variables
                waiting.append(value)
            else:
                assert waiting and len(set(map(abs, waiting))) == len(waiting)
                clauses.append(tuple(waiting))
                waiting = []
    assert variables and len(clauses) == announced and not waiting
    return variables, tuple(clauses)


def sat_cnf(clauses, value):
    return all(any(bool((value >> (abs(lit) - 1)) & 1) == (lit > 0)
                   for lit in clause) for clause in clauses)


def nonce_from_assignment(assignment, nonce_bits):
    return sum((1 << index) for index, variable in nonce_bits
               if variable in assignment)


def get_order(arm, clauses, n, nonce_bits):
    indices = tuple(var - 1 for bit, var in nonce_bits)
    touched = set(var for bit, var in nonce_bits)
    subset = tuple(clause for clause in clauses
                   if any(abs(v) in touched for v in clause))
    expr = [(0, 0)] * n
    for index in indices:
        expr[index] = (1 << index, 0)
    sys.path.insert(0, str(ROOT / "sat"))
    import engine as original_engine
    sha = original_engine.source_fingerprint(clauses)
    started = monotonic_ns()
    nav_ops = 0
    if arm == "constellation":
        values_file = Path(os.environ["PNP_RIME_VALUES"])
        lines = values_file.read_text(encoding="ascii").splitlines()
        assert lines[0] == (
            "PNP_RIME_VALUES|schema=v1|tensors=24|glyphs=27|"
            "source_sha256=9fe92da2b064e39ad52b389fb92d0c59739e1fd7d9db5b82c6d25c0ebf8fa735"
        )
        assert len(lines) == 25
        tensors = []
        for index, line in enumerate(lines[1:]):
            parts = line.split("|")
            assert parts[:2] == ["TENSOR", str(index)]
            assert len(parts) == 29
            tensor = tuple(map(int, parts[2:]))
            assert all(0 <= v < 1000081 for v in tensor)
            tensors.append(tensor)
        # Source-independent GGUF coordinate guide, identical source CNF
        # and shared watched DPLL kernel to the historical Fischer arms.
        # The published Rime tensors are not SAT-trained weights.
        counts = {}
        for clause in subset:
            for lit in clause:
                if abs(lit) in touched:
                    counts[abs(lit)] = counts.get(abs(lit), 0) + 1
        ranked = []
        for bit, index in enumerate(indices):
            var = index + 1
            glyph = (bit + len(subset)) % 27
            tensor = (bit + len(clauses)) % 24
            score = counts.get(var, 0) * 2048 + tensors[tensor][glyph]
            ranked.append((-score, bit, var))
            nav_ops += 1
        result = tuple(var - 1 for _, _, var in sorted(ranked))
    elif arm == "ym_fischer":
        meter = {"fischer_literal_scores": 0}
        result = original_engine.branch_order(
            indices, subset, expr, meter, "fischer")
        nav_ops = meter["fischer_literal_scores"]
    elif arm == "rime_new":
        sys.path.insert(0, str(ROOT / "algebra"))
        import competition
        result, receipts, nav_ops = competition.rank_variables(
            subset, expr, indices, sha)
        assert len(receipts) == len(indices)
    else:
        assert arm == "rime_original"
        sys.path.insert(0, str(ROOT /
                               "historical/asolaria-2026-07-21"))
        import rime_fischer as historical
        import rime_sphere
        generator = rime_sphere.primitive_root(97)
        scores = []
        for v in indices:
            target = 1 + int.from_bytes(hashlib.sha256(
                (sha + ":" + str(v)).encode("ascii")).digest()[:8],
                "big") % 96
            exponent, towers = historical.rime_fischer(
                generator, target, 97)
            assert pow(generator, exponent, 97) == target
            coverage = sum(int(any(
                expr[abs(lit) - 1][0] & (1 << v)
                for lit in clause)) for clause in subset)
            scores.append((-coverage, exponent, v))
            nav_ops += sum(moves for _, moves in towers)
        result = tuple(v for _, _, v in sorted(scores))
    assert len(result) == len(indices) and set(result) == set(indices)
    return tuple(v + 1 for v in result), nav_ops, monotonic_ns() - started


def main(arm, case, cnf_file, description_file):
    assert arm in ARMS and case in genesis.CASES
    started = monotonic_ns()
    raw = Path(cnf_file).read_bytes()
    input_sha = hashlib.sha256(raw).hexdigest()
    manifest = ast.literal_eval(
        Path(description_file).read_text(encoding="ascii"))
    assert manifest["name"] == case
    assert input_sha == manifest["input_sha256"]
    n, clauses = parse(raw)
    assert n == manifest["variables"]
    assert len(clauses) == manifest["clauses"]
    nonce_bits = tuple(manifest["nonce_variables"])
    merkle_bits = tuple(manifest["merkle_variables"])
    free_bits = nonce_bits + tuple((32 + bit, var)
                                   for bit, var in merkle_bits)
    assert len(nonce_bits) == manifest["nonce_free_bits"]
    assert len(merkle_bits) == manifest["merkle_free_bits"]
    assert len(free_bits) == manifest["free_bits_total"]
    assert all(1 < v <= n for bit, v in free_bits)
    preprocess_ns = monotonic_ns() - started
    native_nav_ops = native_nav_ns = 0
    t = monotonic_ns()
    if arm == "hxf4":
        sys.path.insert(0, str(ROOT / "experiments" / "fourth"))
        import engine as fourth_engine
        actual = fourth_engine.run(n, clauses, cap=1024)
        status = actual["status"]
        nodes = actual["search_nodes"]
        proof_kind = actual["proof_kind"]
        checks = actual["literal_checks"]
        model = actual["witness_integer"]
        model_true = None
        if model != "NONE":
            model_true = tuple(v for v in range(1, n + 1)
                               if (model >> (v - 1)) & 1)
    else:
        order, native_nav_ops, native_nav_ns = get_order(
            arm, clauses, n, free_bits)
        node_cap = int(os.getenv("PNP_BITCOIN_NODE_CAP", "1024"))
        assert 1024 <= node_cap <= 100000
        # Permit a larger but still bounded source-blind Boolean search.
        # All comparisons use the same node budget and exact kernel.
        sys.setrecursionlimit(8192)
        actual = watched.solve(n, clauses, priority=order, limit=node_cap)
        status = actual["status"]
        nodes = actual["nodes"]
        checks = actual["literal_checks"]
        model_true = actual["witness"]
        proof_kind = "COMMON_WATCHED_DPLL_ADAPTER"
    solve_ns = monotonic_ns() - t
    nonce = "NONE"
    merkle_hex = "NONE"
    accepted_witness = "NONE"
    if status == "SAT":
        assert model_true is not None
        selected = set(model_true)
        nonce = nonce_from_assignment(selected, nonce_bits)
        known = genesis.GENESIS_NONCE
        free_mask = (1 << manifest["nonce_free_bits"]) - 1
        nonce = (known & ~free_mask) | nonce
        candidate = bytearray(
            genesis.HEADER_PREFIX + nonce.to_bytes(4, "little"))
        if merkle_bits:
            merkle = sum(1 << bit for bit, var in merkle_bits
                         if var in selected)
            candidate[36:68] = merkle.to_bytes(32, "little")
        merkle_hex = bytes(candidate[36:68]).hex()
        computed = hashlib.sha256(hashlib.sha256(
            candidate).digest()).digest()[::-1].hex()
        assert computed == genesis.GENESIS_HASH
        assert all(any((abs(lit) in selected) == (lit > 0)
                       for lit in clause) for clause in clauses)
        accepted_witness = hashlib.sha256(candidate).hexdigest()
    elif status in ("UNSAT_DPLL", "UNSAT_HALL", "UNSAT_XOR"):
        # Every construction has a known, independently validated SAT model.
        raise AssertionError("false UNSAT on actual Bitcoin genesis CNF")
    else:
        assert status in ("HOLD_BUDGET",)
    receipt = dict(arm=arm, case=case, cnf_sha256=input_sha,
                   variables=n, clauses=len(clauses),
                   nonce_free_bits=manifest["nonce_free_bits"],
                   status=status, proof_kind=proof_kind,
                   search_nodes=nodes, node_cap=(node_cap if arm != "hxf4" else 1024),
                   literal_checks=checks, phi_operations=native_nav_ops,
                   navigation_ns=native_nav_ns,
                   source_parse_ns=preprocess_ns,
                   solve_ns=solve_ns, total_ns=monotonic_ns() - started,
                   recovered_nonce=nonce,
                   recovered_merkle_raw_hex=merkle_hex,
                   header_witness_sha256=accepted_witness)
    print("BITCOIN_ARM|" + repr(receipt) + "|json=0", flush=True)


if __name__ == "__main__":
    assert len(sys.argv) == 5
    main(*sys.argv[1:])
