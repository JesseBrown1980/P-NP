#!/usr/bin/env python3
"""Second-seat, standard-library exact reviewer for original Bitcoin 3D -> three 2D shadows.

The input source and RimeSphere binary are the immutable successful hosted
run #44652775916. The new shadow archive is independently reassembled into
*every* 27-cell room. Rank of the three 9-pixel projectors is computed exactly,
and eight witness trits complete the inverse. No Rust projection reader imported.
"""
import argparse
from fractions import Fraction
import hashlib
import struct
from pathlib import Path

import bitcoin_live_sat_referee as btc
import bitcoin_gguf_fischer_referee as gguf

ROOM = 27
RECORD = 35
HEADER = 160
GULP = 729
ORIGINAL_SUCCESS_RUN = 44652775916


def sha(data):
    return hashlib.sha256(data).hexdigest()


def need(ok, why):
    if not ok:
        raise ValueError(why)


def cell(x, y, z):
    return x + 3 * y + 9 * z


def proj(state):
    need(len(state) == 27 and all(-1 <= v <= 1 for v in state),
         "three shadows require exact source trits")
    plane = [[0] * 9 for _ in range(3)]
    for z in range(3):
        for y in range(3):
            for x in range(3):
                v = state[cell(x, y, z)]
                plane[0][3*y + z] += v
                plane[1][3*x + z] += v
                plane[2][3*x + y] += v
    interior = [state[cell(x, y, z)]
                for z in range(2) for y in range(2) for x in range(2)]
    return plane, interior


def play_record(raw):
    need(len(raw) == RECORD, "truncated three-shadow room")
    values = [v if v < 128 else v - 256 for v in raw]
    planes = [values[i*9:(i+1)*9] for i in range(3)]
    witnesses = values[27:35]
    need(all(-3 <= v <= 3 for plane in planes for v in plane)
         and all(-1 <= v <= 1 for v in witnesses),
         "out-of-range signed shadow pixel or residual witness")
    restored = [None] * ROOM
    for z in range(2):
        for y in range(2):
            for x in range(2):
                restored[cell(x,y,z)] = witnesses[x + 2*y + 4*z]
    for count in (1, 2, 3):
        for z in range(3):
            for y in range(3):
                for x in range(3):
                    if (x == 2) + (y == 2) + (z == 2) != count:
                        continue
                    if x == 2:
                        expected = planes[0][3*y+z]
                        a, b = cell(0,y,z), cell(1,y,z)
                    elif y == 2:
                        expected = planes[1][3*x+z]
                        a, b = cell(x,0,z), cell(x,1,z)
                    else:
                        expected = planes[2][3*x+y]
                        a, b = cell(x,y,0), cell(x,y,1)
                    recovered = expected - restored[a] - restored[b]
                    need(-1 <= recovered <= 1,
                         "2D shadows reconstruct a nonternary 3D cell")
                    restored[cell(x,y,z)] = recovered
    need(all(v is not None for v in restored),
         "3D room reconstruction incomplete")
    need(proj(restored) == (planes, witnesses),
         "three independent projected planes disagree on 3D room")
    return bytes(v & 255 for v in restored)


def projector_rank():
    matrix = [[Fraction(0) for _ in range(27)] for _ in range(27)]
    for y in range(3):
        for z in range(3):
            for x in range(3):
                matrix[y*3+z][cell(x,y,z)] = 1
    for x in range(3):
        for z in range(3):
            for y in range(3):
                matrix[9+x*3+z][cell(x,y,z)] = 1
    for x in range(3):
        for y in range(3):
            for z in range(3):
                matrix[18+x*3+y][cell(x,y,z)] = 1
    rank = 0
    for col in range(27):
        found = next((r for r in range(rank,27) if matrix[r][col]), None)
        if found is None:
            continue
        matrix[rank], matrix[found] = matrix[found], matrix[rank]
        pivot = matrix[rank][col]
        matrix[rank] = [v/pivot for v in matrix[rank]]
        for row in range(rank+1,27):
            factor = matrix[row][col]
            if factor:
                matrix[row] = [
                    a-factor*b for a,b in zip(matrix[row], matrix[rank])
                ]
        rank += 1
    return rank


def fields(line, family):
    need(line.startswith(family+"|") and line.endswith("\n"),
         "not an exact full record of " + family)
    pairs = [token.split("=",1) for token in line.rstrip("\n").split("|")[1:]]
    need(all(len(p)==2 for p in pairs), "malformed three-shadow metadata")
    row = dict(pairs)
    need(len(row) == len(pairs) and row.get("json") == "0",
         "duplicate or noncanonical metadata")
    return row


def source_record_layout(original, shadow_binary, shadow_index, input_bytes):
    source_data = input_bytes
    root = sha(b"PNP_BITCOIN_RIMESPHERE_STREAM_V1")
    shadow_root = sha(b"PNP_THREE_LIGHT_THREE_SHADOWS_V1")
    orig_offset = 0
    rooms_seen = 0
    shadow_bytes = 0
    slice_count = 0
    while True:
        header = original.read(HEADER)
        if not header:
            break
        need(len(header) == HEADER and header[:8] == b"PNPRIME1",
             "RimeSphere input truncated")
        n, off = struct.unpack_from("<QQ", header, 8)
        src_used, room_count = struct.unpack_from("<HH", header, 24)
        size = struct.unpack_from("<I", header, 28)[0]
        need(n == slice_count and off == orig_offset
             and 0 < src_used <= GULP
             and room_count == (src_used*6+26)//27
             and size == room_count*27, "original RIME 27-cell dimensions changed")
        origin_sha = header[32:96].decode("ascii")
        expected_rime = header[96:160].decode("ascii")
        old_plane = original.read(size)
        need(len(old_plane) == size,
             "original 27-cell RimeSphere source plane truncated")
        wanted = b"".join(
            bytes(reversed([((byte // (3**power)) % 3 - 1) & 255
                            for power in range(6)]))
            for byte in source_data[off:off+src_used]
        )
        wanted += bytes(size-len(wanted))
        need(old_plane == wanted and sha(source_data[off:off+src_used]) == origin_sha,
             "original Bitcoin RimeSphere source differs")
        root = sha((
            f"PNP_RIME_STREAM_STEP_V1|{root}|{n}|{off}|{src_used}|"
            f"{room_count}|{origin_sha}|{sha(old_plane)}"
        ).encode())
        need(root == expected_rime,
             "existing accepted original Bitcoin RimeSphere root differs")
        projected = shadow_binary.read(room_count*RECORD)
        need(len(projected) == room_count*RECORD,
             "three-shadow projected binary room stream truncated")
        for i in range(room_count):
            row = projected[i*RECORD:(i+1)*RECORD]
            restored = play_record(row)
            original_room = old_plane[i*ROOM:(i+1)*ROOM]
            need(restored == original_room,
                 "three light 2D shadows fail original 3D Bitcoin room")
        index_line = shadow_index.readline()
        row = fields(index_line, "THREE_SHADOW_SLICE")
        shadow_root = sha((
            f"PNP_TRIPLE_LIGHT_STEP_V1|{shadow_root}|{n}|{off}|"
            f"{src_used}|{room_count}|{origin_sha}|{sha(projected)}"
        ).encode())
        for key,value in (
            ("slice", str(n)), ("source_offset",str(off)),
            ("source_bytes",str(src_used)), ("rooms",str(room_count)),
            ("trits",str(src_used*6)), ("original_sha256",origin_sha),
            ("original_rime_root",root), ("shadow_sha256",sha(projected)),
            ("shadow_root",shadow_root)
        ):
            need(row.get(key) == value,
                 "three-light independently verified HBI differs: "+key)
        slice_count += 1
        orig_offset += src_used
        rooms_seen += room_count
        shadow_bytes += len(projected)
    need(slice_count and orig_offset == len(source_data)
         and shadow_index.read(1) == ""
         and shadow_binary.read(1) == b"",
         "three-shadow source stopped early or contains undeclared records")
    return dict(
        slices=slice_count, bytes=orig_offset, rooms=rooms_seen,
        shadow_bytes=shadow_bytes, root=shadow_root, old_root=root
    )


def main():
    args = argparse.ArgumentParser()
    args.add_argument("--case", choices=tuple(btc.CASES), required=True)
    args.add_argument("--source", type=Path, required=True)
    args.add_argument("--old", type=Path, required=True)
    args.add_argument("--new", type=Path, required=True)
    args.add_argument("--out", type=Path, required=True)
    args = args.parse_args()
    expected, variables, clauses = btc.CASES[args.case]
    source = args.source.read_bytes()
    need(sha(source) == expected and source == (args.old/"BITCOIN.replayed.cnf").read_bytes(),
         "source-original/previous RIME byte identity changed")
    rank = projector_rank()
    need(rank == 19 and 27-rank == 8,
         "three orthogonal projected shadows rank/witness theorem differs")
    old_gguf = (args.old/"BITCOIN.rimesphere.gguf").read_bytes()
    new_gguf = (args.new/"BITCOIN.three_shadows.gguf").read_bytes()
    src_gguf, state, meta = gguf.parse(new_gguf)
    need(old_gguf == new_gguf and src_gguf == source
         and len(state) == variables and all(v == 0 for v in state)
         and meta["pnp.original_cnf_sha256"] == expected,
         "three-shadow GGUF differs from independently verified original source")
    for variable in (1,27,28,81,variables//2,variables):
        _, played = gguf.fischer_point(state,variable)
        need(played == 0, "three-shadow restored GGUF injected source SAT data")
    record = fields((args.new/"BITCOIN_THREE_SHADOW.hbp").read_text("ascii"),
                    "BITCOIN_THREE_SHADOW_FINAL")
    with (args.old/"BITCOIN.rime").open("rb") as old, \
         (args.new/"BITCOIN.three_shadows.bin").open("rb") as projected, \
         (args.new/"BITCOIN.three_shadows.hbi").open("rt",encoding="ascii") as idx:
        measured = source_record_layout(old,projected,idx,source)
    for key,val in (
        ("schema","1"),("case",args.case),("source_sha256",expected),
        ("variables",str(variables)),("clauses",str(clauses)),
        ("source_bytes",str(len(source))),
        ("slices",str(measured["slices"])),("rooms",str(measured["rooms"])),
        ("three_lights","3"),("shadows_per_room","3"),
        ("pixels_per_shadow","9"),("projector_rank","19"),
        ("independent_residual_trits","8"),
        ("shadow_record_bytes","35"),
        ("shadow_data_bytes",str(measured["shadow_bytes"])),
        ("shadow_hbi_sha256",sha((args.new/"BITCOIN.three_shadows.hbi").read_bytes())),
        ("shadow_root",measured["root"]),
        ("original_rime_root",measured["old_root"]),
        ("gguf_sha256",sha(new_gguf)),("gguf_bytes",str(len(new_gguf))),
        ("parent_gguf_sha256",sha(old_gguf)),
        ("original_source","BYTE_EXACT"),("3d_reconstruction","EXACT"),
        ("gguf_identity","BYTE_EXACT"),("SAT_search","NOT_RUN"),
        ("status","PASS")
    ):
        need(record.get(key) == val,
             "three-shadow host receipt and independent source disagree: "+key)
    recovered = (args.new/"BITCOIN.three_shadows.replayed.cnf").read_bytes()
    need(recovered == source,
         "independent three-shadow source reconstruction differs")
    need(all(record.get(k,"").isdigit() for k in (
        "shadow_stream_wall_ns","shadow_play_wall_ns","gguf_wall_ns","total_wall_ns"
    )), "unmetered three-shadow processing")
    controls = 0
    with (args.old/"BITCOIN.rime").open("rb") as stream:
        need(len(stream.read(HEADER)) == HEADER, "original first room absent")
        first_original_room = stream.read(ROOM)
    with (args.new/"BITCOIN.three_shadows.bin").open("rb") as stream:
        original_shadow_room = stream.read(RECORD)
    need(len(first_original_room) == ROOM and
         len(original_shadow_room) == RECORD,
         "original three-shadow negative control missing")
    for where in (0,26,27,34):
        sample = bytearray(original_shadow_room)
        sample[where] ^= 32
        need(sha(sample) != sha(original_shadow_room),
             "tampered 2D shadow record failed to change its hash")
        try:
            played = play_record(bytes(sample))
        except ValueError:
            pass
        else:
            need(played != first_original_room,
                 "tampered three-shadow projection unexpectedly preserved 3D source")
        controls += 1
    genesis = btc.PREFIX + btc.NONCE.to_bytes(4,"little")
    need(btc.double_sha(genesis) == btc.TARGET,
         "original Bitcoin genesis full SHA256d target changed")
    for bit in (1,2,4,8):
        for at in (36,76):
            modified = bytearray(genesis)
            modified[at] ^= bit
            need(btc.double_sha(modified) != btc.TARGET,
                 "incorrect Bitcoin header accepted")
    result = (
        f"BITCOIN_THREE_SHADOW_REFEREE|case={args.case}|"
        f"source_sha256={expected}|rooms={measured['rooms']}|"
        f"shadow_rank={rank}|missing_dimensions={27-rank}|"
        f"shadow_bytes={measured['shadow_bytes']}|"
        f"source_bytes={measured['bytes']}|"
        f"gguf_sha256={sha(new_gguf)}|"
        f"root={measured['root']}|"
        f"negative_controls={controls}|"
        "original_cnf=BYTE_EXACT|three_shadows=RECONSTRUCTED|"
        "status=PASS|json=0\n"
    )
    args.out.write_text(result,encoding="ascii")
    print(result,end="",flush=True)


if __name__ == "__main__":
    main()
