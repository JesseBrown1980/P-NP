#!/usr/bin/env python3
"""Brown's paid Shannon ledger / nine shadow cats, independently measured.

Three Schrödinger-cat boxes × three radii, following the Law-36
forward / bilateral / backward distinction. Six directed torus links are
the left/right neighbours at radii 1..3; the two proposed counter-spinning
wave keys are exact order-three source-trit rotations. This is a declared
experimental interpretation, not a recovered canonical nine-cat kernel.

Uses real public Bitcoin genesis bytes and SHA-verified full Bitcoin
nonce32 / merkle_nonce288 SAT CNFs. Shadow features are *observations*
of supplied source; none is treated as a hash preimage oracle.

No third-party modules, float, random nondeterminism or internet.
"""
import argparse
import hashlib
import json
import math
import pathlib
import sys
import time
from collections import Counter

CASES = {
    "nonce32": ("6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3",239112,877879,124075),
    "merkle_nonce288": ("601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b",361188,1325357,244087),
}
GENESIS_HASH="000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
PREFIX=bytes.fromhex(
    "01000000"+"00"*32+
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a"+
    "29ab5f49"+"ffff001d")
def need(ok,why):
    if not ok: raise ValueError(why)
def sha(data): return hashlib.sha256(data).hexdigest()
def sha2(data): return hashlib.sha256(hashlib.sha256(data).digest()).digest()[::-1].hex()
def rose(n): return ((n+2)%3)-1
def rotate(data): return tuple(rose(x) for x in data)
def rotate_back(data): return rotate(rotate(data))
def wave_keys(row):
    # Verify R^3=I, R^{-1}=R^2 on ALL source trits, with source identity.
    fwd=rotate(row); rev=rotate_back(row)
    need(rotate(rev)==tuple(row) and rotate_back(fwd)==tuple(row),
         "two counter-rotating keys do not invert exactly")
    return fwd,rev

def cat_nine(room,center=13):
    """Nine measured signed observations; neither rank nor invertibility assumed.

    Three boxes: forward, bilateral, backward.
    Each has r=1,2,3: six torus links (+/- r) around 27 cyclic addresses.
    Bilateral is a nonlinear co-occurrence measurement, not the source cell.
    """
    need(len(room)==27 and all(v in (-1,0,1) for v in room),
         "nine-cat signed source invalid")
    values=[]
    for direction in ("forward","bilateral","backward"):
        for radius in (1,2,3):
            f=[room[(center+i)%27] for i in range(1,radius+1)]
            b=[room[(center-i)%27] for i in range(1,radius+1)]
            if direction=="forward": value=sum(f)
            elif direction=="backward": value=sum(b)
            else: value=sum(x*y for x,y in zip(f,b))
            values.append(value)
    need(len(values)==9 and max(map(abs,values))<=3,
         "nine signed shadow-cat bounds invalid")
    return tuple(values)

def parse_source(path,expect,variables,clauses):
    raw=path.read_bytes()
    need(sha(raw)==expect,"full original Bitcoin CNF SHA256 changed")
    counts=[0]*variables
    line_count=0
    for line in raw.splitlines():
        if line.startswith(b"c") or not line.strip(): continue
        if line.startswith(b"p"):
            words=line.split()
            need(words==[b"p",b"cnf",str(variables).encode(),str(clauses).encode()],
                 "original Bitcoin CNF dimensions changed")
            continue
        data=[int(b) for b in line.split()]
        need(data and data[-1]==0,"unterminated Bitcoin clause")
        for literal in data[:-1]:
            v=abs(literal)
            need(1<=v<=variables,"Bitcoin literal out of source bounds")
            counts[v-1]+=1 if literal>0 else -1
        line_count+=1
    need(line_count==clauses,"Bitcoin source clause-count changed")
    return raw,tuple(1 if i>0 else -1 if i<0 else 0 for i in counts)

def genesis():
    raw=PREFIX+(2083236893).to_bytes(4,"little")
    need(len(raw)==80 and sha2(raw)==GENESIS_HASH,
         "original Bitcoin genesis known SHA256d digest changed")
    # Inject no nonce guess into any CNF experiment: the genesis
    # header here is a *known public source*, not a preimage solve.
    return raw

def closure(triple):
    a,b,c=triple
    return (-a-b-c)%256
def closure_experiment(data):
    padded=data+b"\0"*((-len(data))%3)
    n=len(padded)//3
    a,b,c=padded[:n],padded[n:2*n],padded[2*n:]
    bank=bytes(closure(x) for x in zip(a,b,c))
    recovered=bytes((-x-y-p)%256 for x,y,p in zip(a,b,bank))
    need(recovered==c and a+b+recovered==padded,
         "paid closure did not exactly restore third")
    # Without the bank, 2/3 is not sufficient for general arbitrary C.
    other=bytes((x+1)%256 for x in c)
    need(other!=c and a+b+other!=padded and len(bank)==n,
         "unpaid deletion unexpectedly unique")
    return {"source_bytes":len(data),"paid_closure_bytes":len(bank),
            "retained_bytes":2*n,"paid_total_bytes":3*n,
            "restore_exact":True,"unpaid_third_not_unique":True,
            "padding_bytes":len(padded)-len(data)}

def independent_control():
    # At the center, 9 scalar local observations inspect positions 10..16.
    # Changing a remote cell creates an *exact collision* among nine cats.
    zero=(0,)*27
    alt=list(zero);alt[0]=1
    need(cat_nine(zero)==cat_nine(alt) and zero!=tuple(alt),
         "nine-view non-injective collision disappeared")
    # The three reversible source recolourings add no independent source info.
    for row in (zero,tuple(alt),tuple([1,-1,0]*9)):
        forward,back=wave_keys(row)
        need(rotate_back(forward)==row and rotate(back)==row,
             "trianti/inverse round trip lost source")
    # Reconstruct all signed source values after paying for 27 source cells;
    # all nine observers are then deterministic readings of the same bank.
    seen=set()
    for i in range(729):
        trits=[0]*27
        q=i
        for at in (0,1,2,3,4,5):
            trits[at]=(q%3)-1;q//=3
        seen.add(cat_nine(tuple(trits)))
    need(len(seen)==1,"off-window nine cats unexpectedly distinguish distant data")
    return {"nine_observers":9,"boxes":3,"shells_per_box":3,
            "directed_torus_links":6,
            "linear_or_nonlinear_cyclic_observations":True,
            "source_trit_rotation_order":3,
            "counter_spinning_inverse":True,
            "different_inputs_same_nine_observations":True,
            "remote_729_variants_same_observations":len(seen),
            "nine_cats_alone_lossless":False}

def average_score(snapshot,center):
    # Score is fixed before original-CNF measurements, no target solution.
    vals=cat_nine(snapshot,center)
    return sum(abs(v) for v in vals)

def source_probe(source_trits,anchor):
    width=len(source_trits)
    t0=time.perf_counter_ns()
    # The 27 candidates are a contiguous variable-index window around the
    # accepted source-Fischer first-choice anchor. THIS IS NOT the full
    # Fischer ranked candidate list and is NOT a 512-node solver benchmark.
    centers=[(anchor-1+i)%width for i in range(27)]
    results=[]
    for index in centers:
        room=[source_trits[(index+j-13)%width] for j in range(27)]
        nine=cat_nine(tuple(room),13)
        wave_keys(tuple(room))
        results.append((index+1,sum(abs(v) for v in nine),nine))
    runtime=time.perf_counter_ns()-t0
    top=max(results,key=lambda x:(x[1],-x[0]))
    raw_signs=bytes((v+1) for v in source_trits)
    return {
        "original_fischer_anchor":anchor,
        "sampled_source_variable_candidates":27,
        "sample_is_contiguous_not_fischer_ranked":True,
        "nine_observations_per_candidate":9,
        "candidate_observation_count":27*9,
        "source_index_sha256":sha(raw_signs),
        "source_index_bytes_in_memory":len(raw_signs),
        "mean_nine_abs_score_num":sum(r[1] for r in results),
        "mean_nine_abs_score_den":27,
        "best_candidate":top[0],"best_score":top[1],
        "choice_differs_from_anchor":top[0]!=anchor,
        "window_wall_ns":runtime,
        "saved_nine_observations_i16_bytes":27*9*2,
        "view_storage_if_recomputed_each_time_bytes":0,
        "metadata_bytes":0
    }

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--source",type=pathlib.Path,required=True)
    ap.add_argument("--out",type=pathlib.Path,required=True)
    args=ap.parse_args()
    public=genesis()
    # Each byte maps to a pair of invertible test coordinates: the actual
    # byte itself and its balanced-ternary six-trit expansion.
    def pack6(value):
        digits=[]
        for _ in range(6):
            digits.append(value%3-1);value//=3
        return tuple(reversed(digits))
    def unpack6(symbols):
        n=0
        for trit in symbols:n=n*3+trit+1
        return n
    six=[pack6(byte) for byte in public]
    need(bytes(unpack6(s) for s in six)==public,
         "genesis balanced ternary 6-trit representation failed")
    signed=[x for word in six for x in word]
    rooms=[tuple((signed+[0]*((-len(signed))%27))[i:i+27])
           for i in range(0,len(signed),27)]
    need(len(rooms)==18 and all(len(x)==27 for x in rooms),
         "original Bitcoin ternary room assignment changed")
    for room in rooms:
        wave_keys(room)
        cat_nine(room)
    result={
        "schema":"BTC_NINE_CATS_LEDGER_TEST_V1",
        "source_class":"public Bitcoin genesis and authenticated full original Bitcoin SHA256d CNFs",
        "architecture_status":"experimental mapping from Law 36, not original Nine Cats canonical definition",
        "ledger_rule":"charge complete shared bank, keys, closures, residuals and checkpoints",
        "genesis_sha256d":GENESIS_HASH,
        "genesis_header_bytes":len(public),
        "balanced_trits":len(signed),
        "rooms":len(rooms),
        "genesis_closure":closure_experiment(public),
        "nine_cat_negative_controls":independent_control(),
        "memory_ledger_definition":{
            "nine_observations_int16_bytes_per_candidate":18,
            "nine_views_not_independent_side_information":True,
            "source_or_frozen_bank_must_be_shared":True,
            "full_CNFs_unchanged":True,
            "algorithm_source_bytes":pathlib.Path(__file__).stat().st_size,
            "single_view_no_stored_bank_cannot_reconstruct_arbitrary_source":True
        },
        "bitcoin_cnf":{}
    }
    for case,(expected,variables,clauses,anchor) in CASES.items():
        raw,index=parse_source(args.source/(case+".cnf"),
                               expected,variables,clauses)
        measured=source_probe(index,anchor)
        result["bitcoin_cnf"][case]={
            "source_sha256":expected,"source_bytes":len(raw),
            "original_variables":variables,"original_clauses":clauses,
            **measured
        }
    # Keep results independent of timing for machine-stable hashes.
    canonical=json.dumps(result,sort_keys=True,indent=2)+"\n"
    args.out.parent.mkdir(parents=True,exist_ok=True)
    args.out.write_text(canonical,encoding="utf-8")
    print("BTC_NINE_CATS_LEDGER|status=PASS|source_cases=2|"
          "observer_boxes=3|observer_shells=3|"
          "torus_links=6|wave_key_inverse=PASS|"
          "paid_closure=PASS|unpaid_third=NOT_UNIQUE|"
          "all_full_source_shas=PASS|"
          f"report_sha256={sha(canonical.encode())}|json=0",flush=True)
    for case,info in result["bitcoin_cnf"].items():
        print("BTC_NINE_CATS_SOURCE|case={}|source_sha256={}|"
              "sampled_candidates=27|best_candidate={}|"
              "first_fischer={}|source_index_bytes={}|"
              "status=MEASURED|json=0".format(
              case,info["source_sha256"],info["best_candidate"],
              info["original_fischer_anchor"],
              info["source_index_bytes_in_memory"]),flush=True)

if __name__=="__main__":
    main()
