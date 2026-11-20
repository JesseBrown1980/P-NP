#!/usr/bin/env python3
"""Independent three-mode Bitcoin original-CNF matrix-guided selector referee.

Uses the previously accepted independent source/model SHA256d referee for
each raw result; never imports the native solver or matrix source. Reports
search effort and charged selector overhead without presuming improvement.
"""
import argparse
import hashlib
from pathlib import Path

import bitcoin_live_sat_referee as baseline

MODES = ("direct", "live", "guided")
CASES = ("nonce32", "merkle_nonce288")


def positive(row, name):
    value = row.get(name)
    baseline.need(value is not None and value.isdigit(),
                  "missing nonnegative guided selector meter " + name)
    return int(value)


def guided_meter_check(row, mode):
    fields = (
        "selector_choices", "selector_nonfirst", "selector_candidates",
        "selector_clause_reads", "selector_literal_reads",
        "selector_room_replays", "selector_frozen_bytes",
        "selector_metadata_bytes", "selector_index_bytes",
        "selector_index_build_ns", "selector_wall_ns"
    )
    meters = {field: positive(row, field) for field in fields}
    if mode == "guided":
        baseline.need(meters["selector_choices"] > 0
                      and meters["selector_nonfirst"] <= meters["selector_choices"],
                      "guided matrix made no actual decisions")
        baseline.need(meters["selector_candidates"] >= meters["selector_choices"]
                      and meters["selector_room_replays"] > 0
                      and meters["selector_frozen_bytes"] > 0,
                      "guided matrix candidate frames absent")
        baseline.need(meters["selector_metadata_bytes"]
                      == 192 * meters["selector_room_replays"],
                      "selector matrix proof metadata unaccounted")
        baseline.need(meters["selector_index_bytes"] > 0
                      and meters["selector_clause_reads"] > 0
                      and meters["selector_literal_reads"] > 0,
                      "source-indexed active clause scoring did not run")
        baseline.need(len(row.get("selector_root", "")) == 64
                      and all(c in "0123456789abcdef" for c in row["selector_root"]),
                      "guided selection receipt root invalid")
    else:
        baseline.need(all(value == 0 for value in meters.values())
                      and row.get("selector_root") == "NONE",
                      "direct/live control performed hidden matrix guidance")
    return meters


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    baseline.need(baseline.double_sha(
        baseline.PREFIX + baseline.NONCE.to_bytes(4, "little")) == baseline.TARGET,
        "Bitcoin genesis original source header differs")
    altered = 0
    for mask in (1, 2, 4, 8):
        for position in (36, 76):
            header = bytearray(baseline.PREFIX + baseline.NONCE.to_bytes(4, "little"))
            header[position] ^= mask
            baseline.need(baseline.double_sha(header) != baseline.TARGET,
                          "altered Bitcoin SHA256d header passed")
            altered += 1
    for case, (expected, _, _) in baseline.CASES.items():
        baseline.need(baseline.sha((args.source / (case + ".cnf")).read_bytes()) == expected,
                      "original Bitcoin full CNF source hash differs")
    results = {}
    selection = {}
    for case in CASES:
        for mode in MODES:
            result = baseline.verify_one(args.results, args.source, case, mode)
            results[mode, case] = result
            if result["status"] != "HOLD_TIME":
                row = baseline.read_hbp(
                    args.results / mode / case / "receipt.hbp")
                selection[mode, case] = guided_meter_check(row, mode)
            else:
                selection[mode, case] = None
    matched_tree = 0
    for case in CASES:
        control = results["direct", case]
        observed = results["live", case]
        if control["status"] != "HOLD_TIME" and observed["status"] != "HOLD_TIME":
            baseline.need(control["status"] == observed["status"]
                          and control["nodes"] == observed["nodes"],
                          "direct/live original DPLL tree differs")
            for field in ("propagations", "literal_checks"):
                left = baseline.read_hbp(
                    args.results / "direct" / case / "receipt.hbp")
                right = baseline.read_hbp(
                    args.results / "live" / case / "receipt.hbp")
                baseline.need(left[field] == right[field],
                              "direct/live original SAT search work differs")
            matched_tree += 1
    lines = []
    for case in CASES:
        for mode in MODES:
            record = results[mode, case]
            meters = selection[mode, case]
            lines.append(
                "BTC_MATRIX_GUIDED_TRIAL|case={case}|mode={mode}|status={status}|"
                "nodes={nodes}|frames={frames}|total_ns={total_ns}|"
                "source_sha256={source}|choices={choices}|nonfirst={nonfirst}|"
                "candidate_evaluations={candidate}|room_replays={replay}|"
                "selector_index_bytes={index_bytes}|selector_frozen_bytes={frozen}|"
                "selector_ns={selector_ns}|json=0".format(
                    **record,
                    choices=meters["selector_choices"] if meters else "UNOBSERVED",
                    nonfirst=meters["selector_nonfirst"] if meters else "UNOBSERVED",
                    candidate=meters["selector_candidates"] if meters else "UNOBSERVED",
                    replay=meters["selector_room_replays"] if meters else "UNOBSERVED",
                    index_bytes=meters["selector_index_bytes"] if meters else "UNOBSERVED",
                    frozen=meters["selector_frozen_bytes"] if meters else "UNOBSERVED",
                    selector_ns=meters["selector_wall_ns"] if meters else "UNOBSERVED",
                )
            )
    body = "\n".join(lines) + "\n"
    header = (
        "BTC_MATRIX_GUIDED_FINAL|schema=1|source=BITCOIN_GENESIS_SHA256D|"
        "cases=2|modes=3|observations=6|node_budget_each=128|"
        "wall_budget_each_seconds=300|"
        "guidance=ORIGINAL_CNF_ACTIVE_CLAUSE_MATRIX27|"
        "known_nonce_injected=NO|"
        f"direct_live_parity_pairs={matched_tree}|"
        f"wrong_header_controls={altered}|"
        f"rows_sha256={hashlib.sha256(body.encode()).hexdigest()}|"
        "status=PASS|json=0\n"
    )
    args.out.write_text(header + body, encoding="ascii")
    print(header.rstrip(), flush=True)
    for line in lines:
        print(line, flush=True)


if __name__ == "__main__":
    main()
