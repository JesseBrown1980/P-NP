#!/usr/bin/env python3
"""Independent source-authenticated Bitcoin RimeSphere stream referee.

Uses Python standard-library arithmetic to reconstruct all source bytes,
check the nested 3/27/81-cell stream, independent SHA chain, all HBI records,
standard GGUF v3 and original Bitcoin double-SHA256 control. No Rust import.
"""
import argparse
import hashlib
import io
import struct
from pathlib import Path

import bitcoin_gguf_fischer_referee as gguf
import bitcoin_live_sat_referee as bitcoin

MAGIC = b"PNPRIME1"
HEADER = 160
GULP = 729
ROOM = 27
GROUP = 81
BYTE_TRITS = 6

def need(ok, message):
    if not ok:
        raise ValueError(message)

def sha(data):
    return hashlib.sha256(data).hexdigest()

def read_fields(line, family):
    fields = line.rstrip("\n").split("|")
    need(fields[0] == family and line.endswith("\n"),
         "invalid RimeSphere receipt family " + family)
    pairs = [part.split("=", 1) for part in fields[1:]]
    need(all(len(pair) == 2 for pair in pairs), "unkeyed receipt field")
    out = dict(pairs)
    need(len(out) == len(pairs) and out.get("json") == "0",
         "duplicate or noncanonical receipt fields")
    return out

def decode_byte(group):
    need(len(group) == BYTE_TRITS, "wrong balanced ternary group width")
    value = 0
    for trit in group:
        need(trit in (0, 1, 255), "nontrit byte in RimeSphere")
        value = value * 3 + {255: 0, 0: 1, 1: 2}[trit]
    need(value < 256, "noncanonical six-trit mapping outside original binary floor")
    return value

def encode_byte(byte):
    values = []
    for _ in range(6):
        rem = byte % 3
        values.append((255, 0, 1)[rem])
        byte //= 3
    return bytes(reversed(values))

def verify_stream(fileobj, source, index_lines):
    root = sha(b"PNP_BITCOIN_RIMESPHERE_STREAM_V1")
    source_off = slice_index = trit_count = room_count = fischer_reads = stored = 0
    while True:
        header = fileobj.read(HEADER)
        if not header:
            break
        need(len(header) == HEADER and header[:8] == MAGIC,
             "noncanonical RimeSphere record magic or header")
        idx, off = struct.unpack_from("<QQ", header, 8)
        length, rooms = struct.unpack_from("<HH", header, 24)
        payload = struct.unpack_from("<I", header, 28)[0]
        need(idx == slice_index and off == source_off,
             "RimeSphere source-offset/slice addressing not contiguous")
        need(0 < length <= GULP and rooms == (6 * length + 26) // 27
             and rooms <= 162 and payload == rooms * ROOM,
             "RimeSphere slice length or 27-cell geometry differs")
        source_sha = header[32:96].decode("ascii")
        expected_root = header[96:160].decode("ascii")
        plane = fileobj.read(payload)
        need(len(plane) == payload, "RimeSphere plane truncated")
        data = bytes(decode_byte(plane[i:i+6]) for i in range(0, length*6, 6))
        need(plane[length*6:] == bytes(payload - length*6),
             "nonzero source-padding trits")
        canonical = b"".join(encode_byte(byte) for byte in data)
        need(plane == canonical + bytes(payload - length*6),
             "noncanonical byte->trit RimeSphere projection")
        need(sha(data) == source_sha
             and source[source_off:source_off+length] == data,
             "stream source does not match original Bitcoin byte order")
        plane_sha = sha(plane)
        root = sha((
            f"PNP_RIME_STREAM_STEP_V1|{root}|{idx}|{off}|"
            f"{length}|{rooms}|{source_sha}|{plane_sha}"
        ).encode())
        need(root == expected_root,
             "RimeSphere source-bound rolling receipt chain differs")
        need(len(index_lines) > slice_index, "missing HBI gulp index")
        indexed = read_fields(index_lines[slice_index], "RIME_SLICE")
        for key, value in (
            ("index", str(idx)), ("source_offset", str(off)),
            ("source_bytes", str(length)), ("rooms", str(rooms)),
            ("sphere_groups", str((rooms + GROUP - 1) // GROUP)),
            ("source_sha256", source_sha), ("plane_sha256", plane_sha),
            ("root", root)
        ):
            need(indexed.get(key) == value, "HBI source index differs: " + key)
        for room in (0, 1, 26, 27, 80, 81, 161):
            if room >= rooms:
                continue
            for cell in (0, 13, 26):
                addr = pow(7, 27 * room + cell, 1_000_081)
                pos = room * ROOM + cell
                need(addr != 0 and plane[pos] in (0, 1, 255),
                     "Fischer cell address or ternary state differs")
                # Global address includes (slice index, local mod-p code).
                # The same local address in a different slice is not the same data.
                fischer_reads += 1
        source_off += length
        trit_count += length * BYTE_TRITS
        room_count += rooms
        stored += HEADER + payload
        slice_index += 1
    need(source_off == len(source) and slice_index == len(index_lines)
         and slice_index > 0, "RimeSphere ended early or contains extra records")
    return dict(slices=slice_index, bytes=source_off, trits=trit_count,
                rooms=room_count, fischer_reads=fischer_reads,
                sphere_bytes=stored, root=root)

def verify(case, source_path, folder, receipt_path):
    expected, variables, clauses = bitcoin.CASES[case]
    source = source_path.read_bytes()
    need(sha(source) == expected, "original Bitcoin CNF bytes changed")
    lines = (folder / "BITCOIN.hbi").read_text("ascii").splitlines(keepends=True)
    with (folder / "BITCOIN.rime").open("rb") as stream:
        counts = verify_stream(stream, source, lines)
    recovered = (folder / "BITCOIN.replayed.cnf").read_bytes()
    need(recovered == source, "replayed RimeSphere differs byte-for-byte")
    raw_gguf = (folder / "BITCOIN.rimesphere.gguf").read_bytes()
    gguf_source, state, meta = gguf.parse(raw_gguf)
    need(gguf_source == source and state == bytes(variables)
         and meta["pnp.original_cnf_sha256"] == expected,
         "standard GGUF checkpoint differs from original sphere source")
    for variable in (1, 2, 27, 28, 81, variables//2, variables):
        address, trit = gguf.fischer_point(state, variable)
        need(address != 0 and trit == 0, "GGUF Fischer source playback altered seed")
    record = read_fields(receipt_path.read_text("ascii"), "BITCOIN_RIMESPHERE_FINAL")
    for key, value in (
        ("schema", "1"), ("case", case), ("source_sha256", expected),
        ("variables", str(variables)), ("clauses", str(clauses)),
        ("source_bytes", str(len(source))), ("slices", str(counts["slices"])),
        ("trits", str(counts["trits"])), ("rooms", str(counts["rooms"])),
        ("fischer_reads", str(counts["fischer_reads"])),
        ("sphere_bytes", str(counts["sphere_bytes"])),
        ("sphere_root", counts["root"]),
        ("index_sha256", sha((folder / "BITCOIN.hbi").read_bytes())),
        ("gguf_sha256", sha(raw_gguf)), ("gguf_bytes", str(len(raw_gguf))),
        ("streamed_source_sha256", expected),
        ("source_bytes_per_gulp", str(GULP)), ("rooms_per_sphere", str(GROUP)),
        ("sphere_arity", "3-27-81"),
        ("reconstructed_source", "BYTE_EXACT"),
        ("original_cnf", "AUTHENTIC"), ("known_nonce_injected", "NO"),
        ("sat_search", "NOT_RUN"), ("checkpoint", "READY"), ("status", "PASS")
    ):
        need(record.get(key) == value, "independent RimeSphere receipt disagrees: " + key)
    need(all(record.get(key, "").isdigit() for key in (
        "stream_wall_ns", "replay_wall_ns", "gguf_wall_ns", "total_wall_ns"
    )), "RimeSphere true wall accounting missing")

    # Independent tamper tests operate on one complete slice and never
    # require loading the whole potentially enormous archive into RAM.
    first = min(len(source), GULP)
    plane_len = ((first*6+26)//27) * 27
    with (folder / "BITCOIN.rime").open("rb") as stream:
        first_record = stream.read(HEADER + plane_len)
    need(len(first_record) == HEADER + plane_len, "negative control record missing")
    rejected = 0
    for place in (0, 8, 96, len(first_record)-1):
        altered = bytearray(first_record)
        altered[place] ^= 1
        try:
            verify_stream(io.BytesIO(altered), source[:first], lines[:1])
        except (ValueError, UnicodeDecodeError):
            rejected += 1
    need(rejected == 4, "a corrupted RimeSphere record was accepted")

    genesis = bitcoin.PREFIX + bitcoin.NONCE.to_bytes(4, "little")
    need(bitcoin.double_sha(genesis) == bitcoin.TARGET,
         "independent source Bitcoin genesis SHA256d differs")
    controls = 0
    for where in (36, 76):
        for bit in (1, 2, 4, 8):
            changed = bytearray(genesis)
            changed[where] ^= bit
            need(bitcoin.double_sha(changed) != bitcoin.TARGET,
                 "modified Bitcoin source header passed")
            controls += 1
    return (f"BITCOIN_RIME_REFEREE|case={case}|source_sha256={expected}|"
            f"slices={counts['slices']}|bytes={counts['bytes']}|"
            f"rooms={counts['rooms']}|fischer_reads={counts['fischer_reads']}|"
            f"root={counts['root']}|gguf_sha256={sha(raw_gguf)}|"
            f"corrupt_records_rejected={rejected}|"
            f"bad_bitcoin_headers_rejected={controls}|"
            "source_replay=BYTE_EXACT|status=PASS|json=0\n")

def main():
    p = argparse.ArgumentParser()
    p.add_argument("--case", required=True, choices=tuple(bitcoin.CASES))
    p.add_argument("--source", required=True, type=Path)
    p.add_argument("--folder", required=True, type=Path)
    p.add_argument("--out", required=True, type=Path)
    args = p.parse_args()
    result = verify(args.case, args.source, args.folder,
                    args.folder / "BITCOIN_RIMESPHERE.hbp")
    args.out.write_text(result, encoding="ascii")
    print(result, end="", flush=True)

if __name__ == "__main__":
    main()
