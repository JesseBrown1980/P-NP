#!/usr/bin/env python3
"""22-arm exact SAT adapter with no artificial per-solve search-node cap.

15 CNF-native MTP/N-Nest/Fischer guidance arms + HXF-1 + original three
Fischer order adapters. Established MiniSat/CaDiCaL/PicoSAT run directly
outside this script.

All research arms receive the SAME original CNF. No source generator,
expected answer, witness, nonce, case name or outcome is passed to this
adapter. Complete SAT models are serialized as original-CNF assignment bits.
"""
import hashlib
import sys
from pathlib import Path
from time import monotonic_ns

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT.joinpath("experiments","mtp_nnest")))
import hybrid
sys.path.insert(0,str(ROOT.joinpath("experiments","fourth")))
import engine as hxf
# Isolate both archived modules named "engine": keep HXF object bound while
# replacing the module registry name with the original sat/engine source.
# bitcoin.arm subsequently imports this original engine by its canonical name.
sys.modules.pop("engine",None)
sys.path.insert(0,str(ROOT.joinpath("sat")))
import engine as original_sat_engine
assert hasattr(original_sat_engine,"source_fingerprint")
sys.path.insert(0,str(ROOT.joinpath("experiments","bitcoin")))
import watched

RESEARCH=("hxf4","ym_fischer","rime_new","rime_original")+hybrid.ARMS
UNRESTRICTED_NODE_CAP=(1<<63)-1


def source_hints(n,clauses,sha,arm):
    candidates,rows=hybrid.candidates_from_source(n,clauses)
    assert len(candidates)<=81
    sys.path.insert(0,str(ROOT.joinpath("experiments","bitcoin")))
    import arm as original_adapter
    pre=monotonic_ns()
    order,operations,nav_ns=original_adapter.get_order(
        arm,clauses,n,tuple((i,v) for i,v in enumerate(candidates)))
    assert set(order)==set(candidates)
    return order,dict(prep_ns=monotonic_ns()-pre,
                      source_hint_count=len(order),
                      fischer_operations=operations,
                      navigation_ns=nav_ns)


def main(arm,original_path,model_path):
    assert arm in RESEARCH
    began=monotonic_ns()
    original=Path(original_path).read_bytes()
    sha=hashlib.sha256(original).hexdigest()
    n,clauses=hxf.parse(original.decode("ascii"))
    assert n>0 and len(clauses)>0
    prep_before=monotonic_ns()
    if arm=="hxf4":
        order=()
        prep=dict(prep_ns=0,source_hint_count=0,
                  family="HXF_ORIGINAL")
    elif arm in hybrid.ARMS:
        order,prep=hybrid.order_for_source(n,clauses,sha,arm)
    else:
        order,prep=source_hints(n,clauses,sha,arm)
    prep_ns=monotonic_ns()-prep_before
    search_before=monotonic_ns()
    if arm=="hxf4":
        result=hxf.run(n,clauses,cap=UNRESTRICTED_NODE_CAP)
        status=result["status"]
        nodes=result["search_nodes"]
        checks=result["literal_checks"]
        family=result["family"]
        witness=result["witness_integer"]
        proof_kind=result["proof_kind"]
        if isinstance(witness,int):
            bits=witness
        else:
            bits=None
    else:
        structural=hxf.hall(n,clauses)
        if structural is None:
            structural=hxf.xor(n,clauses)
        if structural is not None:
            status=structural["status"]
            proof_kind=structural["proof_kind"]
            nodes=checks=0
            bits=structural["witness"]
            family="STRUCTURAL"
        else:
            result=watched.solve(
                n,clauses,priority=order,
                limit=UNRESTRICTED_NODE_CAP)
            status=result["status"]
            proof_kind="COMMON_WATCHED_DPLL_NO_NODE_CAP"
            nodes=result["nodes"]
            checks=result["literal_checks"]
            bits=(sum(1<<(v-1) for v in result["witness"])
                  if result["witness"] is not None else None)
            family="GENERAL_SAT"
    solve_ns=monotonic_ns()-search_before
    assert status in ("SAT","UNSAT_HALL","UNSAT_XOR","UNSAT_DPLL")
    witness_sha="NONE"
    if status=="SAT":
        assert isinstance(bits,int) and 0<=bits<(1<<n)
        assert hxf.holds(clauses,bits)
        packed=bits.to_bytes((n+7)//8,"little")
        Path(model_path).write_bytes(packed)
        witness_sha=hashlib.sha256(packed).hexdigest()
    else:
        assert bits is None
    receipt=dict(
        arm=arm,variables=n,clauses=len(clauses),
        original_sha256=sha,status=status,
        search_nodes=nodes,literal_checks=checks,
        family=family,proof_kind=proof_kind,
        precompute=prep,
        prepare_ns=prep_ns,
        solve_ns=solve_ns,total_ns=monotonic_ns()-began,
        node_limit="NONE",
        original_witness_sha256=witness_sha)
    print("COMPLEXITY_SOLVER|"+repr(receipt)+"|json=0",flush=True)


if __name__=="__main__":
    assert len(sys.argv)==4
    main(*sys.argv[1:])
