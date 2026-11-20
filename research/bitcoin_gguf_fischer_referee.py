#!/usr/bin/env python3
"""Independent, strict GGUF v3 source and Bobby Fischer readback.

This is a research-only reference implementation that does not import the
Rust-native GGUF reader, SHA256, SAT implementation or Fischer inverse.
The GGUF's original complete Bitcoin SHA256d source is checked bytewise.
"""
import argparse
import hashlib
import struct
from pathlib import Path

SOURCES = {
    "nonce32": ("6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3",
                239112, 877879),
    "merkle_nonce288": ("601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b",
                        361188, 1325357),
}
ARCH = "pnp-bitcoin-sha256d-ternary-sat"
NAME = "BITCOIN-FISCHER-FROZEN-CNF"
GENESIS_PREFIX = bytes.fromhex(
    "01000000" + "00" * 32
    + "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a"
    + "29ab5f49" + "ffff001d"
)
TARGET = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def need(ok, message):
    if not ok:
        raise ValueError(message)


class Reader:
    def __init__(self, data):
        self.data = data
        self.at = 0

    def take(self, n):
        need(n >= 0 and self.at + n <= len(self.data),
             "GGUF truncated or offset exceeds archive")
        part = self.data[self.at:self.at+n]
        self.at += n
        return part

    def number(self, fmt):
        return struct.unpack("<" + fmt, self.take(struct.calcsize(fmt)))[0]

    def string(self):
        size = self.number("Q")
        need(size <= 1024, "GGUF string metadata too large")
        return self.take(size).decode("ascii")


def parse(raw):
    cursor = Reader(raw)
    need(cursor.take(4) == b"GGUF" and cursor.number("I") == 3,
         "not a standard GGUF v3 archive")
    need(cursor.number("Q") == 2 and cursor.number("Q") == 5,
         "incorrect GGUF tensor/metadata count")
    fields = {}
    for _ in range(5):
        key = cursor.string()
        need(cursor.number("I") == 8 and key not in fields,
             "GGUF metadata key or type invalid")
        fields[key] = cursor.string()
    need(fields == {
        "general.architecture": ARCH,
        "general.name": NAME,
        "pnp.original_cnf_sha256": fields.get("pnp.original_cnf_sha256"),
        "pnp.sat_trits_sha256": fields.get("pnp.sat_trits_sha256"),
        "pnp.playback": "BOBBY_FISCHER_RIME_27_V1",
    } and len(fields) == 5, "GGUF metadata family changed")
    tensors = []
    for _ in range(2):
        name = cursor.string()
        need(cursor.number("I") == 1, "GGUF rank must be one")
        size = cursor.number("Q")
        need(cursor.number("I") == 24, "GGUF original source must use standard I8")
        offset = cursor.number("Q")
        tensors.append((name, size, offset))
    need(tensors[0][0] == "bitcoin.original_dimacs"
         and tensors[1][0] == "bitcoin.sat_trits"
         and tensors[0][2] == 0 and tensors[0][1] > 0
         and tensors[1][1] > 0, "GGUF tensor names or source offset differ")
    pad = (-cursor.at) % 32
    need(cursor.take(pad) == b"\0" * pad, "GGUF header padding changed")
    source = cursor.take(tensors[0][1])
    pad = (-cursor.at) % 32
    need(cursor.take(pad) == b"\0" * pad
         and tensors[1][2] == tensors[0][1] + pad,
         "GGUF source data alignment changed")
    states = cursor.take(tensors[1][1])
    need(cursor.at == len(raw), "GGUF undeclared trailing data")
    need(sha(source) == fields["pnp.original_cnf_sha256"]
         and sha(states) == fields["pnp.sat_trits_sha256"],
         "GGUF SHA256 source or ternary bytes differ")
    need(all(c in (0, 1, 255) for c in states),
         "GGUF source has invalid signed trit")
    return source, states, fields


def fischer_point(states, variable):
    need(1 <= variable <= len(states), "Fischer variable outside original source")
    index = variable - 1
    room, cell = divmod(index, 27)
    address = pow(7, room * 27 + cell, 1_000_081)
    need(address != 0 and pow(7, index, 1_000_081) == address,
         "Bobby Fischer original modular address does not close")
    return address, states[index]


def verify_file(path, source, variables, changes):
    raw = path.read_bytes()
    original, states, fields = parse(raw)
    need(original == source and len(states) == variables,
         "GGUF does not contain byte-exact original Bitcoin CNF and all variables")
    wanted = bytearray(variables)
    for variable, byte in changes.items():
        need(1 <= variable <= variables and byte in (1, 255),
             "invalid assigned trial state")
        wanted[variable - 1] = byte
    need(states == bytes(wanted), "frozen Bitcoin trits differ from expected branch")
    for variable in sorted(set([1, 2, 3, 27, 28, 81, variables // 2,
                                variables - 1, variables] + list(changes))):
        address, trit = fischer_point(states, variable)
        need(address >= 1 and trit == wanted[variable - 1],
             "Bobby Fischer original ternary playback disagreed")
    return {"bytes": len(raw), "gguf_sha": sha(raw),
            "state_sha": fields["pnp.sat_trits_sha256"]}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--case", choices=tuple(SOURCES), required=True)
    ap.add_argument("--source", type=Path, required=True)
    ap.add_argument("--initial", type=Path, required=True)
    ap.add_argument("--first", type=Path, required=True)
    ap.add_argument("--second", type=Path, required=True)
    ap.add_argument("--initial-receipt", type=Path, required=True)
    ap.add_argument("--first-receipt", type=Path, required=True)
    ap.add_argument("--second-receipt", type=Path, required=True)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()
    expected, variables, clauses = SOURCES[args.case]
    source = args.source.read_bytes()
    need(sha(source) == expected, "original Bitcoin CNF SHA mismatch")
    rows = source.splitlines()
    header = next((row for row in rows if row.startswith(b"p cnf ")), None)
    need(header == f"p cnf {variables} {clauses}".encode(),
         "original Bitcoin CNF declared problem dimensions changed")
    initial = verify_file(args.initial, source, variables, {})
    first = verify_file(args.first, source, variables, {28: 1})
    final = verify_file(args.second, source, variables, {28: 1, variables: 255})
    need(len({initial["gguf_sha"], first["gguf_sha"], final["gguf_sha"]}) == 3,
         "original, first-branch and second-branch freezes must be distinct")
    for file, gguf in [(args.initial_receipt, initial),
                       (args.first_receipt, first),
                       (args.second_receipt, final)]:
        receipt = file.read_text("ascii")
        need(receipt.startswith("BTC_GGUF_FISCHER|schema=1|")
             and f"|case={args.case}|" in receipt
             and f"|gguf_sha256={gguf['gguf_sha']}|" in receipt
             and "|original_cnf=EXACT|fischer_forward_reverse=PASS|" in receipt,
             "Bobby Fischer native played receipt fails independent GGUF bytes")
        need("|solver_search=NOT_RUN|" in receipt
             and "|known_nonce_injected=NO|" in receipt,
             "unjustified Bitcoin SAT/preimage claim")
    genesis_header = GENESIS_PREFIX + (2083236893).to_bytes(4, "little")
    calc = lambda b: hashlib.sha256(hashlib.sha256(b).digest()).digest()[::-1].hex()
    need(len(genesis_header) == 80 and calc(genesis_header) == TARGET,
         "independent original Bitcoin SHA256d target differs")
    negatives = 0
    for position in (36, 76):
        for mask in (1, 2, 4, 8):
            changed = bytearray(genesis_header)
            changed[position] ^= mask
            need(calc(changed) != TARGET, "wrong Bitcoin header accepted")
            negatives += 1
    # This is an exact source-freeze/play check, not a SAT search result.
    report = (
        f"BTC_GGUF_REFEREE|schema=1|case={args.case}|source_sha256={expected}|"
        f"gguf_v=3|stages=3|variables={variables}|clauses={clauses}|"
        f"initial_gguf_sha256={initial['gguf_sha']}|"
        f"branch_gguf_sha256={first['gguf_sha']}|"
        f"final_gguf_sha256={final['gguf_sha']}|"
        f"final_state_sha256={final['state_sha']}|"
        f"fischer_address_checks=PASS|source_roundtrip=BYTE_EXACT|"
        f"negative_header_controls={negatives}|solver_search=NOT_RUN|"
        "status=PASS|json=0\n"
    )
    args.out.write_text(report, "ascii")
    print(report, end="", flush=True)


if __name__ == "__main__":
    main()
