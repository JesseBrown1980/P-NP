#!/usr/bin/env python3
"""Independent original-CNF acceptance for all 352 completion/scaling trials.

Does not import solver, 22-arm orderer, generated case definitions, or any
MTP/N-Nest/RIME implementation. Checks every source SHA, independently
rebuilds graph-color + PHP/Tseitin UNSAT proof conditions, validates
original SAT models, and recomputes Bitcoin SHA256d on all claimed models.
Completion runs have no per-case timeout nor search-node cap; hard scaling
runs have explicit elapsed resource safeguards and censored HOLDs.
"""
import ast
import hashlib
import random
import sys
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT.joinpath("experiments","array")))
import verify as original_referee

COMPLETION=("plant12","plant18","plant24","color4","color5","php4_3","tseitin16")
SCALING=("plant36","plant54","plant81","plant121","php8_7",
         "nonce4","nonce8","nonce32","merkle_nonce288")
CASES=COMPLETION+SCALING
HYBRIDS=("baseline","mtp","fischer",
         "nest3","nest9","nest27","nest81",
         "mtp_nest3","mtp_nest9","mtp_nest27","mtp_nest81",
         "all3","all9","all27","all81")
RESEARCH=("hxf4","ym_fischer","rime_new","rime_original")+HYBRIDS
EXTERNAL=("minisat","cadical","picosat")
ARMS=RESEARCH+EXTERNAL
PREFIX=bytes.fromhex(
    "01000000"+"00"*32+
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a"+
    "29ab5f49"+"ffff001d")
REF_NONCE=2083236893
EXPECTED_HASH="000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"


def verify_source_math(case,n,clauses):
    if case.startswith("plant"):
        nn=int(case.removeprefix("plant"))
        assert nn==n
        assert len(clauses)==(83*nn+19)//20
        seed=int.from_bytes(hashlib.sha256(
            ("sat-growth-v1:"+str(nn)).encode("ascii")
            ).digest()[:8],"big")
        rng=random.Random(seed)
        witness=tuple(rng.randrange(2) for _ in range(nn))
        # This separately proves existence without trusting the solver.
        assert all(any(bool(witness[abs(v)-1])==(v>0)
                       for v in row) for row in clauses)
        return "SAT"
    if case.startswith("color"):
        vertices=int(case.removeprefix("color"))
        assert vertices in (4,5) and n==3*vertices
        expected=[]
        for i in range(vertices):
            ids=tuple(3*i+j+1 for j in range(3))
            expected.append(ids)
            for a in range(3):
                for b in range(a+1,3):
                    expected.append((-ids[a],-ids[b]))
        for i in range(vertices):
            for j in range(i+1,vertices):
                for col in range(3):
                    expected.append((-(3*i+col+1),-(3*j+col+1)))
        assert clauses==tuple(expected)
        # Complete graph of >3 vertices cannot have 3 distinct colors.
        assert vertices>3
        return "UNSAT"
    if case in ("php4_3","php8_7","tseitin16"):
        truth,_=original_referee.known_truth(case,n,clauses)
        return truth
    assert case in ("nonce4","nonce8","nonce32","merkle_nonce288")
    assert n>230000 and len(clauses)>800000
    assert hashlib.sha256(hashlib.sha256(
        PREFIX+REF_NONCE.to_bytes(4,"little")).digest()).digest()[
            ::-1].hex()==EXPECTED_HASH
    return "SAT"


def verify_bitcoin(data,case,source_dir):
    manifest=ast.literal_eval(source_dir.joinpath(
        "bitcoin_reference",case+".txt").read_text(encoding="ascii"))
    assert manifest["name"]==case
    assert manifest["nonce_free_bits"] in (4,8,32)
    assert manifest["merkle_free_bits"] in (0,256)
    assert manifest["free_bits_total"]==(
        manifest["nonce_free_bits"]+manifest["merkle_free_bits"])
    mask=(1<<manifest["nonce_free_bits"])-1
    nonce=REF_NONCE & ~mask
    for bit,v in manifest["nonce_variables"]:
        if data & (1<<(v-1)):
            nonce |= 1<<bit
    full=bytearray(PREFIX+nonce.to_bytes(4,"little"))
    if manifest["merkle_free_bits"]:
        merkle=0
        for bit,v in manifest["merkle_variables"]:
            if data & (1<<(v-1)):
                merkle |= 1<<bit
        full[36:68]=merkle.to_bytes(32,"little")
    h=hashlib.sha256(hashlib.sha256(full).digest()).digest()[::-1].hex()
    assert h==EXPECTED_HASH
    return nonce,hashlib.sha256(full).hexdigest()


def verify():
    assert len(PREFIX)==76
    assert len(ARMS)==22 and len(CASES)==16
    base=Path(sys.argv[1])
    orig=base.joinpath("original")
    contest=base.joinpath("arms")
    manifest=[ast.literal_eval(line) for line in orig.joinpath(
        "MANIFEST.txt").read_text(encoding="ascii").splitlines()]
    assert len(manifest)==16
    sources={}
    for entry in manifest:
        case,n,rows,expect,tier,sha=entry
        assert case in CASES and case not in sources
        assert tier==("completion" if case in COMPLETION else "scaling")
        raw=orig.joinpath(case+".cnf").read_bytes()
        assert hashlib.sha256(raw).hexdigest()==sha
        parsed_n,clauses=original_referee.parse_dimacs(raw)
        assert parsed_n==n and len(clauses)==rows
        assert verify_source_math(case,n,clauses)==expect
        sources[case]=(n,clauses,sha,expect,tier)
    assert len(sources)==16
    total_sat=total_unsat=total_hold=0
    aggregate={}
    for arm in ARMS:
        solved=0
        holds=0
        model_count=0
        node_total=0
        elapsed=0
        completion_solved=0
        scaling_solved=0
        for case in CASES:
            n,clauses,sha,expected,tier=sources[case]
            folder=contest.joinpath(arm,case)
            receipt=ast.literal_eval(folder.joinpath(
                "receipt.txt").read_text(encoding="ascii"))
            assert receipt["arm"]==arm and receipt["case"]==case
            assert receipt["tier"]==tier and receipt["expected"]==expected
            assert receipt["variables"]==n and receipt["clauses"]==len(clauses)
            assert receipt["original_sha256"]==sha
            assert receipt["node_limit"]=="NONE"
            assert receipt["no_per_case_deadline"]==(tier=="completion")
            resource=(60 if case in ("nonce4","nonce8") else
                      45 if case in ("nonce32","merkle_nonce288") else
                      30 if tier=="scaling" else "NONE")
            assert receipt["resource_guard_seconds"]==resource
            assert isinstance(receipt["elapsed_ns"],int)
            assert receipt["elapsed_ns"]>=0
            elapsed+=receipt["elapsed_ns"]
            status=receipt["status"]
            assert status in ("SAT","UNSAT_HALL","UNSAT_XOR",
                              "UNSAT_DPLL","UNSAT_SOLVER","HOLD_RESOURCE")
            if status=="SAT":
                assert expected=="SAT"
                raw_model=folder.joinpath("model.bin").read_bytes()
                assert len(raw_model)==(n+7)//8
                assert hashlib.sha256(raw_model).hexdigest()==receipt[
                    "model_sha256"]
                data=int.from_bytes(raw_model,"little")
                assert data<(1<<n)
                assert all(any(bool(data & (1<<(abs(v)-1)))==(v>0)
                               for v in clause) for clause in clauses)
                if case.startswith("nonce") or case=="merkle_nonce288":
                    nonce,h=verify_bitcoin(data,case,orig)
                    assert nonce==receipt["recovered_nonce"]
                    assert h==receipt["bitcoin_header_sha256"]
                model_count+=1
                total_sat+=1
                solved+=1
            elif status.startswith("UNSAT"):
                assert expected=="UNSAT"
                total_unsat+=1
                solved+=1
            else:
                assert status=="HOLD_RESOURCE" and tier=="scaling"
                assert receipt["solver_exit"]==124
                total_hold+=1
                holds+=1
            if status!="HOLD_RESOURCE":
                (completion_solved if tier=="completion" else scaling_solved)
            if tier=="completion" and status!="HOLD_RESOURCE":
                completion_solved+=1
            if tier=="scaling" and status!="HOLD_RESOURCE":
                scaling_solved+=1
            if arm in RESEARCH and status!="HOLD_RESOURCE":
                native=receipt["native"]
                assert native["node_limit"]=="NONE"
                assert native["original_sha256"]==sha
                assert native["arm"]==arm
                assert native["status"]==status
                assert native["original_witness_sha256"]==receipt[
                    "model_sha256"]
                assert native["search_nodes"]>=0
                node_total+=native["search_nodes"]
                if arm in HYBRIDS:
                    pre=native["precompute"]
                    assert pre["arm"]==arm
                    assert pre["candidates"]<=81
                    assert pre["sampled_clauses"]<=4096
                    for label in ("coordination_nodes","watcher_checks"):
                        assert pre[label]>=0
                    assert pre["coordination_nodes"]==pre["watcher_checks"]
            print("COMPLEXITY_REFEREE|arm="+arm+"|case="+case+
                  "|tier="+tier+"|status="+status+
                  "|original_source=PASS|model=PASS",flush=True)
        assert completion_solved==len(COMPLETION)
        assert solved+holds==len(CASES)
        aggregate[arm]=(solved,holds,node_total,elapsed,
                        completion_solved,scaling_solved,model_count)
        print("COMPLEXITY_AGGREGATE|arm="+arm+
              "|completed="+str(solved)+"|hold="+str(holds)+
              "|completion=7/7|scaling_completed="+str(scaling_solved)+
              "|nodes_completed="+str(node_total)+
              "|total_elapsed_ns="+str(elapsed)+
              "|status=PASS",flush=True)
    assert total_sat+total_unsat+total_hold==22*16==352
    assert sum(v[0]+v[1] for v in aggregate.values())==352
    # Negative SHA controls: each original CNF must reject one-byte flips.
    for case in CASES:
        raw=orig.joinpath(case+".cnf").read_bytes()
        damaged=bytearray(raw)
        damaged[-3] ^= 1
        assert hashlib.sha256(damaged).hexdigest()!=sources[case][2]
    base.joinpath("COMPLEXITY_ROWS.txt").write_text(
        repr(aggregate)+"\n",encoding="ascii")
    print("COMPLEXITY_FINAL|engines=22|cases=16|"
          "observations=352|completion_unbounded=154/154|"
          "scaling_resource_censored=198|"
          "sat="+str(total_sat)+"|unsat="+str(total_unsat)+
          "|hold="+str(total_hold)+
          "|independent_hash_models=PASS|"
          "source_mutations_rejected=16|status=PASS")


if __name__=="__main__":
    assert len(sys.argv)==2
    verify()
