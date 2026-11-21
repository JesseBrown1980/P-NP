#!/usr/bin/env python3
"""Independent, source-bound DPLL frontier auditor for Bitcoin SAT continuity.

Decodes Rust PNPFRT01 without importing any native Rust serializer/solver.
Checks complete assignment-trail and backtracking-frame integrity, SHA256
binding and checkpoint links across separate hosted workers. An uninterrupted
128-node reference must exactly match the 8+24+96 restart trace, decisions,
SAT status, and original-CNF work counts. Source acceptance is inherited from
the independently accepted original Bitcoin GGUF and three-shadow runs.
"""
import argparse
import ast
import hashlib
from pathlib import Path
import struct

import bitcoin_gguf_fischer_referee as gguf
import bitcoin_live_sat_referee as btc

CASES=("nonce32","merkle_nonce288")
MODES=("direct","one","three")
WIDTHS=(8,32,128)
QUANTA=(8,24,96)
HEADER=8+4+64*4+2+8*14+8
REGRESSION={
    ("nonce32","direct"): (124075,124075),
    ("nonce32","one"): (124075,124075),
    ("nonce32","three"): (124075,11030),
    ("merkle_nonce288","direct"): (244087,244087),
    ("merkle_nonce288","one"): (244087,114107),
    ("merkle_nonce288","three"): (244087,10142),
}
def sha(data): return hashlib.sha256(data).hexdigest()
def need(ok, why):
    if not ok: raise ValueError(why)

class Reader:
    def __init__(self, data): self.data, self.at = data, 0
    def take(self, n):
        need(n>=0 and self.at+n<=len(self.data), "truncated DPLL frontier")
        x=self.data[self.at:self.at+n]; self.at+=n
        return x
    def u8(self): return self.take(1)[0]
    def u32(self): return struct.unpack("<I",self.take(4))[0]
    def u64(self): return struct.unpack("<Q",self.take(8))[0]
    def digest(self):
        x=self.take(64).decode("ascii")
        need(len(x)==64 and all(c in "0123456789abcdefABCDEF" for c in x),
             "noncanonical DPLL SHA256 digest")
        return x

def parse_checkpoint(path,variables,clauses,source,mode,seed_sha):
    raw=path.read_bytes()
    need(len(raw)>=HEADER+64 and raw[:8]==b"PNPFRT01",
         "unrecognized exact frontier checkpoint")
    need(sha(raw[:-64])==raw[-64:].decode("ascii"),
         "checkpoint independently computed SHA256 mismatch")
    r=Reader(raw[:-64])
    need(r.take(8)==b"PNPFRT01" and r.u32()==1,
         "frontier schema/version changed")
    source_sha,order_sha,seed,trace = [r.digest() for _ in range(4)]
    modebyte,status=[r.u8(),r.u8()]
    need(source_sha==source and seed==seed_sha and modebyte==MODES.index(mode),
         "checkpoint original Bitcoin source, seed or shadow mode changed")
    need(status in (0,1,2),"bad DPLL source state")
    names=("vars","clauses","nodes","propagations","literal_checks",
           "decisions","first_order","first_selected","projected_rooms",
           "projected_bytes","pixel_reads","reconstructed_rooms",
           "source_trit_reads","source_index_bytes")
    met={k:r.u64() for k in names}
    need(met["vars"]==variables and met["clauses"]==clauses,
         "checkpoint original source dimensions changed")
    trail_count,frame_count=r.u32(),r.u32()
    need(trail_count<=variables and frame_count<=trail_count,
         "impossible source SAT trail/frame lengths")
    known={}
    trail=[]
    for _ in range(trail_count):
        var,sign=r.u32(),r.u8()
        need(1<=var<=variables and sign in (1,255) and var not in known,
             "checkpoint original SAT variable duplicated or polarity malformed")
        known[var]=sign
        trail.append((var,sign))
    stack=[]
    for _ in range(frame_count):
        var,mark,minus=r.u32(),r.u32(),r.u8()
        need(1<=var<=variables and mark<trail_count and minus in (0,1),
             "invalid source SAT branch-frame address")
        need(trail[mark]==(var,255 if minus else 1),
             "frontier branch polarity differs from preserved assignment trail")
        need(not stack or stack[-1][1]<mark,
             "checkpoint nested backtracking frames not ordered")
        stack.append((var,mark,minus))
    need(r.at==len(raw)-64,"undeclared trailing frontier checkpoint data")
    need(met["nodes"]>=met["decisions"] and
         met["propagations"]>=trail_count,
         "impossible original-CNF work counters")
    if status==2:
        need(not stack,"UNSAT record retains live undiscovered branches")
    if mode=="direct":
        for k in ("projected_rooms","projected_bytes","pixel_reads",
                  "reconstructed_rooms","source_trit_reads","source_index_bytes"):
            need(met[k]==0,"direct search performed shadow work")
    else:
        need(met["projected_rooms"]>0 or met["nodes"]==0,
             "no actual SAT shadow projections counted")
        need(met["projected_bytes"]==
             (9 if mode=="one" else 18 if mode=="nine" else 35)*met["projected_rooms"],
             "shadow byte cost differs from actual signed 2D representation")
        need(met["reconstructed_rooms"]==
             (met["projected_rooms"] if mode=="three" else 0),
             "8-trit reconstruction witness not counted")
    return dict(
        path=str(path), sha=sha(raw), mode=mode,status=("RUNNING","SAT","UNSAT")[status],
        source_sha=source_sha,order_sha=order_sha,seed_sha=seed,
        trace=trace, trail=trail, stack=stack, raw=raw, met=met
    )

def fields(path):
    lines=path.read_text("ascii").splitlines()
    need(len(lines)==1 and lines[0].startswith("BTC_FRONTIER_SESSION|"),
         "missing canonical single-line frontier receipt")
    pairs=[part.split("=",1) for part in lines[0].split("|")[1:]]
    need(all(len(p)==2 for p in pairs),"invalid DPLL receipt field")
    data=dict(pairs)
    need(len(data)==len(pairs),"duplicate frontier receipt key")
    return data

def source_model(case,source_root,folder,row):
    expected,variables,clauses=btc.CASES[case]
    source=(folder/"MODEL.trits").read_bytes()
    need(len(source)==variables+1 and source.endswith(b"\n")
         and all(b in (43,45) for b in source[:-1])
         and sha(source)==row["model_sha256"],"invalid exact SAT proof model")
    selected={i+1 for i,b in enumerate(source[:-1]) if b==43}
    btc.verify_cnf_model(source_root/(case+".cnf"),selected,variables,clauses)
    manifest=ast.literal_eval((source_root/(case+".txt")).read_text("ascii"))
    mask=(1<<manifest["nonce_free_bits"])-1
    low=sum(1<<bit for bit,var in manifest["nonce_variables"] if var in selected)
    nonce=(btc.NONCE & ~mask)|low
    header=bytearray(btc.PREFIX+nonce.to_bytes(4,"little"))
    if manifest["merkle_free_bits"]:
        merkle=sum(1<<bit for bit,var in manifest["merkle_variables"] if var in selected)
        header[36:68]=merkle.to_bytes(32,"little")
    need(btc.double_sha(header)==btc.TARGET,
         "claimed completed Bitcoin SAT proof fails full SHA256d")

def verify_one(folder,case,mode,source_root,seed_sha,prior=None,cap=8,quantum=8):
    expected,variables,clauses=btc.CASES[case]
    checkpoint=parse_checkpoint(folder/"FRONTIER.frt",
        variables,clauses,expected,mode,seed_sha)
    receipt=fields(folder/"SESSION.hbp")
    for key,value in (
        ("schema","1"),("case",case),("mode",mode),
        ("state",checkpoint["status"]),("original_source_sha256",expected),
        ("original_variables",str(variables)),("original_clauses",str(clauses)),
        ("session_node_quota",str(quantum)),
        ("previous_checkpoint_sha256",prior["sha"] if prior else "NEW"),
        ("previous_nodes",str(prior["met"]["nodes"]) if prior else "0"),
        ("original_cnf","EXACT"),("known_nonce_injected","NO"),
        ("algorithmic_timer","NONE"),("work_quantum","CHECKPOINT_BOUNDARY"),
        ("frontier_sha256",checkpoint["sha"]),
        ("frontier_bytes",str(len(checkpoint["raw"]))),
        ("trace_sha256",checkpoint["trace"]),("json","0")
    ):
        need(receipt.get(key)==value,
             "full-source frontier receipt/checkpoint mismatch: "+key)
    m=checkpoint["met"]
    keys=("nodes","propagations","literal_checks","decisions",
          "first_order","first_selected","projected_rooms","projected_bytes",
          "pixel_reads","reconstructed_rooms","source_trit_reads","source_index_bytes")
    receipt_keys=("nodes","propagations","literal_checks","branch_decisions",
                  "first_fischer","first_selected","projected_rooms","projected_bytes",
                  "shadow_pixel_reads","reconstructed_rooms","shadow_source_reads",
                  "shadow_index_bytes")
    for key,field in zip(keys,receipt_keys):
        need(receipt.get(field)==str(m[key]),
             "independent frontier source-work meter differs: "+field)
    previous=prior["met"]["nodes"] if prior else 0
    visited=m["nodes"]-previous
    need(0<=visited<=quantum and
         receipt.get("new_nodes")==str(visited) and
         m["nodes"]<=cap,
         "original Bitcoin DPLL quantum replayed or skipped source nodes")
    need(receipt.get("prev_trace_sha256")==
         (prior["trace"] if prior else sha(
             (f"PNP_DPLL_FRONTIER_BEGIN_V1|{expected}|"
              f"{checkpoint['order_sha']}|{seed_sha}|{mode}").encode())),
         "frontier progress chain did not resume from actual prior state")
    if prior:
        need(prior["source_sha"]==checkpoint["source_sha"]
             and prior["order_sha"]==checkpoint["order_sha"]
             and prior["seed_sha"]==checkpoint["seed_sha"]
             and prior["mode"]==checkpoint["mode"],
             "frontier original-source binding changed across runner")
    for key in ("cnf_read_ns","gguf_read_ns","source_route_ns",
                "frontier_replay_ns","solver_work_ns","frontier_freeze_ns",
                "session_total_ns"):
        need(receipt.get(key,"").isdigit(),"missing session wall accounting: "+key)
    need(int(receipt["session_total_ns"])>=int(receipt["solver_work_ns"]),
         "per-session solver cost exceeds total")
    need(receipt.get("model_sha256")=="NONE" or checkpoint["status"]=="SAT",
         "unproven SAT model appeared on resumable frontier")
    if checkpoint["status"]=="SAT":
        source_model(case,source_root,folder,receipt)
    else:
        need(receipt.get("model_sha256")=="NONE"
             and not (folder/"MODEL.trits").exists(),
             "uncompleted Bitcoin frontier claimed a valid model")
    need((folder/"stdout.log").is_file()
         and (folder/"stderr.log").is_file()
         and (folder/"worker_time.hbp").is_file(),
         "missing raw computer-session evidence")
    return checkpoint

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--source",type=Path,required=True)
    ap.add_argument("--gguf",type=Path,required=True)
    ap.add_argument("--root",type=Path,required=True)
    ap.add_argument("--out",type=Path,required=True)
    args=ap.parse_args()
    for case in CASES:
        expected,variables,clauses=btc.CASES[case]
        original=(args.source/(case+".cnf")).read_bytes()
        need(sha(original)==expected,"original Bitcoin full CNF SHA changed")
        frozen=(args.gguf/case/"BITCOIN.three_shadows.gguf").read_bytes()
        full,seed,meta=gguf.parse(frozen)
        need(full==original and seed==bytes(variables)
             and meta["pnp.original_cnf_sha256"]==expected,
             "accepted three-shadow GGUF not original source/all-zero root")
    lines=[]
    for case in CASES:
        expected,variables,_=btc.CASES[case]
        seed_sha=sha(bytes(variables))
        for mode in MODES:
            prior=None
            for cap,quantum in zip(WIDTHS,QUANTA):
                folder=args.root/case/mode/str(cap)
                candidate=verify_one(folder,case,mode,args.source,
                    seed_sha,prior,cap,quantum)
                prior=candidate
                need(candidate["met"]["nodes"]==cap or candidate["status"]!="RUNNING",
                     "original Bitcoin work quantum did not visit all allowed new nodes")
            complete=verify_one(args.root/case/mode/"whole128",case,mode,
                args.source,seed_sha,None,128,128)
            for key in ("state","trace","trail","stack"):
                if key=="state":
                    need(prior["status"]==complete["status"],
                         "split/restart SAT completion status differs")
                else:
                    need(prior[key]==complete[key],
                         "full DPLL split/restart source frontier differs: "+key)
            for key in ("nodes","propagations","literal_checks",
                        "decisions","first_order","first_selected",
                        "projected_rooms","projected_bytes","pixel_reads",
                        "reconstructed_rooms","source_trit_reads"):
                need(prior["met"][key]==complete["met"][key],
                     "real Bitcoin split/restart exact DPLL work differs: "+key)
            first_expected=REGRESSION[case,mode]
            need((prior["met"]["first_order"],prior["met"]["first_selected"])==
                 first_expected,"pilot-confirmed original Bitcoin first Fischer/shadow choices changed")
            lines.append(
                f"BTC_FRONTIER_CASE|case={case}|mode={mode}|"
                f"status={prior['status']}|nodes={prior['met']['nodes']}|"
                f"decisions={prior['met']['decisions']}|"
                f"literal_checks={prior['met']['literal_checks']}|"
                f"projected_rooms={prior['met']['projected_rooms']}|"
                f"first_order={first_expected[0]}|first_selected={first_expected[1]}|"
                f"trace_sha256={prior['trace']}|"
                "split_8_32_128=EXACT|uninterrupted_128=EXACT|json=0"
            )
    output="\n".join(lines)+"\n"
    report=(
        "BTC_FRONTIER_FINAL|schema=1|original_cases=2|modes=3|"
        "completed_comparisons=6|restart_stages=8-32-128|"
        "separate_runner_sessions=YES|algorithmic_timer=NONE|"
        "complete_frontier_snapshot=SHA256_BOUND|"
        f"rows_sha256={sha(output.encode())}|status=PASS|json=0\n"
    )
    args.out.write_text(report+output,"ascii")
    print(report+output,end="",flush=True)

if __name__=="__main__":
    main()
