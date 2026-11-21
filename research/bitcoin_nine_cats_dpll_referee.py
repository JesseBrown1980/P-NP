#!/usr/bin/env python3
"""Independent acceptance for original Bitcoin full-SHA256d nine-cat DPLL.

Tests exact 128→512 restart vs separately initiated NEW→512, parses every
saved source-bound assignment and pending polarity frame, and measures
source-native clause work against prior *accepted* direct/one/three 512 nodes.
No inference from historical labels: all actual nine-cat data come from
full authentic original-CNF GitHub runners.
"""
import argparse
import hashlib
from pathlib import Path

import bitcoin_frontier_referee as proof
import bitcoin_live_sat_referee as btc
import bitcoin_gguf_fischer_referee as gguf

proof.MODES=("direct","one","three","nine")
BASE={
"nonce32": {"direct":1740534501,"one":1745415940,"three":1747887714},
"merkle_nonce288":{"direct":2652585890,"one":2637785521,"three":2652789996}
}
def sha(data):return hashlib.sha256(data).hexdigest()
def need(ok,msg):
    if not ok:raise ValueError(msg)

def compare(case,source_root,gguf_root,root):
    source_sha,vars,clauses=btc.CASES[case]
    original=(source_root/(case+".cnf")).read_bytes()
    need(sha(original)==source_sha,"authentic original Bitcoin SHA256d CNF changed")
    gguf_bytes=(gguf_root/case/"BITCOIN.three_shadows.gguf").read_bytes()
    full,seed,meta=gguf.parse(gguf_bytes)
    need(full==original and seed==bytes(vars)
         and meta["pnp.original_cnf_sha256"]==source_sha,
         "original source-frozen Bitcoin GGUF differs")
    seed_sha=sha(seed)
    mode="nine"
    first=proof.verify_one(root/case/mode/"128",case,mode,source_root,
                           seed_sha,None,128,128)
    resumed=proof.verify_one(root/case/mode/"512",case,mode,source_root,
                             seed_sha,first,512,384)
    reference=proof.verify_one(root/case/mode/"whole512",case,mode,source_root,
                               seed_sha,None,512,512)
    need(first["status"]=="RUNNING" and resumed["status"]==reference["status"],
         "source-bound nine cat original BTC SAT state changed")
    need(first["met"]["nodes"]==128 and resumed["met"]["nodes"]==512,
         "original Bitcoin nine-cat cumulative visits did not reach 512")
    for key in ("trace","trail","stack","source_sha","seed_sha","order_sha"):
        need(resumed[key]==reference[key],
             "nine-cat full pending DPLL frontier differs: "+key)
    identical=("nodes","propagations","literal_checks","decisions",
               "first_order","first_selected","projected_rooms",
               "projected_bytes","pixel_reads","reconstructed_rooms",
               "source_trit_reads")
    for key in identical:
        need(resumed["met"][key]==reference["met"][key],
             "nine-cat source exact work/replay differs: "+key)
    m=resumed["met"]
    need(m["projected_bytes"]==18*m["projected_rooms"]
         and m["pixel_reads"]==9*m["projected_rooms"]
         and m["source_trit_reads"]==6*m["projected_rooms"]
         and m["reconstructed_rooms"]==0,
         "nine-cat operation charges fail exact source accounting")
    need(m["source_index_bytes"]==2*vars,
         "two hosted source-index bank constructions not both paid")
    need(reference["met"]["source_index_bytes"]==vars,
         "uninterrupted source-index paid incorrectly")
    return {
        "case":case,
        "state":resumed["status"],
        "nodes":m["nodes"],
        "literal_checks":m["literal_checks"],
        "decisions":m["decisions"],
        "first_fischer":m["first_order"],
        "first_nine":m["first_selected"],
        "observer_windows":m["projected_rooms"],
        "nine_observations":m["pixel_reads"],
        "paid_ephemeral_projection_bytes":m["projected_bytes"],
        "source_signed_reads":m["source_trit_reads"],
        "held_signed_source_index_bytes_split":m["source_index_bytes"],
        "held_signed_source_index_bytes_whole":reference["met"]["source_index_bytes"],
        "frontier_sha256":resumed["sha"],
        "trace_sha256":resumed["trace"],
        "parity":"EXACT",
        "vs_direct_literal_checks":m["literal_checks"]-BASE[case]["direct"],
        "vs_one_literal_checks":m["literal_checks"]-BASE[case]["one"],
        "vs_three_literal_checks":m["literal_checks"]-BASE[case]["three"]
    }

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--source",type=Path,required=True)
    ap.add_argument("--gguf",type=Path,required=True)
    ap.add_argument("--root",type=Path,required=True)
    ap.add_argument("--out",type=Path,required=True)
    args=ap.parse_args()
    rows=[compare(case,args.source,args.gguf,args.root)
          for case in ("nonce32","merkle_nonce288")]
    lines=[]
    for r in rows:
        lines.append(
            "BTC_NINE_CATS_DPLL|case={case}|status={state}|nodes={nodes}|"
            "literal_checks={literal_checks}|decisions={decisions}|"
            "first_fischer={first_fischer}|first_nine={first_nine}|"
            "observer_windows={observer_windows}|"
            "nine_observations={nine_observations}|"
            "paid_shadow_bytes={paid_ephemeral_projection_bytes}|"
            "source_signed_reads={source_signed_reads}|"
            "held_signed_bank_bytes_split={held_signed_source_index_bytes_split}|"
            "held_signed_bank_bytes_whole={held_signed_source_index_bytes_whole}|"
            "delta_vs_direct_checks={vs_direct_literal_checks}|"
            "delta_vs_one_checks={vs_one_literal_checks}|"
            "delta_vs_three_checks={vs_three_literal_checks}|"
            "frontier_sha256={frontier_sha256}|trace_sha256={trace_sha256}|"
            "split_vs_uninterrupted={parity}|json=0".format(**r)
        )
    data="\n".join(lines)+"\n"
    report=("BTC_NINE_CATS_DPLL_FINAL|schema=1|original_sha256d_cases=2|"
            "nine_cats=9|source_index_paid=YES|"
            "split_nodes=128-512|fresh_reference=512|"
            "original_CNF_SAT=EXACT|split_uninterrupted_parity=PASS|"
            f"rows_sha256={sha(data.encode())}|status=PASS|json=0\n")
    args.out.write_text(report+data,"utf-8")
    print(report+data,end="",flush=True)

if __name__=="__main__":main()
