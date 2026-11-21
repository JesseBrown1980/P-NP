#!/usr/bin/env python3
"""Bitcoin white-first tail funnel: source-paid three paths and 3×3 cats.

Original public Bitcoin Genesis 80-byte header with its high 20 nonce bits
fixed. Unknown suffix = 12 bits, not an unrestricted 32/256-bit preimage.
Eight black/white/brown alphabet views are 3 commuting byte permutations.
An exact inverse changes representation, not SHA256d information content.
Source-known Path1 and CRT Path2 are never counted as hash-only recovery.
No third-party modules; CPU source, not quantum/GPU hardware.
"""
import argparse
import hashlib
import json
import math
import time
from pathlib import Path

N=4096
PRIME=1_000_081
KNOWN_NONCE=2083236893
HIGH=KNOWN_NONCE & ~(N-1)
PREFIX=bytes.fromhex("01000000"+"00"*32+
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a"+
    "29ab5f49"+"ffff001d")
GENESIS_TAIL="000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
EXPECTED_28={'sequential':66175,'fischer':46430,'black':52517,'white':48318,'brown':46716}
EXPECTED_256={'sequential':502268,'fischer':518341,'black':528703,'white':528483,'brown':510348}
VIEWS={'black':0,'white':7,'brown':6}
def sha(data):return hashlib.sha256(data).hexdigest()
def digest(data):return hashlib.sha256(hashlib.sha256(data).digest()).digest()[::-1]
def need(condition,message):
    if not condition: raise ValueError(message)
def header(low):return PREFIX+(HIGH|low).to_bytes(4,"little")
def byte_trits(byte):
    result=[]
    for _ in range(6):
        byte,remainder=divmod(byte,3)
        result.append(remainder-1)
    need(byte==0,"noncanonical 6-trit byte")
    return list(reversed(result))
def untrits(digits):
    n=0
    for t in digits:n=3*n+t+1
    need(0<=n<256,"noncanonical balanced-ternary byte")
    return n
def wave(trit):return (trit+2)%3-1
def nib_rev(v):return ((v&1)<<3)|((v&2)<<1)|((v&4)>>1)|((v&8)>>3)
def alphabet(data,mask):
    if mask&1: data=data[::-1]
    if mask&2: data=bytes((x>>4)|((x&15)<<4) for x in data)
    if mask&4: data=bytes((nib_rev(x>>4)<<4)|nib_rev(x&15) for x in data)
    return data
def order_fischer():
    return sorted(range(N),key=lambda i:(pow(7,i,PRIME),i))
def candidate_digits(n):
    out=[]
    for _ in range(6):
        n,t=divmod(n,3);out.append(t-1)
    return out
CANDIDATES=[candidate_digits(x) for x in range(N)]
def cat_score(value,tail):
    # Experimental three boxes x three radii with six signed contacts.
    c=CANDIDATES[value]
    f=b=v=score=0
    for k in range(3):
        # Exactly the same order-three ternary phase algebra as source bank.
        x=(c[k]+tail[k]+2)%3-1
        y=(c[k+3]+tail[k+9]+2)%3-1
        f+=x;b+=y;v+=x*y
        score+=abs(f)+abs(b)+abs(v)
    return score
def rank_nine(tail,fischer):
    trits=[x for byte in tail for x in byte_trits(byte)]
    ranking=[]
    for i,nonce in enumerate(fischer):
        ranking.append((-cat_score(nonce,trits),i,nonce))
    ranking.sort()
    return [row[2] for row in ranking]
def path1(low):
    data=header(low);stored=data+b"\0"*(512-len(data))
    a,b=stored[:256],stored[256:]
    p=bytes(x^y for x,y in zip(a,b))
    need((a+b)[:80]==data,"AB failed")
    need((a+bytes(x^y for x,y in zip(a,p)))[:80]==data,"AP failed")
    need((bytes(x^y for x,y in zip(b,p))+b)[:80]==data,"BP failed")
    return 768,4608
def path2(low):
    mod=(17,19,23); M=math.prod(mod)
    a=[low%m for m in mod]
    value=sum(r*(M//m)*pow(M//m,-1,m) for r,m in zip(a,mod))%M
    need(M>4096 and value==low,"source-paid CRT residue reconstruction fails")
    return tuple(a)
def heldout28():
    out=[KNOWN_NONCE&(N-1)];i=0
    while len(out)<28:
        n=int.from_bytes(hashlib.sha256(f"P-NP-Tail-heldout-2026-{i}".encode()).digest()[:2],"big")%N
        if n not in out:out.append(n)
        i+=1
    return out
def heldout256():
    out=[];seen=set();i=0
    while len(out)<256:
        n=int.from_bytes(hashlib.sha256(f"PNP-white-first-independent-heldout-20261121-{i}".encode()).digest()[:4],"big")%N
        if n not in seen:out.append(n);seen.add(n)
        i+=1
    return out
def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--out",type=Path,required=True)
    args=parser.parse_args()
    begin=time.perf_counter_ns()
    need(len(PREFIX)==76,"public original Genesis header context width")
    need(digest(header(KNOWN_NONCE & (N-1))).hex()==GENESIS_TAIL,
         "Bitcoin genesis SHA256d target mismatch")
    roots=[digest(header(n)) for n in range(N)]
    need(len(set(roots))==N,"collision in bounded authentic Bitcoin search")
    for value in range(256):
        for mask in range(8):
            need(alphabet(alphabet(bytes([value]),mask),mask)==bytes([value]),
                 "incorrect 256-symbol reversible translation")
    for mask in range(8):
        need(all(alphabet(alphabet(v,mask),mask)==v for v in roots),
             "real Bitcoin SHA256d eight-point inversion mismatch")
    source_bytes=header(KNOWN_NONCE & (N-1))
    for data,whole in [(source_bytes,486),(roots[KNOWN_NONCE&4095],216)]:
        digits=[x for byte in data for x in byte_trits(byte)]
        need(len(digits)==len(data)*6 and (len(digits)+26)//27*27==whole,
             "source/tail frozen RimeSphere padding width")
        for phase in range(3):
            moved=digits[:]
            for _ in range(phase):moved=[wave(x) for x in moved]
            for _ in range((3-phase)%3):moved=[wave(x) for x in moved]
            need(bytes(untrits(moved[i:i+6]) for i in range(0,len(moved),6))==data,
                 "ternary wave inverse lost actual Bitcoin source")
    fischer=order_fischer()
    need(len(set(fischer))==N,"prime-index collision in Fischer order")
    rank_fischer={n:i+1 for i,n in enumerate(fischer)}
    trial_sets={"primary28":heldout28(),"heldout256":heldout256()}
    receipts={}
    for tag,cases in trial_sets.items():
        rows=[];total={label:0 for label in ("sequential","fischer","black","white","brown")}
        for index,low in enumerate(cases):
            target=roots[low]
            row={"case":index,"target_sha256":sha(target),
                 "nonce_suffix_verifier_only":low,
                 "sequential":low+1,"fischer":rank_fischer[low],"orientations":{}}
            for label,mask in VIEWS.items():
                translated=alphabet(target,mask)
                t0=time.perf_counter_ns()
                choice_order=rank_nine(translated,fischer)
                compute_ns=time.perf_counter_ns()-t0
                need(len(set(choice_order))==N,"nine-cat rank incomplete")
                rank=choice_order.index(low)+1
                need(alphabet(roots[choice_order[rank-1]],mask)==translated,
                     "translated target matcher not source-exact")
                wall=0
                if tag=="primary28":
                    # Actually compute all forward SHA256d candidates until match;
                    # never provide low answer to selector or watcher.
                    t0=time.perf_counter_ns();checks=0;winner=None
                    for guess in choice_order:
                        checks+=1
                        if alphabet(digest(header(guess)),mask)==translated:
                            winner=guess;break
                    wall=time.perf_counter_ns()-t0
                    need(winner==low and checks==rank,"real Bitcoin hash witness failed")
                    need(path1(low)==(768,4608),"2-of-3 retained source bank payment")
                    path2(low) # source-derived advice is separately charged
                row["orientations"][label]={"mask":mask,"rank":rank,"setup_ns":compute_ns,
                    "actual_sha256d_watcher_ns":wall,
                    "actual_sha256d_watcher_executed":tag=="primary28",
                    "candidate_nine_cat_observations":N*9,
                    "candidate_six_link_reads":N*6,
                    "potential_materialized_observer_bytes":N*9*2}
                total[label]+=rank
            total["sequential"]+=low+1;total["fischer"]+=rank_fischer[low]
            rows.append(row)
        expected=EXPECTED_28 if tag=="primary28" else EXPECTED_256
        need(total==expected, f"{tag} exact original Bitcoin reference ranks changed")
        receipts[tag]={"target_count":len(cases),"totals":total,
                       "mean_rank":{key:v/len(cases) for key,v in total.items()},
                       "rows":rows}
    result={
      "schema":"BITCOIN_WHITE_FIRST_TAIL_LEDGER_V1","status":"PASS",
      "source":"original public Bitcoin genesis header; high 20 nonce bits fixed; 12 bits unknown",
      "source_sha256":sha(source_bytes),"target_sha256d":GENESIS_TAIL,
      "bit_information":{"alphabet_cardinality":256,"bits_per_byte":8,"hash_output_bits":256,
          "possible_full_hashes":"2^256","actual_test_domain":N,
          "bijective_hash_translation_changes_entropy":False},
      "representations":VIEWS,"original_eight_point_involutions":"PASS",
      "all_256_symbols_and_4096_original_hashes_per_mask_roundtrip":"PASS",
      "three_phase_ternary_inverse":"PASS",
      "rimesphere":{"source_i8_trits_with_padding":486,"tail_i8_trits_with_padding":216},
      "brown_ledger":{
          "retained_three_binary_bank_bytes":768,
          "retained_three_glyph_banks_bytes":4608,
          "path2_CRT_residues_bytes":3,
          "path2_provides_source_derived_nonce_suffix_information":True,
          "path3_hash_only_fixed_public_header_context_bytes":80,
          "tail_bytes":32,
          "all_4096_hashes_computed_for_index":True,
          "inverted_lookup_table_precompute_hash_calls":4096,
          "inverted_lookup_table_minimum_raw_digest_plus_nonce_bytes":4096*34,
          "per_target_transient_nine_cat_projection_bytes":N*9*2,
          "materialized_per_variant_full_tail_bytes":32,
          "shared_dictionary_source_code_bytes":Path(__file__).stat().st_size,
          "decoder_and_code_paid_if_deployed":True,
          "genuine_unrestricted_32bit_nonce_or_256bit_preimage_recovery":False
      },
      "primary28":receipts["primary28"],
      "heldout256":receipts["heldout256"],
      "test_process_wall_ns":time.perf_counter_ns()-begin
    }
    args.out.parent.mkdir(parents=True,exist_ok=True)
    args.out.write_text(json.dumps(result,sort_keys=True,indent=2)+"\n",encoding="utf-8")
    print("BITCOIN_WHITE_FIRST|status=PASS|primary28=SHA256D_EXECUTED|"+
          "heldout256=EXACT_RANK|eight_orbits=PASS|wave_inverse=PASS|"+
          "brown_ledger=PAID|"+
          "|".join(f"{x}_calls28={receipts['primary28']['totals'][x]}" for x in ("fischer","black","white","brown"))+
          "|json=0",flush=True)
if __name__=="__main__":main()
