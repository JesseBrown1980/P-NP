#!/usr/bin/env python3
"""One original-source SAT solve by an HXF-2-style bounded fusion adapter.

All 15 arms use the same HXF-1 Hall/XOR exact fast path followed by the
same source-pinned watched-literal DPLL kernel. They differ ONLY in
source-derived branch hints: baseline / frozen MTP / Fischer / nested watcher
or ablated combinations. Timings include the frozen/coordination work.
"""
import ast
import hashlib
import sys
from pathlib import Path
from time import monotonic_ns

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT.joinpath("experiments","fourth")))
import engine as hxf
sys.path.insert(0,str(ROOT.joinpath("experiments","bitcoin")))
import watched
import hybrid

NODE_LIMIT=2000


def run(arm,source_path,model_path):
    assert arm in hybrid.ARMS
    started=monotonic_ns()
    raw=Path(source_path).read_bytes()
    input_sha=hashlib.sha256(raw).hexdigest()
    n,clauses=hxf.parse(raw.decode("ascii"))
    before=monotonic_ns()
    order,prep=hybrid.order_for_source(n,clauses,input_sha,arm)
    analysis_ns=monotonic_ns()-before
    structural_ns=0
    solver_ns=0
    witness="NONE"
    solved=hxf.hall(n,clauses)
    if solved is None:
        solved=hxf.xor(n,clauses)
    structural_ns=monotonic_ns()-before-analysis_ns
    if solved is not None:
        status=solved["status"]
        proof_kind=solved["proof_kind"]
        nodes=0
        checks=0
        if status=="SAT":
            bits=solved["witness"]
            assert hxf.holds(clauses,bits)
            witness=bits
    else:
        before_solve=monotonic_ns()
        result=watched.solve(n,clauses,priority=order,limit=NODE_LIMIT)
        solver_ns=monotonic_ns()-before_solve
        status=result["status"]
        proof_kind="COMMON_WATCHED_DPLL"
        nodes=result["nodes"]
        checks=result["literal_checks"]
        if status=="SAT":
            variables=set(result["witness"])
            witness=sum(1 << (v-1) for v in variables)
            assert hxf.holds(clauses,witness)
    if status=="SAT":
        encoded=witness.to_bytes((n+7)//8,"little")
        Path(model_path).write_bytes(encoded)
        model_sha=hashlib.sha256(encoded).hexdigest()
    else:
        assert status in ("UNSAT_HALL","UNSAT_XOR",
                          "UNSAT_DPLL","HOLD_BUDGET")
        model_sha="NONE"
    report=dict(arm=arm,variables=n,clauses=len(clauses),
                input_sha256=input_sha,status=status,
                proof_kind=proof_kind,nodes=nodes,
                literal_checks=checks,
                node_cap=NODE_LIMIT,preprocess=prep,
                analysis_ns=analysis_ns,structural_ns=structural_ns,
                solver_ns=solver_ns,total_ns=monotonic_ns()-started,
                model_sha256=model_sha)
    print("MTP_NNEST_RESULT|"+repr(report)+"|json=0",flush=True)


if __name__=="__main__":
    assert len(sys.argv)==4
    run(*sys.argv[1:])
