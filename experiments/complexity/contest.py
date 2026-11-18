#!/usr/bin/env python3
"""Source-frozen 22-arm completion vs scaling SAT/Bitcoin competition.

COMPLETION: no per-problem time limit and no SAT node cap; exact SAT/UNSAT.
SCALING: 30/45/60-second *resource safeguards*, no SAT search-node cap.
GitHub job deadline is an additional finite machine-resource safeguard.
SAT outcomes are validated on every original clause and Bitcoin header.
Every timeout is explicitly censored as HOLD_RESOURCE.
"""
import ast
import hashlib
import os
import signal
import subprocess
import sys
from pathlib import Path
from time import monotonic_ns

import cases

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT.joinpath("experiments","array")))
import verify as original_verify
sys.path.insert(0,str(ROOT.joinpath("experiments","bitcoin")))
import genesis

EXTERNAL=("minisat","cadical","picosat")
RESEARCH=("hxf4","ym_fischer","rime_new","rime_original",
          "baseline","mtp","fischer",
          "nest3","nest9","nest27","nest81",
          "mtp_nest3","mtp_nest9","mtp_nest27","mtp_nest81",
          "all3","all9","all27","all81")
ARMS=RESEARCH+EXTERNAL


def run_process(command,deadline):
    began=monotonic_ns()
    process=subprocess.Popen(command,stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE,start_new_session=True)
    timed=False
    try:
        stdout,stderr=process.communicate(timeout=deadline)
        rc=process.returncode
    except subprocess.TimeoutExpired:
        timed=True
        os.killpg(process.pid,signal.SIGKILL)
        stdout,stderr=process.communicate()
        rc=124
    return rc,stdout.decode("ascii",errors="replace"),(
        stderr.decode("ascii",errors="replace")),monotonic_ns()-began,timed


def external_model(arm,stdout,model_file,n):
    if arm=="minisat":
        data=model_file.read_text(encoding="ascii").split()
        assert data and data[0]=="SAT"
        words=data[1:]
    else:
        words=[token for line in stdout.splitlines()
               if line.startswith("v ") for token in line.split()[1:]]
    assert words
    assigned={}
    terminated=False
    for word in words:
        v=int(word)
        assert not terminated
        if not v:
            terminated=True
        else:
            assert 1<=abs(v)<=n and abs(v) not in assigned
            assigned[abs(v)]=v>0
    assert terminated
    return sum(1<<(v-1) for v,value in assigned.items() if value)


def valid_model(clauses,bits):
    return all(any(bool(bits & (1<<(abs(lit)-1)))==(lit>0)
                   for lit in clause) for clause in clauses)


def check_bitcoin(source_dir,case,bits):
    meta=ast.literal_eval(source_dir.joinpath(
        "bitcoin_reference",case+".txt").read_text(encoding="ascii"))
    assert meta["name"]==case
    assert meta["input_sha256"]==hashlib.sha256(source_dir.joinpath(
        case+".cnf").read_bytes()).hexdigest()
    mask=(1<<meta["nonce_free_bits"])-1
    partial=sum(1<<k for k,var in meta["nonce_variables"]
                if bits & (1<<(var-1)))
    nonce=(genesis.GENESIS_NONCE & ~mask)|partial
    hdr=bytearray(genesis.HEADER_PREFIX+nonce.to_bytes(4,"little"))
    if meta["merkle_free_bits"]:
        root=sum(1<<k for k,var in meta["merkle_variables"]
                 if bits & (1<<(var-1)))
        hdr[36:68]=root.to_bytes(32,"little")
    digest=hashlib.sha256(hashlib.sha256(hdr).digest(
        )).digest()[::-1].hex()
    assert digest==genesis.GENESIS_HASH
    return nonce,hashlib.sha256(hdr).hexdigest()


def deadline(case):
    if case in cases.COMPLETION:
        return None
    if case in ("nonce4","nonce8"):
        return 60
    if case in ("nonce32","merkle_nonce288"):
        return 45
    return 30


def run_case(arm,case,source_dir,result_dir,expected,tier):
    assert arm in ARMS and case in cases.CASES
    source=source_dir.joinpath(case+".cnf")
    raw=source.read_bytes()
    sha=hashlib.sha256(raw).hexdigest()
    n,clauses=original_verify.parse_dimacs(raw)
    assert len(clauses)>0
    result_dir.mkdir(parents=True,exist_ok=True)
    model_file=result_dir.joinpath("model.bin")
    external_file=result_dir.joinpath("model.sol")
    if arm in RESEARCH:
        command=[sys.executable,"-s","-B",
                 str(ROOT.joinpath("experiments","complexity","adapter.py")),
                 arm,str(source),str(model_file)]
    elif arm=="minisat":
        command=["minisat","-verb=0",str(source),str(external_file)]
    else:
        command=[arm,str(source)]
    cap=deadline(case)
    assert (tier=="completion")==(cap is None)
    rc,stdout,stderr,elapsed,timed=run_process(command,cap)
    status="HOLD_RESOURCE"
    native="NONE"
    witness="NONE"
    nonce="NONE"
    bitcoin_sha="NONE"
    nodes="NONE"
    checks="NONE"
    if timed:
        assert rc==124
    elif arm in RESEARCH and rc==0:
        reports=[line for line in stdout.splitlines()
                 if line.startswith("COMPLEXITY_SOLVER|")]
        assert len(reports)==1 and reports[0].endswith("|json=0")
        native=ast.literal_eval(reports[0].split("|",2)[1])
        assert native["arm"]==arm and native["original_sha256"]==sha
        assert native["node_limit"]=="NONE"
        status=native["status"]
        nodes=native["search_nodes"]
        checks=native["literal_checks"]
        assert status in ("SAT","UNSAT_HALL","UNSAT_XOR","UNSAT_DPLL")
    elif arm in EXTERNAL and rc in (10,20):
        status="SAT" if rc==10 else "UNSAT_SOLVER"
    else:
        raise RuntimeError("solver error arm="+arm+" case="+case+
                           " rc="+str(rc)+" "+stderr[-1200:])
    if status=="SAT":
        if arm in RESEARCH:
            data=model_file.read_bytes()
            assert len(data)==(n+7)//8
            bits=int.from_bytes(data,"little")
            assert bits<(1<<n)
        else:
            bits=external_model(arm,stdout,external_file,n)
            data=bits.to_bytes((n+7)//8,"little")
            model_file.write_bytes(data)
        assert expected=="SAT" and valid_model(clauses,bits)
        witness=hashlib.sha256(data).hexdigest()
        if case in ("nonce4","nonce8","nonce32","merkle_nonce288"):
            nonce,bitcoin_sha=check_bitcoin(source_dir,case,bits)
    elif status.startswith("UNSAT"):
        assert expected=="UNSAT"
    else:
        assert status=="HOLD_RESOURCE"
    if external_file.exists():
        external_file.unlink()
    receipt=dict(
        arm=arm,case=case,tier=tier,expected=expected,status=status,
        original_sha256=sha,variables=n,clauses=len(clauses),
        no_per_case_deadline=(cap is None),
        resource_guard_seconds=cap if cap is not None else "NONE",
        node_limit="NONE",elapsed_ns=elapsed,
        solver_exit=rc,nodes=nodes,literal_checks=checks,
        model_sha256=witness,recovered_nonce=nonce,
        bitcoin_header_sha256=bitcoin_sha,
        native=native)
    result_dir.joinpath("receipt.txt").write_text(
        repr(receipt)+"\n",encoding="ascii")
    result_dir.joinpath("solver.stdout.txt").write_text(
        stdout[-2500:],encoding="utf-8")
    result_dir.joinpath("solver.stderr.txt").write_text(
        stderr[-2500:],encoding="utf-8")
    print("COMPLEXITY_CASE|arm="+arm+"|case="+case+
          "|tier="+tier+"|status="+status+
          "|nodes="+str(nodes)+"|elapsed_ns="+str(elapsed)+
          "|source=PASS|model_checked=PASS",flush=True)
    return receipt


def main(arm,original_dir,out):
    assert arm in ARMS
    source_dir=Path(original_dir)
    path=Path(out).joinpath(arm)
    path.mkdir(parents=True,exist_ok=True)
    manifest=[ast.literal_eval(line) for line in
              source_dir.joinpath("MANIFEST.txt").read_text(
                  encoding="ascii").splitlines()]
    assert len(manifest)==16
    summary={}
    for case,n,m,expected,tier,sha in manifest:
        assert case in cases.CASES
        assert hashlib.sha256(source_dir.joinpath(
            case+".cnf").read_bytes()).hexdigest()==sha
        result=run_case(arm,case,source_dir,path.joinpath(case),expected,tier)
        assert result["original_sha256"]==sha
        summary[case]=result["status"]
    complete=sum(v=="SAT" or v.startswith("UNSAT")
                 for v in summary.values())
    hold=16-complete
    assert len(summary)==16
    print("COMPLEXITY_ARM_FINAL|arm="+arm+"|cases=16|"
          "complete="+str(complete)+"|hold="+str(hold)+
          "|completion_no_deadline=7|scaling_resource_guard=9|status=PASS")


if __name__=="__main__":
    assert len(sys.argv)==4
    main(*sys.argv[1:])
