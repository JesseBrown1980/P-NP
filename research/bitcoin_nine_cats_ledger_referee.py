#!/usr/bin/env python3
"""A separate referee for Brown ledger nine-cat Bitcoin observations.

Uses neither the experiment's feature extractor nor its ledger functions.
Verifies the authentic two SHA-pinned original Bitcoin CNFs and reconstructs
the purported nine-cat choices from raw signed literal incidence.
"""
import argparse
import hashlib
import json
import pathlib

REFS={
    "nonce32":("6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3",239112,877879,124075),
    "merkle_nonce288":("601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b",361188,1325357,244087)
}
def require(ok,reason):
    if not ok:raise ValueError(reason)
def digest(s):return hashlib.sha256(s).hexdigest()

def independent_cats(full,location):
    n=len(full);base=location-1
    forward=[];backward=[];center=[]
    for radius in (1,2,3):
        l=[full[(base-k)%n] for k in range(1,radius+1)]
        r=[full[(base+k)%n] for k in range(1,radius+1)]
        forward.append(sum(r))
        backward.append(sum(l))
        center.append(sum(a*b for a,b in zip(l,r)))
    return tuple(forward+center+backward)

def independent_source(path,expected,variables,clauses):
    require(digest(path.read_bytes())==expected,"Bitcoin source sha differs")
    signs=[0]*variables
    rows=0
    for line in path.open("r",encoding="ascii"):
        if line[0:1] in ("p","c") or not line.strip():continue
        bits=line.split()
        require(bits[-1]=="0","undelimited source clause")
        for literal in bits[:-1]:
            a=int(literal)
            signs[abs(a)-1]+=1 if a>0 else -1
        rows+=1
    require(rows==clauses,"Bitcoin actual source clause count differs")
    return tuple((x>0)-(x<0) for x in signs)

def verify():
    ap=argparse.ArgumentParser()
    ap.add_argument("--source",type=pathlib.Path,required=True)
    ap.add_argument("--report",type=pathlib.Path,required=True)
    ap.add_argument("--out",type=pathlib.Path,required=True)
    args=ap.parse_args()
    record=json.loads(args.report.read_text("utf-8"))
    require(record["schema"]=="BTC_NINE_CATS_LEDGER_TEST_V1","unknown nine cat ledger")
    need=record["nine_cat_negative_controls"]
    require(need["nine_observers"]==9 and need["boxes"]==3
            and need["shells_per_box"]==3
            and need["directed_torus_links"]==6,
            "claimed nine cat box/radius mapping differs")
    require(need["nine_cats_alone_lossless"] is False
            and need["remote_729_variants_same_observations"]==1,
            "unpaid distant-trit collision is not acknowledged")
    close=record["genesis_closure"]
    require(close["source_bytes"]==80 and close["paid_total_bytes"]==81
            and close["paid_closure_bytes"]==27
            and close["retained_bytes"]==54
            and close["restore_exact"] and close["unpaid_third_not_unique"],
            "incorrect Shannon cost or third-channel closure")
    prefix=bytes.fromhex("01000000"+"00"*32
        +"3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a"
        +"29ab5f49"+"ffff001d")
    h=prefix+bytes.fromhex("1dac2b7c")
    require(len(h)==80
            and hashlib.sha256(hashlib.sha256(h).digest()).digest()[::-1].hex()
            =="000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
            "independent genesis source SHA256d differs")
    padded=h+b"\x00"
    width=27
    a,b,c=(padded[i*width:(i+1)*width] for i in range(3))
    paid=bytes((-(x+y+z))%256 for x,y,z in zip(a,b,c))
    require(bytes((-x-y-p)%256 for x,y,p in zip(a,b,paid))==c,
            "standalone 2/3 paid closure roundtrip failed")
    checks=[]
    for case,(original_sha,variables,clauses,anchor) in REFS.items():
        values=independent_source(args.source/(case+".cnf"),original_sha,variables,clauses)
        require(len(values)==variables,"original source state width differs")
        row=record["bitcoin_cnf"][case]
        require(row["original_variables"]==variables and row["original_clauses"]==clauses
                and row["source_sha256"]==original_sha
                and row["source_index_sha256"]==digest(bytes(v+1 for v in values)),
                "source literal signed incidence bank mismatch")
        candidates=[((anchor+i-1)%variables)+1 for i in range(27)]
        scores=[(sum(abs(v) for v in independent_cats(values,c)),c)
                for c in candidates]
        score,choice=max(scores,key=lambda x:(x[0],-x[1]))
        require(choice==row["best_candidate"] and score==row["best_score"]
                and sum(s for s,_ in scores)==row["mean_nine_abs_score_num"],
                "independently recomputed nine observations/choices differ")
        require(row["candidate_observation_count"]==243
                and row["saved_nine_observations_i16_bytes"]==486
                and row["view_storage_if_recomputed_each_time_bytes"]==0,
                "observed/stored nine-cat cost understated")
        checks.append("BTC_NINE_CATS_REFEREE|case={}|choice={}|score={}|source=EXACT|status=PASS|json=0".format(case,choice,score))
    lines="\n".join(checks)+"\n"
    summary=("BTC_NINE_CATS_REFEREE_FINAL|cases=2|cats=9|boxes=3|"
             "links=6|two_wave_key_directions=CHECKED|"
             "paid_shannon_closure=PASS|nonunique_without_bank=PASS|"
             "original_SHA256d_CNF=EXACT|"
             f"row_sha256={digest(lines.encode())}|status=PASS|json=0\n")
    args.out.write_text(summary+lines,"utf-8")
    print(summary+lines,end="",flush=True)

if __name__=="__main__":verify()
