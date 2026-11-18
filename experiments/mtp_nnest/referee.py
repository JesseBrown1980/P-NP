#!/usr/bin/env python3
"""Clean-room 270-receipt referee: no hybrid or solver imports.

Check original SHA256, SAT model replay on full original CNF, known PHP
pigeonhole and odd-charge Tseitin mathematical UNSAT conditions, Bitcoin
header SHA256d if SAT, bounded decision status, virtual 3^k watcher tree
node counts, source-free prep labels and aggregate resource usage.
"""
import ast
import hashlib
import sys
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT.joinpath("experiments","array")))
import verify as original_verify

ARMS=("baseline","mtp","fischer",
      "nest3","nest9","nest27","nest81",
      "mtp_nest3","mtp_nest9","mtp_nest27","mtp_nest81",
      "all3","all9","all27","all81")
SMALL=("php3_2","php4_3","php5_4","php6_5","php8_7",
       "php12_11","php21_20","php3_3","php4_4","php6_6",
       "tseitin16","tseitin32","tseitin64","tseitin128")
BTC=("nonce4","nonce8","nonce32","merkle_nonce288")
CASES=SMALL+BTC
GENESIS_NONCE=2083236893
HEADER_PREFIX=bytes.fromhex(
    "01000000"+"00"*32+
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a"+
    "29ab5f49"+"ffff001d")
TARGET="000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"


def expected_depth(arm):
    for p in ("nest","mtp_nest","all"):
        if arm.startswith(p):
            n=int(arm[len(p):])
            assert n in (3,9,27,81)
            return (n,{3:1,9:2,27:3,81:4}[n])
    return (0,0)


def validate_bitcoin(bits,manifest):
    nonce_mask=(1<<manifest["nonce_free_bits"])-1
    recovered=sum(1<<bit for bit,v in manifest["nonce_variables"]
                  if bits & (1<<(v-1)))
    nonce=(GENESIS_NONCE&~nonce_mask)|recovered
    full=bytearray(HEADER_PREFIX+nonce.to_bytes(4,"little"))
    if manifest["merkle_free_bits"]:
        merkle=sum(1<<bit for bit,v in manifest["merkle_variables"]
                   if bits & (1<<(v-1)))
        full[36:68]=merkle.to_bytes(32,"little")
    assert hashlib.sha256(hashlib.sha256(full).digest(
        )).digest()[::-1].hex()==TARGET
    return nonce,bytes(full[36:68]).hex()


def main(root):
    base=Path(root)
    originals=base.joinpath("original")
    outcomes=base.joinpath("contest")
    sat=unsat=hold=0
    totals={}
    for arm in ARMS:
        accepted=0
        timing=0
        branchwork=0
        overhead=0
        leaves,depth=expected_depth(arm)
        for case in CASES:
            original=originals.joinpath(
                "array" if case in SMALL else "bitcoin",case+".cnf")
            raw=original.read_bytes()
            n,clauses=original_verify.parse_dimacs(raw)
            sha=hashlib.sha256(raw).hexdigest()
            truth=(original_verify.known_truth(case,n,clauses)[0]
                   if case in SMALL else "SAT")
            folder=outcomes.joinpath(arm,case)
            row=ast.literal_eval(folder.joinpath("receipt.txt"
                ).read_text(encoding="ascii"))
            assert row["arm"]==arm and row["case"]==case
            assert row["expected"]==truth and row["variables"]==n
            assert row["clauses"]==len(clauses)
            assert row["original_sha256"]==sha
            assert row["limit_seconds"]==12
            assert row["source_historical_fischer"]=="FROZEN"
            assert row["elapsed_ns"]>=0
            timing+=row["elapsed_ns"]
            status=row["status"]
            assert status in ("SAT","UNSAT_HALL","UNSAT_XOR",
                              "UNSAT_DPLL","HOLD_TIME","HOLD_BUDGET")
            if status=="SAT":
                assert truth=="SAT"
                packed=folder.joinpath("model.bin").read_bytes()
                assert len(packed)==(n+7)//8
                assert hashlib.sha256(packed).hexdigest()==row[
                    "original_model_sha256"]
                bits=int.from_bytes(packed,"little")
                assert bits<(1<<n)
                assert all(any(bool(bits & (1<<(abs(v)-1)))==(v>0)
                               for v in clause) for clause in clauses)
                if case in BTC:
                    manifest=ast.literal_eval(originals.joinpath(
                        "bitcoin",case+".txt").read_text(encoding="ascii"))
                    assert manifest["input_sha256"]==sha
                    nonce,merkle=validate_bitcoin(bits,manifest)
                    assert row["recovered_nonce"]==nonce
                    assert row["recovered_merkle"]==merkle
                sat+=1
                accepted+=1
            elif status.startswith("UNSAT"):
                assert truth=="UNSAT"
                unsat+=1
                accepted+=1
            else:
                hold+=1
            if status=="HOLD_TIME":
                assert row["process_exit"]==124
                assert row["native"]=="NONE"
            else:
                receipt=row["native"]
                assert receipt["input_sha256"]==sha
                assert receipt["arm"]==arm
                assert receipt["node_cap"]==2000
                assert receipt["status"]==status
                assert receipt["nodes"]<=2000
                assert receipt["model_sha256"]==row[
                    "original_model_sha256"]
                if status=="HOLD_BUDGET":
                    assert receipt["nodes"]==2000
                prep=receipt["preprocess"]
                assert prep["arm"]==arm
                assert prep["levels"]==leaves
                assert prep["sampled_clauses"]<=4096
                assert prep["candidates"]<=81
                assert prep["prep_ns"]>=0
                assert prep["mtp_freeze_ns"]>=0
                assert prep["fischer_ns"]>=0
                assert prep["nest_ns"]>=0
                expected_nodes=(pow(3,depth+1)-1)//2 if leaves else 0
                assert prep["coordination_nodes"]==expected_nodes
                assert prep["watcher_checks"]==expected_nodes
                if leaves:
                    assert len(prep["root_sha256"])==64
                else:
                    assert prep["root_sha256"]=="NONE"
                branchwork+=receipt["nodes"]
                overhead+=prep["prep_ns"]
            print("MTP_NNEST_REFEREE|arm="+arm+"|case="+case+
                  "|status="+status+"|source=PASS|model=CHECKED",
                  flush=True)
        totals[arm]=(accepted,18-accepted,branchwork,timing,overhead)
        print("MTP_NNEST_AGGREGATE|arm="+arm+
              "|completed="+str(accepted)+
              "|hold="+str(18-accepted)+
              "|nodes="+str(branchwork)+
              "|elapsed_ns="+str(timing)+
              "|prep_ns="+str(overhead)+"|status=PASS",flush=True)
    assert len(totals)==15
    assert sat+unsat+hold==15*18==270
    assert sat+unsat+hold==sum(sum(x[:2]) for x in totals.values())
    base.joinpath("SUMMARY.txt").write_text(repr(totals)+"\n",
                                       encoding="ascii")
    print("MTP_NNEST_FINAL|arms=15|original_cases=18|"
          "observations=270|source_integrity=PASS|"
          "independent_original_models=PASS|nested_watcher_levels=3,9,27,81|"
          "sat="+str(sat)+"|unsat="+str(unsat)+"|hold="+str(hold)+
          "|status=PASS")


if __name__=="__main__":
    assert len(sys.argv)==2
    main(sys.argv[1])
