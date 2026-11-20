#!/usr/bin/env python3
"""Independent 128→256→512 Bitcoin DPLL frontier continuation audit.

Read the last *accepted* 128-node checkpoint from hosted run 44751204056;
authenticate all saved assignments, previously tried branch polarities,
original CNF/GGUF source and receipts. Verify new 256/512 stages link to
those exact bytes, then compare an unrelated fresh uninterrupted 512-node
reference for every Bitcoin source/mode.

The original independent checkpoint parser/referee is reused; the new
acceptance is independently calculated and does not import Rust.
"""
import argparse
import hashlib
from pathlib import Path

import bitcoin_frontier_referee as frontier
import bitcoin_gguf_fischer_referee as gguf
import bitcoin_live_sat_referee as btc

CASES = ("nonce32", "merkle_nonce288")
MODES = ("direct", "one", "three")
COUNTS = (128, 256, 512)

def sha(data):
    return hashlib.sha256(data).hexdigest()

def need(ok, why):
    if not ok:
        raise ValueError(why)

def independent(case, mode, source_dir, gguf_dir, prior_dir, result_dir):
    expected, variables, clauses = btc.CASES[case]
    original = (source_dir / (case + ".cnf")).read_bytes()
    need(sha(original) == expected, "original full Bitcoin CNF hash changed")
    archive = (gguf_dir / case / "BITCOIN.three_shadows.gguf").read_bytes()
    restored_source, state, gguf_metadata = gguf.parse(archive)
    need(restored_source == original and state == bytes(variables)
         and gguf_metadata["pnp.original_cnf_sha256"] == expected,
         "original Bitcoin source/GGUF was not authenticated")
    seed_sha = sha(bytes(variables))
    prior = frontier.parse_checkpoint(
        prior_dir / case / mode / "128" / "FRONTIER.frt",
        variables, clauses, expected, mode, seed_sha
    )
    need(prior["met"]["nodes"] == 128 and prior["status"] == "RUNNING",
         "previously accepted original Bitcoin source frontier has wrong node count")
    need((prior["met"]["first_order"], prior["met"]["first_selected"])
         == frontier.REGRESSION[(case, mode)],
         "previously accepted Bitcoin 128-node first branch changed")
    # Each new stage is a *different* process and a different hosted job,
    # continuing from byte-exact prior frontiers, not from all-zero seeds.
    c256 = frontier.verify_one(
        result_dir / case / mode / "256", case, mode,
        source_dir, seed_sha, prior=prior, cap=256, quantum=128
    )
    c512 = frontier.verify_one(
        result_dir / case / mode / "512", case, mode,
        source_dir, seed_sha, prior=c256, cap=512, quantum=256
    )
    whole = frontier.verify_one(
        result_dir / case / mode / "whole512", case, mode,
        source_dir, seed_sha, prior=None, cap=512, quantum=512
    )
    need(c256["met"]["nodes"] == 256 or c256["status"] != "RUNNING",
         "256-node stage unexpectedly ended without a result")
    need(c512["met"]["nodes"] == 512 or c512["status"] != "RUNNING",
         "512-node stage unexpectedly ended without a result")
    need(c512["status"] == whole["status"],
         "full Bitcoin 512-node terminal status differs across restart")
    for name in ("source_sha", "seed_sha", "order_sha", "trace", "trail", "stack"):
        need(c512[name] == whole[name],
             "512-node complete frontier differs from uninterrupted: " + name)
    # Count native CNF visits, propagations, all literal reads, shadow
    # projections and independent original-source decision counts.
    # source_index_bytes is intentionally *not* equated: the resumed runs
    # reconstruct their source index each session and correctly charge it.
    for name in (
        "nodes", "propagations", "literal_checks", "decisions",
        "first_order", "first_selected",
        "projected_rooms", "projected_bytes", "pixel_reads",
        "reconstructed_rooms", "source_trit_reads",
    ):
        need(c512["met"][name] == whole["met"][name],
             "512-node exact source work differs across restart: " + name)
    need(c512["met"]["nodes"] >= prior["met"]["nodes"],
         "restarted original Bitcoin search lost committed work")
    return dict(
        case=case, mode=mode, status=c512["status"],
        nodes=c512["met"]["nodes"],
        literal_checks=c512["met"]["literal_checks"],
        decisions=c512["met"]["decisions"],
        propagated=c512["met"]["propagations"],
        projected_rooms=c512["met"]["projected_rooms"],
        projected_bytes=c512["met"]["projected_bytes"],
        source_index_bytes_restart=c512["met"]["source_index_bytes"],
        source_index_bytes_whole=whole["met"]["source_index_bytes"],
        first_fischer=c512["met"]["first_order"],
        first_selected=c512["met"]["first_selected"],
        frontier_sha=c512["sha"], trace=c512["trace"],
        accepted_128_sha=prior["sha"],
    )

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--gguf", type=Path, required=True)
    parser.add_argument("--prior", type=Path, required=True)
    parser.add_argument("--result", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args=parser.parse_args()
    results=[]
    for case in CASES:
        for mode in MODES:
            results.append(independent(case, mode, args.source, args.gguf,
                                       args.prior, args.result))
    need(len(results)==6, "original Bitcoin 512-node comparison not complete")
    lines=[]
    for entry in results:
        lines.append(
            "BTC_FRONTIER_512_CASE|case={case}|mode={mode}|"
            "status={status}|nodes={nodes}|literal_checks={literal_checks}|"
            "propagations={propagated}|decisions={decisions}|"
            "projected_rooms={projected_rooms}|projected_bytes={projected_bytes}|"
            "first_fischer={first_fischer}|first_selected={first_selected}|"
            "index_bytes_split={source_index_bytes_restart}|"
            "index_bytes_whole={source_index_bytes_whole}|"
            "prior128_sha256={accepted_128_sha}|"
            "frontier512_sha256={frontier_sha}|trace_sha256={trace}|"
            "restarted_parity=EXACT|json=0".format(**entry)
        )
    payload="\n".join(lines)+"\n"
    final=(
        "BTC_FRONTIER_512_FINAL|schema=1|cases=2|modes=3|groups=6|"
        "accepted_predecessor_nodes=128|next_nodes=256-512|"
        "fresh_uninterrupted_nodes=512|"
        "exact_branch_and_source_parity=PASS|"
        "algorithmic_timer=NONE|"
        f"case_receipts_sha256={sha(payload.encode())}|"
        "status=PASS|json=0\n"
    )
    args.out.write_text(final + payload, "ascii")
    print(final + payload, end="", flush=True)

if __name__=="__main__":
    main()
