#!/usr/bin/env python3
"""15 bounded ablation arms over 18 archived original CNF inputs.

Deterministic input archive, independent known SAT/UNSAT class checks,
original-CNF model checks, true full SHA256d Bitcoin header check.
A timeout is HOLD_TIME (no invented result).
"""
import ast
import hashlib
import sys
from pathlib import Path
from subprocess import run,TimeoutExpired
from time import monotonic_ns

import hybrid

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT.joinpath("experiments","bitcoin")))
import genesis
sys.path.insert(0,str(ROOT.joinpath("experiments","array")))
import verify as original_referee

SMALL=(
    "php3_2","php4_3","php5_4","php6_5","php8_7",
    "php12_11","php21_20","php3_3","php4_4","php6_6",
    "tseitin16","tseitin32","tseitin64","tseitin128",
)
BTC=("nonce4","nonce8","nonce32","merkle_nonce288")
CASES=SMALL+BTC
TIME_LIMIT=12


def real_bitcoin_solution(bits,manifest):
    nonce=genesis.GENESIS_NONCE
    mask=(1 << manifest["nonce_free_bits"])-1
    value=sum(1<<pos for pos,v in manifest["nonce_variables"]
              if bits & (1<<(v-1)))
    nonce=(nonce&~mask)|value
    header=bytearray(genesis.HEADER_PREFIX+nonce.to_bytes(4,"little"))
    if manifest["merkle_free_bits"]:
        merkle=sum(1<<pos for pos,v in manifest["merkle_variables"]
                   if bits & (1<<(v-1)))
        header[36:68]=merkle.to_bytes(32,"little")
    assert hashlib.sha256(hashlib.sha256(
        header).digest()).digest()[::-1].hex()==genesis.GENESIS_HASH
    return nonce,bytes(header[36:68]).hex()


def run_case(arm,case,inputs,out):
    root=Path(inputs)
    source=root.joinpath("array" if case in SMALL else "bitcoin",
                         case+".cnf")
    raw=source.read_bytes()
    n,clauses=original_referee.parse_dimacs(raw)
    expected=(original_referee.known_truth(case,n,clauses)[0]
              if case in SMALL else "SAT")
    out=Path(out)
    out.mkdir(parents=True,exist_ok=True)
    target=out.joinpath("model.bin")
    command=[sys.executable,"-s","-B",
             str(ROOT.joinpath("experiments","mtp_nnest","instance.py")),
             arm,str(source),str(target)]
    started=monotonic_ns()
    try:
        child=run(command,capture_output=True,check=False,
                  timeout=TIME_LIMIT)
        rc=child.returncode
        stdout=child.stdout.decode("ascii",errors="replace")
        stderr=child.stderr.decode("ascii",errors="replace")
    except TimeoutExpired as err:
        rc=124
        stdout=(err.stdout or b"").decode("ascii",errors="replace")
        stderr=(err.stderr or b"").decode("ascii",errors="replace")
    elapsed=monotonic_ns()-started
    status="HOLD_TIME"
    native="NONE"
    nonce="NONE"
    merkle="NONE"
    witness_sha="NONE"
    if rc==124:
        assert not target.exists()
    elif rc==0:
        match=[line for line in stdout.splitlines()
               if line.startswith("MTP_NNEST_RESULT|")]
        assert len(match)==1 and match[0].endswith("|json=0")
        native=ast.literal_eval(match[0].split("|",2)[1])
        assert native["arm"]==arm
        assert native["input_sha256"]==hashlib.sha256(raw).hexdigest()
        assert native["variables"]==n
        assert native["clauses"]==len(clauses)
        status=native["status"]
        if status=="SAT":
            packed=target.read_bytes()
            assert len(packed)==(n+7)//8
            witness_sha=hashlib.sha256(packed).hexdigest()
            assert witness_sha==native["model_sha256"]
            integer=int.from_bytes(packed,"little")
            assert integer < (1<<n)
            assert all(any(bool(integer & (1<<(abs(lit)-1)))==(
                lit>0) for lit in clause) for clause in clauses)
            assert expected=="SAT"
            if case in BTC:
                meta=ast.literal_eval(root.joinpath(
                    "bitcoin",case+".txt").read_text(encoding="ascii"))
                nonce,merkle=real_bitcoin_solution(integer,meta)
        elif status in ("UNSAT_HALL","UNSAT_XOR","UNSAT_DPLL"):
            assert expected=="UNSAT"
        else:
            assert status=="HOLD_BUDGET"
            assert native["nodes"]==2000
    else:
        raise RuntimeError("error arm="+arm+" case="+case+" rc="+str(rc)+
                           " stderr="+stderr[-2000:])
    receipt=dict(arm=arm,case=case,status=status,
                 expected=expected,original_sha256=hashlib.sha256(raw).hexdigest(),
                 variables=n,clauses=len(clauses),
                 source_historical_fischer="FROZEN",
                 status_kind="COMPLETED" if status.startswith("UNSAT") or
                 status=="SAT" else "HOLD",
                 elapsed_ns=elapsed,
                 limit_seconds=TIME_LIMIT,
                 process_exit=rc,
                 recovered_nonce=nonce,recovered_merkle=merkle,
                 original_model_sha256=witness_sha,
                 native=native)
    out.joinpath("receipt.txt").write_text(
        repr(receipt)+"\n",encoding="ascii")
    out.joinpath("stdout.txt").write_text(stdout[-2400:],encoding="ascii")
    out.joinpath("stderr.txt").write_text(stderr[-2400:],encoding="ascii")
    print("MTP_NNEST_CASE|arm="+arm+"|case="+case+
          "|status="+status+"|elapsed_ns="+str(elapsed)+
          "|model=VERIFIED_IF_SAT",flush=True)
    return receipt


def main(arm,inputs,out):
    assert arm in hybrid.ARMS
    rows=[]
    for case in CASES:
        rows.append(run_case(arm,case,inputs,Path(out).joinpath(
            arm,case)))
    assert len(rows)==18
    sats=sum(row["status"]=="SAT" for row in rows)
    unsat=sum(row["status"].startswith("UNSAT") for row in rows)
    hold=18-sats-unsat
    print("MTP_NNEST_ARM_FINAL|arm="+arm+"|cases=18|"
          "sat="+str(sats)+"|unsat="+str(unsat)+
          "|hold="+str(hold)+"|status=PASS")


if __name__=="__main__":
    assert len(sys.argv)==4
    main(*sys.argv[1:])
