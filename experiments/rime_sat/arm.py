#!/usr/bin/env python3
"""Source-frozen seven SAT solvers on signed RIME-wound generic CNFs.

Three old research contenders use their original ordering code through
the SAME existing watched-literal DPLL adapter. HXF-1 uses its existing
structural recognizers and original fallback. All receive ONLY the CNF.
The caller checks every SAT output after independently unwinding the model.
"""
import ast
import sys
from pathlib import Path
from time import monotonic_ns

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import transform

ARMS = ("hxf4", "ym_fischer", "rime_new", "rime_original")
LIMIT = 2000


def run(arm, source_path, mapping_path, description_path, model_path):
    assert arm in ARMS
    started = monotonic_ns()
    n, clauses = transform.parse(Path(source_path).read_bytes())
    mapping = tuple(int(x) for x in Path(mapping_path).read_text(
        encoding="ascii").split())
    assert len(mapping) == n + 1
    description = ast.literal_eval(
        Path(description_path).read_text(encoding="ascii"))
    focus = []
    if description.get("kind") == "bitcoin":
        manifest = description["original_bitcoin_manifest"]
        focus = [abs(mapping[v]) for bit, v in
                 manifest["nonce_variables"]]
        focus.extend(abs(mapping[v]) for bit, v in
                     manifest["merkle_variables"])
        assert len(focus) == manifest["free_bits_total"]
    else:
        focus = list(range(1, n + 1))
    preprocess_ns = monotonic_ns() - started
    if arm == "hxf4":
        sys.path.insert(0, str(ROOT.joinpath("experiments", "fourth")))
        import engine as hxf
        outcome = hxf.run(n, clauses, cap=LIMIT)
        status = outcome["status"]
        search_nodes = outcome["search_nodes"]
        checks = outcome["literal_checks"]
        method = outcome["proof_kind"]
        integer = outcome["witness_integer"]
        true_vars = ([v for v in range(1, n + 1)
                      if (integer >> (v - 1)) & 1]
                     if isinstance(integer, int) else None)
        nav_ns = 0
        nav_ops = 0
    else:
        sys.path.insert(0, str(ROOT.joinpath("experiments", "bitcoin")))
        import arm as historical_adapter
        import watched
        hints = tuple((i, variable - 1 + 1)
                      for i, variable in enumerate(focus))
        order, nav_ops, nav_ns = historical_adapter.get_order(
            arm, clauses, n, hints)
        outcome = watched.solve(n, clauses, priority=order,
                                limit=LIMIT)
        status = outcome["status"]
        search_nodes = outcome["nodes"]
        checks = outcome["literal_checks"]
        method = "ORIGINAL_FISCHER_ORDER_WITH_COMMON_WATCHED"
        true_vars = outcome["witness"]
    if status == "SAT":
        assert true_vars is not None
        Path(model_path).write_text(
            " ".join(str(v) for v in true_vars) + "\n",
            encoding="ascii")
    else:
        assert status in ("HOLD_BUDGET", "UNSAT_HALL",
                          "UNSAT_XOR", "UNSAT_DPLL")
        Path(model_path).write_text("", encoding="ascii")
    report = dict(arm=arm, status=status, n=n,
                  clauses=len(clauses), model_written=status == "SAT",
                  search_nodes=search_nodes, literal_checks=checks,
                  method=method, navigation_ns=nav_ns,
                  navigation_ops=nav_ops, parse_ns=preprocess_ns,
                  total_ns=monotonic_ns() - started,
                  limit_nodes=LIMIT)
    print("RIME_SAT_ARM|" + repr(report) + "|json=0", flush=True)


if __name__ == "__main__":
    assert len(sys.argv) == 6
    run(*sys.argv[1:])
