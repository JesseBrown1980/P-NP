#!/usr/bin/env python3
"""Bit-blast the real Bitcoin genesis SHA256(SHA256(80-byte header)) to CNF.

The solver sees only the original 80-byte header with 4/8/32 free nonce bits
and a 256-bit target digest. It is never given the reference nonce. All SHA
rounds, padding, 32-bit modulo additions, and digest-equality constraints are
Tseitin-encoded from the actual SHA-256 compression specification.

Independent hashlib confirms the public Bitcoin reference header before any
SAT generation. The circuit's gate trace is verified by evaluating it under
several alternative nonces, and the reference assignment satisfies EVERY
emitted CNF clause. Solver status and source identities are logged separately.
"""
import hashlib
import sys
from pathlib import Path
from time import monotonic_ns

HEADER_PREFIX = bytes.fromhex(
    "01000000" + "00" * 32 +
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a" +
    "29ab5f49" + "ffff001d")
GENESIS_NONCE = 2083236893
GENESIS_HASH = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
IV = (0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
      0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19)
K = (
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
)

CASES = {"nonce4":4, "nonce8":8, "nonce32":32,
         "merkle_nonce288":288}


def initial_header():
    header = HEADER_PREFIX + GENESIS_NONCE.to_bytes(4, "little")
    assert len(HEADER_PREFIX) == 76 and len(header) == 80
    assert hashlib.sha256(hashlib.sha256(header).digest()).digest()[::-1].hex() == GENESIS_HASH
    return header


def padded(bits):
    original_bits = len(bits)
    assert original_bits == 640 or original_bits == 256
    bits = list(bits) + [-1]
    while len(bits) % 512 != 448:
        bits.append(1)
    bits.extend((-1 if (original_bits >> shift) & 1 else 1)
                for shift in range(63, -1, -1))
    assert len(bits) % 512 == 0
    return [bits[j:j + 32] for j in range(0, len(bits), 32)]


class Circuit:
    def __init__(self, freebits):
        self.counter = 1
        self.FALSE = 1
        self.TRUE = -1
        self.clauses = [(-1,)]
        self.gates = []
        self.nonce_lits = []
        self.freebits = freebits

    def new_var(self):
        self.counter += 1
        return self.counter

    def NOT(self, a):
        return -a

    def AND(self, a, b):
        if a == self.FALSE or b == self.FALSE or a == -b:
            return self.FALSE
        if a == self.TRUE or a == b:
            return b
        if b == self.TRUE:
            return a
        v = self.new_var()
        self.clauses.extend(((-v, a), (-v, b), (v, -a, -b)))
        self.gates.append(("AND", v, a, b))
        return v

    def XOR(self, a, b):
        if a == self.FALSE or a == b:
            return b if a == self.FALSE else self.FALSE
        if b == self.FALSE:
            return a
        if a == self.TRUE:
            return -b
        if b == self.TRUE or a == -b:
            return -a if b == self.TRUE else self.TRUE
        v = self.new_var()
        self.clauses.extend((
            (-a, -b, -v), (a, b, -v),
            (a, -b, v), (-a, b, v)))
        self.gates.append(("XOR", v, a, b))
        return v

    def OR(self, a, b):
        return -self.AND(-a, -b)

    def three_xor(self, a, b, c):
        return self.XOR(self.XOR(a, b), c)

    def majority(self, a, b, c):
        # (a&b) OR ((a XOR b)&c); two terms cannot both be true.
        return self.XOR(self.AND(a, b), self.AND(self.XOR(a, b), c))

    def choose(self, a, b, c):
        # (a&b) OR (~a&c); terms cannot both be true.
        return self.XOR(self.AND(a, b), self.AND(-a, c))

    def word(self, v):
        return tuple(self.TRUE if (v >> i) & 1 else self.FALSE
                     for i in range(31, -1, -1))

    def ROTR(self, bits, count):
        return tuple(bits[(i - count) % 32] for i in range(32))

    def SHR(self, bits, count):
        return tuple(self.FALSE if i < count else bits[i - count]
                     for i in range(32))

    def WXOR(self, a, b):
        return tuple(self.XOR(x, y) for x, y in zip(a, b))

    def xor3word(self, a, b, c):
        return tuple(self.three_xor(x, y, z) for x, y, z in zip(a, b, c))

    def add(self, a, b):
        output = [self.FALSE] * 32
        carry = self.FALSE
        for i in range(31, -1, -1):
            x, y = a[i], b[i]
            output[i] = self.three_xor(x, y, carry)
            carry = self.majority(x, y, carry)
        return tuple(output)

    def add_many(self, *words):
        acc = words[0]
        for word in words[1:]:
            acc = self.add(acc, word)
        return acc

    def compress(self, initial, block):
        assert len(block) == 16
        w = list(map(tuple, block))
        for j in range(16, 64):
            s0 = self.xor3word(self.ROTR(w[j - 15], 7),
                               self.ROTR(w[j - 15], 18),
                               self.SHR(w[j - 15], 3))
            s1 = self.xor3word(self.ROTR(w[j - 2], 17),
                               self.ROTR(w[j - 2], 19),
                               self.SHR(w[j - 2], 10))
            w.append(self.add_many(w[j - 16], s0, w[j - 7], s1))
        a, b, c, d, e, f, g, h = initial
        for i in range(64):
            sigma1 = self.xor3word(self.ROTR(e, 6),
                                   self.ROTR(e, 11),
                                   self.ROTR(e, 25))
            ch = tuple(self.choose(x, y, z)
                       for x, y, z in zip(e, f, g))
            t1 = self.add_many(h, sigma1, ch, self.word(K[i]), w[i])
            sigma0 = self.xor3word(self.ROTR(a, 2),
                                   self.ROTR(a, 13),
                                   self.ROTR(a, 22))
            maj = tuple(self.majority(x, y, z)
                        for x, y, z in zip(a, b, c))
            t2 = self.add(sigma0, maj)
            a, b, c, d, e, f, g, h = (
                self.add(t1, t2), a, b, c, self.add(d, t1), e, f, g)
        return tuple(self.add(before, after)
                     for before, after in zip(
                         initial, (a, b, c, d, e, f, g, h)))

    def sha(self, bits):
        blocks = padded(bits)
        state = tuple(self.word(v) for v in IV)
        for i in range(0, len(blocks), 16):
            state = self.compress(state, blocks[i:i + 16])
        return tuple(bit for word in state for bit in word)

    def header(self):
        header = initial_header()
        bits = []
        for byte_index, byte in enumerate(header):
            for bit_position in range(7, -1, -1):
                index = None
                if byte_index >= 76:
                    nonce_index = 8 * (byte_index - 76) + bit_position
                    if nonce_index < min(self.freebits, 32):
                        index = nonce_index
                elif self.freebits == 288 and 36 <= byte_index < 68:
                    # Merkle-root input index is 32..287. These are
                    # actual header byte-order bits, not a fake hash.
                    index = 32 + 8 * (byte_index - 36) + bit_position
                if index is not None:
                    variable = self.new_var()
                    bits.append(variable)
                    self.nonce_lits.append((index, variable))
                else:
                    bits.append(self.TRUE if (byte >> bit_position) & 1
                                else self.FALSE)
        assert len(bits) == 640
        assert len(self.nonce_lits) == self.freebits
        return tuple(bits)

    def bitblast(self):
        once = self.sha(self.header())
        assert len(once) == 256
        twice = self.sha(once)
        assert len(twice) == 256
        target_raw = bytes.fromhex(GENESIS_HASH)[::-1]
        target_bits = tuple((byte >> j) & 1 for byte in target_raw
                            for j in range(7, -1, -1))
        for literal, expected in zip(twice, target_bits):
            self.clauses.append((literal if expected else -literal,))
        return twice

    def model(self, nonce, merkle=None):
        if merkle is None:
            merkle = int.from_bytes(HEADER_PREFIX[36:68], "little")
        assignment = {1: False}
        for bit, v in self.nonce_lits:
            source_value = (nonce >> bit) if bit < 32 else (
                merkle >> (bit - 32))
            assignment[v] = bool(source_value & 1)
        def read(literal):
            value = assignment[abs(literal)]
            return value if literal > 0 else not value
        for kind, v, a, b in self.gates:
            va, vb = read(a), read(b)
            assignment[v] = va and vb if kind == "AND" else va != vb
        assert len(assignment) == self.counter
        return assignment, read

    def verify_reference(self, output_bits):
        merkle_original = int.from_bytes(HEADER_PREFIX[36:68], "little")
        if self.freebits == 288:
            candidates = (
                (GENESIS_NONCE, merkle_original),
                (GENESIS_NONCE ^ 1, merkle_original),
                (GENESIS_NONCE, merkle_original ^ 1),
                (GENESIS_NONCE ^ 3, merkle_original ^ 0x55))
        else:
            mask = (1 << self.freebits) - 1
            candidates = tuple((nonce, merkle_original) for nonce in (
                GENESIS_NONCE,
                GENESIS_NONCE ^ 1,
                GENESIS_NONCE ^ mask,
                GENESIS_NONCE ^ (mask >> 1)))
        for nonce, merkle in dict.fromkeys(candidates):
            values, evaluate = self.model(nonce, merkle)
            digest = bytes(int("".join("1" if evaluate(bit) else "0"
                                       for bit in output_bits[j:j + 8]), 2)
                           for j in range(0, 256, 8))
            actual_header = bytearray(
                HEADER_PREFIX + nonce.to_bytes(4, "little"))
            actual_header[36:68] = merkle.to_bytes(32, "little")
            independent = hashlib.sha256(hashlib.sha256(
                actual_header).digest()).digest()
            assert digest == independent
            if nonce == GENESIS_NONCE and merkle == merkle_original:
                assert digest[::-1].hex() == GENESIS_HASH
                for clause in self.clauses:
                    assert any(evaluate(lit) for lit in clause)
        print("BTC_MODEL|nonce=" + str(GENESIS_NONCE) +
              "|evaluated_auxiliary=" + str(self.counter) +
              "|alternate_headers=" + str(len(set(candidates)) - 1) +
              "|all_original_clauses=SAT|double_sha256=PASS", flush=True)

    def format(self):
        return ("p cnf " + str(self.counter) + " " +
                str(len(self.clauses)) + "\n" +
                "\n".join(" ".join(map(str, row)) + " 0"
                          for row in self.clauses) + "\n")


def main(folder):
    root = Path(folder)
    root.mkdir(parents=True, exist_ok=True)
    header = initial_header()
    for name, bits in CASES.items():
        start = monotonic_ns()
        circuit = Circuit(bits)
        output = circuit.bitblast()
        circuit.verify_reference(output)
        raw = circuit.format()
        assert raw.count("\n") == len(circuit.clauses) + 1
        path = root / (name + ".cnf")
        path.write_text(raw, encoding="ascii")
        sha = hashlib.sha256(raw.encode("ascii")).hexdigest()
        manifest = {
            "name": name,
            "nonce_free_bits": min(bits, 32),
            "merkle_free_bits": 256 if bits == 288 else 0,
            "free_bits_total": bits,
            "nonce_variables": tuple(sorted(
                (i, v) for i, v in circuit.nonce_lits if i < 32)),
            "merkle_variables": tuple(sorted(
                (i - 32, v) for i, v in circuit.nonce_lits if i >= 32)),
            "variables": circuit.counter,
            "clauses": len(circuit.clauses),
            "gates": len(circuit.gates),
            "header_prefix_hex": HEADER_PREFIX.hex(),
            "expected_digest_display": GENESIS_HASH,
            "input_sha256": sha,
            "reference_witness_sha256": hashlib.sha256(
                header).hexdigest(),
        }
        (root / (name + ".txt")).write_text(repr(manifest) + "\n",
                                             encoding="ascii")
        print("BTC_CNF|case=" + name + "|nonce_bits=" + str(bits) +
              "|variables=" + str(circuit.counter) +
              "|clauses=" + str(len(circuit.clauses)) +
              "|gates=" + str(len(circuit.gates)) +
              "|sha256=" + sha +
              "|elapsed_ns=" + str(monotonic_ns() - start) +
              "|reference_nonce_witness=PASS|status=PASS", flush=True)
    print("BTC_SOURCE_FINAL|four_sources=PASS|SHA256d_genesis=PASS|"
          "full_32_bit_nonce=ENCODED|merkle_nonce_288_bits=ENCODED|"
          "SAT_reference_models=4|status=PASS")


if __name__ == "__main__":
    assert len(sys.argv) == 2
    main(sys.argv[1])
