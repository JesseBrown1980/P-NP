#!/usr/bin/env python3
"""Independent original-Bitcoin one-versus-three live SAT shadow referee.

All source and GGUF bytes come from previously independently accepted
Bitcoin RimeSphere and 3-shadow jobs. No solver or shadow Rust imports.
"""
import argparse
import ast
import hashlib
from pathlib import Path

import bitcoin_gguf_fischer_referee as gguf
import bitcoin_live_sat_referee as btc

MODES = ("direct", "one", "three")
CASES = ("nonce32", "merkle_nonce288")
CAP = 2

def need(condition, reason):
    if not condition:
        raise ValueError(reason)

def sha(data):
    return hashlib.sha256(data).hexdigest()

def fields(path):
    lines = path.read_text("ascii").splitlines()
    need(len(lines) == 1 and lines[0].startswith("BTC_THREE_SHADOW_SEARCH|"),
         "missing source-bound shadow SAT receipt")
    pairs = [x.split("=", 1) for x in lines[0].split("|")[1:]]
    need(all(len(pair) == 2 for pair in pairs), "non-field SAT entry")
    row = dict(pairs)
    need(len(row) == len(pairs), "duplicate shadow SAT meter")
    return row

def counts(row):
    keys = (
        "source_bytes", "gguf_bytes", "node_budget", "nodes",
        "propagations", "literal_checks", "first_order_candidate",
        "first_shadow_candidate", "first_shadow_score",
        "projection_choices", "projection_changes", "projected_rooms",
        "source_trit_reads", "shadow_pixel_reads", "projected_bytes",
        "reconstructed_rooms", "shadow_index_bytes", "shadow_index_ns",
        "shadow_choice_ns", "live_frames", "live_frozen_bytes",
        "live_wall_ns", "cnf_parse_ns", "gguf_read_ns",
        "fischer_route_ns", "solver_ns", "total_ns"
    )
    need(all(row.get(k, "").isdigit() for k in keys),
         "unmetered shadow SAT work")
    return {key: int(row[key]) for key in keys}

def test_completed_model(case, folder, original_source, row):
    expected, variables, clauses = btc.CASES[case]
    result = (folder / "model.trits").read_bytes()
    need(sha(result) == row.get("model_sha256")
         and len(result) == variables + 1 and result.endswith(b"\n")
         and all(b in (43, 45) for b in result[:-1]),
         "invalid completed SAT model")
    selected = {i + 1 for i, value in enumerate(result[:-1]) if value == 43}
    btc.verify_cnf_model(original_source / (case + ".cnf"), selected, variables, clauses)
    manifest = ast.literal_eval((original_source / (case + ".txt")).read_text("ascii"))
    nonce_mask = (1 << manifest["nonce_free_bits"]) - 1
    nonce_low = sum(1 << bit for bit, var in manifest["nonce_variables"]
                    if var in selected)
    nonce = (btc.NONCE & ~nonce_mask) | nonce_low
    header = bytearray(btc.PREFIX + nonce.to_bytes(4, "little"))
    if manifest["merkle_free_bits"]:
        merkle = sum(1 << bit for bit, var in manifest["merkle_variables"]
                     if var in selected)
        header[36:68] = merkle.to_bytes(32, "little")
    need(btc.double_sha(header) == btc.TARGET,
         "Bitcoin SHA256d full original target not achieved")

def verify(case, mode, source, result, accepted):
    expected, variables, clauses = btc.CASES[case]
    raw = (source / (case+".cnf")).read_bytes()
    need(sha(raw) == expected, "original full SHA256d CNF source changed")
    frozen = (accepted / case / "BITCOIN.three_shadows.gguf").read_bytes()
    original, states, meta = gguf.parse(frozen)
    need(original == raw and states == bytes(variables)
         and meta["pnp.original_cnf_sha256"] == expected,
         "three-shadow GGUF failed source/original SAT seed referee")
    folder = result / mode / case
    need((folder/"stdout.log").is_file()
         and (folder/"stderr.log").is_file()
         and (folder/"worker_time.hbp").is_file(),
         "missing raw original-Bitcoin shadow worker evidence")
    row = fields(folder/"receipt.hbp")
    for key, value in (
        ("schema", "1"), ("case", case), ("mode", mode),
        ("source_sha256", expected), ("gguf_sha256", sha(frozen)),
        ("variables", str(variables)), ("clauses", str(clauses)),
        ("shadow_policy", "SOURCE_SIGNED_OCCURRENCE_PLUS_LIVE_ASSIGNMENT"),
        ("artificial_timer", "NO"), ("known_nonce_injected", "NO"),
        ("SAT_proof", "ONLY_IF_SOURCE_MODEL_VALIDATED"),
        ("json", "0"),
    ):
        need(row.get(key) == value, "original Bitcoin shadow receipt mismatch: " + key)
    meter = counts(row)
    need(meter["node_budget"] == CAP and 0 <= meter["nodes"] <= CAP
         and meter["source_bytes"] == len(raw)
         and meter["gguf_bytes"] == len(frozen)
         and meter["total_ns"] >= meter["solver_ns"]
         and meter["total_ns"] >= meter["gguf_read_ns"],
         "original Bitcoin source or pilot budget/cost meters invalid")
    if mode == "direct":
        need(all(meter[k] == 0 for k in (
            "projection_choices", "projection_changes", "projected_rooms",
            "source_trit_reads", "shadow_pixel_reads", "projected_bytes",
            "reconstructed_rooms", "shadow_index_bytes",
            "shadow_index_ns", "shadow_choice_ns", "live_frames",
            "live_frozen_bytes", "live_wall_ns",
            "first_order_candidate", "first_shadow_candidate",
            "first_shadow_score"
        )), "direct baseline performed shadow work")
        need(row.get("shadow_decision_root") == "NONE"
             and row.get("live_root") == "NONE",
             "direct baseline has unexpected extra projection receipts")
    else:
        need(meter["projection_choices"] > 0
             and meter["projected_rooms"] > 0
             and meter["source_trit_reads"] > 0
             and meter["shadow_pixel_reads"] > 0
             and 1 <= meter["first_order_candidate"] <= variables
             and 1 <= meter["first_shadow_candidate"] <= variables
             and meter["live_frames"] > 0
             and meter["live_frozen_bytes"] > 0
             and len(row.get("live_root", "")) == 64
             and len(row.get("shadow_decision_root", "")) == 64,
             "projected original SAT choices never fed live exact DPLL")
        width = 9 if mode == "one" else 35
        need(meter["projected_bytes"] == width * meter["projected_rooms"],
             "missing actual per-room signed projection storage costs")
        need(meter["reconstructed_rooms"] ==
             (meter["projected_rooms"] if mode == "three" else 0),
             "three-shadow exact 8-trit inverse or one-shadow control not counted")
        need(meter["projection_changes"] <= meter["projection_choices"],
             "invalid first-Fischer shadow decision counts")
        need(meter["shadow_index_bytes"] == variables,
             "source-sign index bytes are not fully charged")
    need(row.get("status") in ("HOLD_NODE", "SAT"),
         "unsupported finished Bitcoin shadow SAT outcome")
    if row["status"] == "SAT":
        test_completed_model(case, folder, source, row)
    else:
        need(row.get("model_sha256") == "NONE"
             and not (folder/"model.trits").exists(),
             "node-limited search falsely claimed completed Bitcoin preimage")
    return {
        "case": case, "mode": mode, "status": row["status"],
        "nodes": meter["nodes"], "literal_checks": meter["literal_checks"],
        "first": meter["first_shadow_candidate"],
        "default": meter["first_order_candidate"],
        "choices": meter["projection_choices"],
        "shadow_ns": meter["shadow_choice_ns"],
        "solver_ns": meter["solver_ns"],
        "total_ns": meter["total_ns"],
        "gguf_sha": sha(frozen),
    }

def main():
    p = argparse.ArgumentParser()
    p.add_argument("--source", required=True, type=Path)
    p.add_argument("--accepted", required=True, type=Path)
    p.add_argument("--results", required=True, type=Path)
    p.add_argument("--out", required=True, type=Path)
    args = p.parse_args()
    baseline = btc.PREFIX + btc.NONCE.to_bytes(4, "little")
    need(btc.double_sha(baseline) == btc.TARGET,
         "original Bitcoin full genesis SHA256d reference differs")
    results = {}
    for case in CASES:
        for mode in MODES:
            results[case,mode] = verify(
                case,mode,args.source,args.results,args.accepted
            )
    lines=[]
    for case in CASES:
        one = results[case,"one"]
        three = results[case,"three"]
        need(one["default"] == three["default"],
             "the same Fischer candidate order changed between shadow policies")
        for mode in MODES:
            data = results[case,mode]
            lines.append(
                "BTC_SHADOW_TRIAL|case={case}|mode={mode}|status={status}|"
                "nodes={nodes}|literal_checks={literal_checks}|"
                "first_fischer={default}|first_selected={first}|"
                "choices={choices}|shadow_ns={shadow_ns}|"
                "solver_ns={solver_ns}|total_ns={total_ns}|"
                "gguf_sha256={gguf_sha}|json=0".format(**data)
            )
        lines.append(
            f"BTC_SHADOW_COMPARE|case={case}|"
            f"first_one={one['first']}|first_three={three['first']}|"
            f"changed={'YES' if one['first'] != three['first'] else 'NO'}|"
            "identical_original_cnf=PASS|identical_gguf=PASS|json=0"
        )
    output="\n".join(lines)+"\n"
    summary = (
        "BTC_SHADOW_FINAL|schema=1|original_cases=2|modes=3|trials=6|"
        "artificial_timer=NO|node_cap=2|"
        "independent_gguf_and_full_source=PASS|"
        "three_shadow_witness=SOURCE_CNF_PLUS_LIVE_ASSIGNMENT|"
        f"rows_sha256={sha(output.encode())}|status=PASS|json=0\n"
    )
    args.out.write_text(summary+output,encoding="ascii")
    print(summary+output,end="",flush=True)

if __name__ == "__main__":
    main()
