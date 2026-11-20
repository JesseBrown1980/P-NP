#!/usr/bin/env python3
"""Independent original-Bitcoin SAT referee for GGUF-frozen Bobby Fischer search.

No native Rust code is imported. GGUF is parsed with a second implementation;
exact original source files, candidate models, 27-cell addresses and honest
HOLD outcomes are checked without injecting the known Bitcoin nonce.
"""
import argparse
import hashlib
import time
from pathlib import Path

import bitcoin_gguf_fischer_referee as format_ref
import bitcoin_live_sat_referee as bitcoin

MODES = ("direct", "gguf", "guided")
CASES = ("nonce32", "merkle_nonce288")
NODES = 128

def need(ok, reason):
    if not ok:
        raise ValueError(reason)

def sha(payload):
    return hashlib.sha256(payload).hexdigest()

def unsigned(row, key):
    raw = row.get(key)
    need(raw is not None and raw.isdecimal(), "invalid numeric receipt meter " + key)
    return int(raw)

def receipt_file(path):
    lines = path.read_text("ascii").splitlines()
    need(len(lines) == 1 and lines[0].startswith("BTC_GGUF_SAT_RESUME|"),
         "noncanonical SAT GGUF search receipt")
    raw = lines[0].split("|")[1:]
    pairs = [x.split("=", 1) for x in raw]
    need(all(len(pair) == 2 for pair in pairs), "non-keyed SAT receipt field")
    fields = dict(pairs)
    need(len(fields) == len(pairs), "repeated SAT receipt field")
    return fields

def reference_fischer_trace(states):
    seen = {x for x in (1, 2, 27, 28, 81, 82,
                        len(states)//2, len(states)-1, len(states))
            if 1 <= x <= len(states)}
    seen.update(1 + ((x * 65537) % len(states)) for x in range(27))
    raw = b"".join(
        f"FISCHER|variable={v}|address={pow(7, v-1, 1000081)}|trit=0\n".encode()
        for v in sorted(seen)
    )
    return len(seen), sha(raw)

def independent_trial(case, mode, source, output):
    expected, variables, clauses = bitcoin.CASES[case]
    directory = output / mode / case
    need(directory.is_dir(), "missing original-CNF solver output " + str(directory))
    need((directory / "stderr.log").is_file() and
         (directory / "stdout.log").is_file() and
         (directory / "rss.txt").is_file(), "missing raw timed solver evidence")
    memory_lines = [
        line.strip() for line in (directory / "rss.txt")
        .read_text("ascii").splitlines() if line.strip()
    ]
    need(bool(memory_lines) and memory_lines[-1].isdecimal(),
         "missing GNU time peak memory RSS meter")
    memory_kib = memory_lines[-1]
    record = directory / "receipt.hbp"
    timeout = directory / "timeout.hbp"
    need(record.is_file() != timeout.is_file(),
         "worker must publish exactly one completed or time-limited state")
    if timeout.is_file():
        need(timeout.read_text("ascii") == (
             f"PNP_BTC_GGUF_TIMEOUT|mode={mode}|case={case}|"
             "wall_budget_s=300|status=HOLD_TIME|"
             "nodes=UNOBSERVED|json=0\n"),
             "invalid original-CNF GGUF wall-time HOLD")
        need(not (directory / "model.trits").exists() and
             not record.exists(), "timed-out worker claims a SAT model")
        return dict(case=case, mode=mode, status="HOLD_TIME",
                    nodes="UNOBSERVED", time="UNOBSERVED",
                    gguf="UNOBSERVED", source=expected,
                    worker_non_solver_ns="UNOBSERVED",
                    fischer_io_ns="UNOBSERVED",
                    solver_ns="UNOBSERVED", matrix_ns="UNOBSERVED",
                    selector_ns="UNOBSERVED", max_rss_kib=memory_kib)
    row = receipt_file(record)
    for field, value in (
        ("schema", "1"), ("case", case), ("mode", mode),
        ("original_cnf_sha256", expected),
        ("variables", str(variables)), ("clauses", str(clauses)),
        ("seed_known_assignments", "0"), ("wall_budget_seconds", "300"),
        ("node_cap", str(NODES)), ("known_bitcoin_nonce_injected", "NO"),
        ("original_clauses", "EXACT"), ("json", "0")
    ):
        need(row.get(field) == value, "original-CNF receipt mismatch: " + field)
    ints = {key: unsigned(row, key) for key in (
        "nodes", "propagations", "literal_checks", "source_parse_ns",
        "gguf_bytes", "gguf_freeze_ns", "gguf_write_ns", "gguf_read_ns",
        "gguf_thaw_ns", "fischer_play_ns", "fischer_reads",
        "fischer_route_ns", "solver_ns", "total_ns", "live_frames",
        "live_watcher_reads", "live_frozen_bytes", "live_metadata_bytes",
        "live_matrix_ns", "selector_choices", "selector_nonfirst",
        "selector_candidates", "selector_index_bytes",
        "selector_room_replays", "selector_frozen_bytes",
        "selector_metadata_bytes", "selector_index_ns", "selector_ns",
    )}
    if NODES != 128:
        need(row.get("internal_deadline_s") == "285",
             "raised-node Bitcoin GGUF worker lacks internal time stop")
    need(ints["nodes"] <= NODES and
         ints["total_ns"] >= ints["solver_ns"] and
         ints["live_frames"] == ints["live_watcher_reads"] and
         ints["live_metadata_bytes"] == 192 * ints["live_frames"],
         "bad exact solver budget or live matrix cost metering")
    need(row.get("seed_state_sha256") == sha(bytes(variables)),
         "original root assignment is not independently verified all-zero trits")
    if mode == "direct":
        need(not (directory / "source_state.gguf").exists() and
             row.get("gguf_sha256") == "NONE" and
             row.get("fischer_trace_sha256") == "NONE",
             "direct SAT baseline incorrectly includes GGUF or Fischer")
        need(all(ints[key] == 0 for key in (
            "gguf_bytes", "gguf_freeze_ns", "gguf_write_ns", "gguf_read_ns",
            "gguf_thaw_ns", "fischer_play_ns", "fischer_reads",
            "live_frames", "live_frozen_bytes", "selector_choices"
        )), "hidden GGUF operations in direct baseline")
    else:
        path = directory / "source_state.gguf"
        need(path.is_file(), "missing real GGUF artifact for resumed SAT")
        raw = path.read_bytes()
        unpacked, state, meta = format_ref.parse(raw)
        need(unpacked == source and len(state) == variables and
             state == bytes(variables), "GGUF cannot reconstruct original SAT root")
        need(row.get("gguf_sha256") == sha(raw) and
             ints["gguf_bytes"] == len(raw) and
             meta["pnp.original_cnf_sha256"] == expected,
             "GGUF bytes/record/source digest mismatch")
        reads, trace_sha = reference_fischer_trace(state)
        need(ints["fischer_reads"] == reads and
             row.get("fischer_trace_sha256") == trace_sha,
             "Bobby Fischer modular 27-room replay differs independently")
        need(ints["live_frames"] > 0 and ints["live_frozen_bytes"] > 0,
             "live solver resumed without actual matrix state frames")
        need(len(row.get("live_frame_root", "")) == 64,
             "no source-bound native watcher root")
    if mode == "guided":
        need(ints["selector_choices"] > 0 and
             ints["selector_candidates"] >= ints["selector_choices"] and
             ints["selector_room_replays"] > 0 and
             ints["selector_frozen_bytes"] > 0 and
             ints["selector_index_bytes"] > 0 and
             ints["selector_metadata_bytes"] ==
                 192 * ints["selector_room_replays"],
             "matrix-guided selector not actually used and fully charged")
        need(len(row.get("selector_root", "")) == 64,
             "matrix-guided decision root absent")
    else:
        need(all(ints[key] == 0 for key in (
             "selector_choices", "selector_candidates", "selector_nonfirst",
             "selector_index_bytes", "selector_room_replays",
             "selector_frozen_bytes", "selector_metadata_bytes",
             "selector_index_ns", "selector_ns")),
             "non-guided control used extra selector")
        need(row.get("selector_root") == "NONE", "unexpected guided receipt")
    status = row.get("status")
    need(status in ("SAT", "HOLD_NODE", "HOLD_TIME", "HOLD_STACK"),
         "unsupported exact SAT result")
    if NODES == 128:
        need(status in ("SAT", "HOLD_NODE"), "pilot search reported raised-only status")
    if status == "HOLD_TIME":
        need(row.get("time_exhausted") == "true",
             "internal wall hold without timer event")
    if status == "HOLD_STACK":
        need(row.get("stack_exhausted") == "true",
             "stack hold without bounded recursive guard")
    if status == "HOLD_NODE":
        need(row.get("time_exhausted") == "false"
             and row.get("stack_exhausted") == "false",
             "node-budget stop contains unreported time or stack hold")
    if status == "SAT":
        model = (directory / "model.trits").read_bytes()
        need(sha(model) == row.get("model_sha256") and
             len(model) == variables + 1 and model.endswith(b"\n") and
             set(model[:-1]).issubset({43, 45}),
             "invalid complete source-SAT witness")
        selected = {i + 1 for i, byte in enumerate(model[:-1]) if byte == 43}
        bitcoin.verify_cnf_model(source_path(case, source), selected, variables, clauses)
        verify_bitcoin_header(case, selected)
    else:
        need(not (directory / "model.trits").exists() and
             row.get("model_sha256") == "NONE",
             "HOLD_NODE incorrectly claims SAT witness")
    fischer_io_ns = sum(ints[key] for key in (
        "gguf_freeze_ns", "gguf_write_ns", "gguf_read_ns",
        "gguf_thaw_ns", "fischer_play_ns",
    ))
    need(fischer_io_ns <= ints["total_ns"],
         "original-CNF GGUF I/O exceeds total measured worker time")
    return dict(case=case, mode=mode, status=status,
                nodes=str(ints["nodes"]), time=str(ints["total_ns"]),
                gguf=row.get("gguf_sha256"), source=expected,
                propagations=ints["propagations"],
                literal_checks=ints["literal_checks"],
                worker_non_solver_ns=str(ints["total_ns"] - ints["solver_ns"]),
                fischer_io_ns=str(fischer_io_ns),
                solver_ns=str(ints["solver_ns"]),
                matrix_ns=str(ints["live_matrix_ns"]),
                selector_ns=str(ints["selector_ns"]),
                max_rss_kib=memory_kib)

def source_path(case, raw_source):
    # This gets a path by matching the externally supplied, authenticated
    # manifest directory; no source rewriting or clause simplification.
    p = SOURCES_DIR / (case + ".cnf")
    need(p.read_bytes() == raw_source, "source bytes altered during verification")
    return p

SOURCES_DIR = Path(".")

def verify_bitcoin_header(case, selected):
    manifest = __import__("ast").literal_eval(
        (SOURCES_DIR / (case + ".txt")).read_text("ascii")
    )
    mask = (1 << manifest["nonce_free_bits"]) - 1
    nonce_low = sum(1 << bit for bit, var in manifest["nonce_variables"]
                    if var in selected)
    nonce = (bitcoin.NONCE & ~mask) | nonce_low
    header = bytearray(bitcoin.PREFIX + nonce.to_bytes(4, "little"))
    if manifest["merkle_free_bits"]:
        merkle = sum(1 << bit for bit, var in manifest["merkle_variables"]
                     if var in selected)
        header[36:68] = merkle.to_bytes(32, "little")
    need(bitcoin.double_sha(header) == bitcoin.TARGET,
         "complete source SAT witness does not hash to Bitcoin genesis target")

def main():
    global SOURCES_DIR, NODES
    parser = argparse.ArgumentParser()
    parser.add_argument("--node-cap", type=int, choices=(128, 1_000_000_000_000), default=128)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    SOURCES_DIR = args.source
    NODES = args.node_cap
    origin = bitcoin.PREFIX + bitcoin.NONCE.to_bytes(4, "little")
    need(bitcoin.double_sha(origin) == bitcoin.TARGET,
         "independent original Bitcoin header hash differs")
    corrupted = 0
    for where in (36, 76):
        for bit in (1, 2, 4, 8):
            other = bytearray(origin)
            other[where] ^= bit
            need(bitcoin.double_sha(other) != bitcoin.TARGET,
                 "altered original Bitcoin header passed")
            corrupted += 1
    known = {}
    for case in CASES:
        original = (args.source / (case + ".cnf")).read_bytes()
        need(sha(original) == bitcoin.CASES[case][0],
             "original Bitcoin SHA256d CNF source differs")
        for mode in MODES:
            verify_started = time.perf_counter_ns()
            known[mode, case] = independent_trial(
                case, mode, original, args.results
            )
            known[mode, case]["referee_ns"] = time.perf_counter_ns() - verify_started
    parity = 0
    for case in CASES:
        a, b = known["direct", case], known["gguf", case]
        if a["status"] in ("SAT", "HOLD_NODE") and b["status"] in ("SAT", "HOLD_NODE"):
            need(all(a[key] == b[key] for key in
                     ("status", "nodes", "propagations", "literal_checks")),
                 "GGUF playback altered exact source-Fischer DPLL tree")
            parity += 1
    records = []
    for case in CASES:
        for mode in MODES:
            value = known[mode, case]
            records.append(
                "BTC_GGUF_SAT_TRIAL|case={case}|mode={mode}|status={status}|"
                "nodes={nodes}|total_ns={time}|solver_ns={solver_ns}|"
                "non_solver_ns={worker_non_solver_ns}|"
                "gguf_fischer_io_ns={fischer_io_ns}|"
                "live_matrix_ns={matrix_ns}|guided_selector_ns={selector_ns}|"
                "max_rss_kib={max_rss_kib}|"
                "independent_referee_ns={referee_ns}|gguf_sha256={gguf}|"
                "source_sha256={source}|json=0".format(**value)
            )
    body = "\n".join(records) + "\n"
    result = (
        "BTC_GGUF_SAT_FINAL|schema=1|"
        "source=ORIGINAL_BITCOIN_GENESIS_SHA256D|"
        "cases=2|modes=3|trials=6|"
        f"node_cap_each={NODES}|wall_seconds_each=300|"
        "states=REAL_REPLAYED_GGUF|"
        f"direct_gguf_tree_parity_pairs={parity}|"
        f"wrong_header_controls={corrupted}|"
        f"rows_sha256={sha(body.encode())}|"
        "status=PASS|json=0\n" + body
    )
    args.out.write_text(result, "ascii")
    print(result, end="", flush=True)

if __name__ == "__main__":
    main()
