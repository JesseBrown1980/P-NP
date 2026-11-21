#!/usr/bin/env python3
"""Independent referee: reconstruct every black/white/brown nonce rank anew.

Does not import the experimental script or its feature extractor. All SHA256d
roots, byte alphabet inverses and 3-box×3-radius scores are recomputed here.
"""
import argparse
import hashlib
import json
from pathlib import Path

N=4096
KNOWN=2083236893
HIGH=KNOWN & ~4095
PREFIX=bytes.fromhex("01000000"+"00"*32+
 "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a"+
 "29ab5f49"+"ffff001d")
MODES={"black":0,"white":7,"brown":6}
def need(value,message):
    if not value:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def original_hash(low):
    s=PREFIX+(HIGH|low).to_bytes(4,"little")
    return hashlib.sha256(hashlib.sha256(s).digest()).digest()[::-1]
def white_form(data,mask):
    # Different operation sequence than main experimental producer.
    # Each transformation is self-inverse and they commute.
    if mask&4:
        result=[]
        for byte in data:
            b=0
            for half in range(2):
                nib=(byte>>(half*4))&15
                rev=0
                for shift in range(4):
                    rev|=((nib>>shift)&1)<<(3-shift)
                b|=rev<<(half*4)
            result.append(b)
        data=bytes(result)
    if mask&2:data=bytes(((x&15)<<4)|(x>>4) for x in data)
    if mask&1:data=bytes(reversed(data))
    return data
def digits(byte):
    d=[0]*6
    for j in range(5,-1,-1):
        byte,r=divmod(byte,3)
        d[j]=r-1
    return d
def digit_candidates():
    rows=[]
    for n in range(N):
        row=[]
        for _ in range(6):
            n,r=divmod(n,3);row.append(r-1)
        rows.append(row)
    return rows
CDIGITS=digit_candidates()
def independent_rank(root,mask,fischer,answer):
    white=white_form(root,mask)
    signed=[]
    for x in white:signed+=digits(x)
    ranked=[]
    for ix,n in enumerate(fischer):
        row=CDIGITS[n]
        accum_f=accum_b=accum_c=0
        score=0
        for r in (0,1,2):
            f=((row[r]+1)+(signed[r]+1))%3-1
            b=((row[r+3]+1)+(signed[r+9]+1))%3-1
            accum_f+=f
            accum_b+=b
            accum_c+=f*b
            score+=abs(accum_f)+abs(accum_b)+abs(accum_c)
        ranked.append((-score,ix,n))
    ranked.sort()
    return next(i for i,x in enumerate(ranked,1) if x[2]==answer)
def fixtures28():
    out=[KNOWN&4095];i=0
    while len(out)<28:
        n=int.from_bytes(hashlib.sha256(f"P-NP-Tail-heldout-2026-{i}".encode()).digest()[:2],"big")%N
        if n not in out:out.append(n)
        i+=1
    return out
def fixtures256():
    out=[];seen=set();i=0
    while len(out)<256:
        n=int.from_bytes(hashlib.sha256(f"PNP-white-first-independent-heldout-20261121-{i}".encode()).digest()[:4],"big")%N
        if n not in seen:out.append(n);seen.add(n)
        i+=1
    return out
def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--source",type=Path,required=True)
    ap.add_argument("--out",type=Path,required=True)
    args=ap.parse_args()
    entry=json.loads(args.source.read_text(encoding="utf-8"))
    need(entry["schema"]=="BITCOIN_WHITE_FIRST_TAIL_LEDGER_V1","unknown original Bitcoin experiment schema")
    need(entry["status"]=="PASS","producer did not report executed PASS")
    need(entry["source_sha256"]==sha(PREFIX+KNOWN.to_bytes(4,"little")),"original source bytes changed")
    need(entry["target_sha256d"]=="000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
         "bitcoin target digest no longer matches canonical genesis")
    roots=[original_hash(i) for i in range(N)]
    need(len(set(roots))==N,"nonce-domain collision")
    need(roots[KNOWN&4095].hex()==entry["target_sha256d"],"real double-SHA256 doesn't authenticate")
    for mask in range(8):
        seen=set()
        for root in roots:
            view=white_form(root,mask)
            need(white_form(view,mask)==root,"loss in original 256 alphabet translation")
            seen.add(view)
        need(len(seen)==N,"original SHA256d white-first collision created")
    fischer=sorted(range(N),key=lambda i:(pow(7,i,1000081),i))
    position={n:i+1 for i,n in enumerate(fischer)}
    signatures={}
    for section,targets in [("primary28",fixtures28()),("heldout256",fixtures256())]:
        witness=entry[section]
        need(witness["target_count"]==len(targets)
             and len(witness["rows"])==len(targets),"held-out target count changed")
        total={k:0 for k in ("sequential","fischer","black","white","brown")}
        for row,nonce in zip(witness["rows"],targets):
            need(row["nonce_suffix_verifier_only"]==nonce,"known target injected or changed")
            need(row["target_sha256"]==sha(roots[nonce]),"target binding differs from real SHA256d")
            need(row["fischer"]==position[nonce] and row["sequential"]==nonce+1,
                 "independent primary original Fischer order or sequential rank changed")
            total["fischer"]+=position[nonce];total["sequential"]+=nonce+1
            for label,mask in MODES.items():
                paid=row["orientations"][label]
                expected=independent_rank(roots[nonce],mask,fischer,nonce)
                need(expected==paid["rank"],"independent three-box torus score ranking differs")
                need(paid["mask"]==mask
                     and paid["candidate_nine_cat_observations"]==N*9
                     and paid["candidate_six_link_reads"]==N*6
                     and paid["potential_materialized_observer_bytes"]==N*18,
                     "Brown's Ledger understated source observation costs")
                need(paid["actual_sha256d_watcher_executed"]==(section=="primary28"),
                     "hash-only witness execution status overclaimed")
                total[label]+=expected
        need(total==witness["totals"],"independent held-out totals mismatch")
        signatures[section]=total
    need(signatures["primary28"]==
         {"sequential":66175,"fischer":46430,"black":52517,"white":48318,"brown":46716},
         "primary original Bitcoin result changed")
    need(signatures["heldout256"]==
         {"sequential":502268,"fischer":518341,"black":528703,"white":528483,"brown":510348},
         "256 additional held-outs changed")
    cost=entry["brown_ledger"]
    need(cost["retained_three_binary_bank_bytes"]==768
         and cost["retained_three_glyph_banks_bytes"]==4608
         and cost["path2_CRT_residues_bytes"]==3
         and cost["inverted_lookup_table_precompute_hash_calls"]==N
         and cost["inverted_lookup_table_minimum_raw_digest_plus_nonce_bytes"]==N*34
         and cost["per_target_transient_nine_cat_projection_bytes"]==N*18,
         "incomplete source/CRT/orbit/index payment")
    need(entry["bit_information"]["hash_output_bits"]==256
         and entry["bit_information"]["alphabet_cardinality"]==256
         and entry["bit_information"]["hash_state_count"]=="2^256",
         "confused 256-symbol alphabet with 256-bit output")
    text=("BTC_WHITE_FIRST_REFEREE|schema=1|original_bitcoin_genesis_SHA256d=PASS|"
          "full_4096_hash_orbit_inversion=PASS|"
          "black_white_brown_28_recomputed=PASS|"
          "black_white_brown_256_recomputed=PASS|"
          "source_bank_shannon_cost=PAID|"
          "hash_only_32bit_preimage=NOT_CLAIMED|status=PASS|json=0\n")
    args.out.write_text(text,encoding="utf-8")
    print(text,end="",flush=True)
if __name__=="__main__":main()
