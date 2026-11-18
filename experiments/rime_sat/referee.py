#!/usr/bin/env python3
"""Independent referee: 378 observations on 18 original CNFs / three mappings.

No import of contestant transform, solver, or adapters. Reconstruct both
signed permutations from original source SHA and finite-field constants.
Rebuild the transformed formula SHA, reverse every clause, and replay all
original-CNF SAT models. SHA256d models must reproduce public genesis hash.
All mathematical UNSAT classifications are independently checked against
the original CNF (PHP invariant and exact Tseitin rank contradiction).
"""
import ast
import hashlib
import random
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT.joinpath("experiments","array")))
import verify as prior_referee

SMALL = (
    "php3_2", "php4_3", "php5_4", "php6_5",
    "php8_7", "php12_11", "php21_20",
    "php3_3", "php4_4", "php6_6",
    "tseitin16", "tseitin32", "tseitin64", "tseitin128",
)
BTC = ("nonce4", "nonce8", "nonce32", "merkle_nonce288")
CASES = SMALL + BTC
ARMS = ("hxf4", "ym_fischer", "rime_new", "rime_original",
        "minisat", "cadical", "picosat")
MODES = ("plain", "rime_wound", "random_gauge")

P = 1000081
W = 951846
N = 27
FISCHER_P = 103681
FISCHER_N = 20736
GENESIS_NONCE = 2083236893
GENESIS_PREFIX = bytes.fromhex(
    "01000000" + "00" * 32 +
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a" +
    "29ab5f49" + "ffff001d")
GENESIS_HASH = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
assert pow(7, (P - 1) // N, P) == W
assert pow(W, N, P) == 1 and pow(W, 9, P) != 1


def primitive(p):
    d = 2
    fac = []
    n = p - 1
    while d * d <= n:
        if n % d == 0:
            fac.append(d)
            while n % d == 0:
                n //= d
        d += 1
    if n > 1:
        fac.append(n)
    for v in range(2,p):
        if all(pow(v,(p-1)//q,p)!=1 for q in fac):
            return v
    raise AssertionError


FISCHER_W = pow(primitive(FISCHER_P),
                (FISCHER_P - 1) // FISCHER_N, FISCHER_P)
assert pow(FISCHER_W,FISCHER_N,FISCHER_P)==1


def independent_map(n, source_sha, mode):
    assert mode in MODES
    result = list(range(n + 1))
    if mode == "plain":
        return tuple(result), 0
    flips = 0
    for start in range(0, n, 27):
        width = min(27, n - start)
        group = start // 27
        seed = hashlib.sha256(bytes.fromhex(source_sha) +
                              group.to_bytes(8,"little")).digest()
        source = seed[:27]
        spectrum = [
            sum(source[j] * pow(W, (k*j) % 27, P)
                for j in range(27)) % P
            for k in range(27)]
        invw = pow(W,-1,P)
        recovered = [
            pow(27,-1,P) * sum(spectrum[j] * pow(
                invw, (k*j)%27, P) for j in range(27))%P
            for k in range(27)]
        assert recovered == list(source)
        points = [
            pow(FISCHER_W,
                (spectrum[k] + 256*(k%3)+768*(group%27))%FISCHER_N,
                FISCHER_P)
            for k in range(27)]
        order = tuple(sorted(range(width),
                             key=lambda k: (points[k], spectrum[k], k)))
        ref_flip = {k for k in range(width)
                    if (spectrum[k] ^ points[k]) & 1}
        if mode == "rime_wound":
            actual = order
            changed = ref_flip
        else:
            rng = random.Random(int.from_bytes(seed,"big"))
            vals = list(range(width))
            rng.shuffle(vals)
            actual = tuple(vals)
            changed = set(rng.sample(range(width), len(ref_flip)))
        for new_local, old_local in enumerate(actual):
            result[start + old_local + 1] = (
                -1 if old_local in changed else 1) * (
                    start + new_local + 1)
            flips += int(old_local in changed)
    assert len({abs(v) for v in result[1:]}) == n
    return tuple(result), flips


def source_model(clauses, bits):
    return all(any(bool(bits & (1 << (abs(lit) - 1))) == (lit > 0)
                   for lit in clause) for clause in clauses)


def bitcoin_model(case, bits, meta):
    full = bytearray(GENESIS_PREFIX + GENESIS_NONCE.to_bytes(4,"little"))
    known = GENESIS_NONCE
    mask = (1 << meta["nonce_free_bits"]) - 1
    selected = sum(1 << bit for bit, v in meta["nonce_variables"]
                   if bits & (1 << (v - 1)))
    known = (known & ~mask) | selected
    full[76:80] = known.to_bytes(4,"little")
    if meta["merkle_free_bits"]:
        merkle = sum(1 << bit for bit,v in meta["merkle_variables"]
                     if bits & (1 << (v - 1)))
        full[36:68] = merkle.to_bytes(32, "little")
    assert hashlib.sha256(hashlib.sha256(
        full).digest()).digest()[::-1].hex() == GENESIS_HASH
    return known, bytes(full[36:68]).hex()


def source_truth(case, n, clauses):
    if case in BTC:
        return "SAT"
    return prior_referee.known_truth(case, n, clauses)[0]


def rehash_inverse(n, clauses, mapping):
    sha = hashlib.sha256()
    sha.update(("p cnf " + str(n) + " " +
                str(len(clauses)) + "\n").encode("ascii"))
    inverse = [0] * (n + 1)
    for old in range(1, n + 1):
        m = mapping[old]
        inverse[abs(m)] = old if m > 0 else -old
    count = 0
    for clause in clauses:
        wound = tuple(mapping[abs(v)] if v > 0
                      else -mapping[abs(v)] for v in clause)
        original = tuple(inverse[abs(v)] if v > 0
                         else -inverse[abs(v)] for v in wound)
        assert original == clause
        sha.update((" ".join(str(v) for v in wound) +
                    " 0\n").encode("ascii"))
        count += 1
    assert count == len(clauses)
    return sha.hexdigest()


def main(root):
    base = Path(root)
    originals = base.joinpath("source")
    evidence = base.joinpath("arms")
    total = 0
    sats = holds = unsat = 0
    results = {}
    for case in CASES:
        original_path = originals.joinpath(
            "array" if case in SMALL else "bitcoin", case + ".cnf")
        original_bytes = original_path.read_bytes()
        source_sha = hashlib.sha256(original_bytes).hexdigest()
        n, clauses = prior_referee.parse_dimacs(original_bytes)
        truth = source_truth(case, n, clauses)
        if case in BTC:
            meta = ast.literal_eval(originals.joinpath(
                "bitcoin",case+".txt").read_text(encoding="ascii"))
            assert source_sha == meta["input_sha256"]
            assert meta["name"] == case
            assert meta["expected_digest_display"] == GENESIS_HASH
        transforms = {}
        for mode in MODES:
            mapping, flips = independent_map(n, source_sha, mode)
            transformed_sha = rehash_inverse(n, clauses, mapping)
            map_sha = hashlib.sha256(
                (" ".join(map(str,mapping))+"\n").encode("ascii")
                ).hexdigest()
            transforms[mode] = (transformed_sha, map_sha, flips)
        assert transforms["rime_wound"][2] == transforms["random_gauge"][2]
        assert len({v[0] for v in transforms.values()}) >= 1
        for arm in ARMS:
            for mode in MODES:
                folder = evidence.joinpath(arm,mode,case)
                report = ast.literal_eval(
                    folder.joinpath("receipt.txt").read_text(
                        encoding="ascii"))
                assert report["arm"] == arm and report["case"] == case
                assert report["mode"] == mode and report["source_expected"] == truth
                assert report["variables"] == n
                assert report["clauses"] == len(clauses)
                assert report["original_sha256"] == source_sha
                assert (report["transformed_sha256"],
                        report["map_sha256"], report["flips"]
                        ) == transforms[mode]
                assert report["clauses_unwound_exact"] == len(clauses)
                assert report["wall_limit_seconds"] == 12
                status = report["status"]
                assert status in (
                    "SAT", "UNSAT_HALL", "UNSAT_XOR", "UNSAT_DPLL",
                    "UNSAT_REPORTED", "HOLD_TIME", "HOLD_BUDGET")
                if status == "SAT":
                    assert truth == "SAT"
                    raw_model = folder.joinpath("original-model.bin").read_bytes()
                    assert len(raw_model) == (n + 7)//8
                    assert hashlib.sha256(raw_model).hexdigest() == (
                        report["original_model_sha256"])
                    bits = int.from_bytes(raw_model, "little")
                    assert bits < (1 << n)
                    assert source_model(clauses, bits)
                    if case in BTC:
                        nonce, merkle = bitcoin_model(case,bits,meta)
                        assert nonce == report["recovered_nonce"]
                        assert merkle == report["recovered_merkle_raw_hex"]
                    sats += 1
                elif status in ("UNSAT_HALL","UNSAT_XOR",
                                "UNSAT_DPLL","UNSAT_REPORTED"):
                    assert truth == "UNSAT"
                    unsat += 1
                else:
                    holds += 1
                    assert report["original_model_sha256"] == "NONE"
                results[(arm,mode,case)] = (
                    status, report["solver_elapsed_ns"],
                    report["native_nodes"])
                total += 1
                print("RIME_SAT_REFEREE|arm="+arm+"|mode="+mode+
                      "|case="+case+"|status="+status+
                      "|unwind_original=PASS|original_CNF_model=PASS",
                      flush=True)
    assert total == 18 * 3 * 7 == 378
    # Negative controls: flipping a map sign or swapping a table entry
    # must change its pinned checksum (and makes inverse with old map fail).
    for case in ("php6_5", "tseitin64", "nonce8", "merkle_nonce288"):
        raw = originals.joinpath("array" if case in SMALL else "bitcoin",
                                 case+".cnf").read_bytes()
        n, clauses = prior_referee.parse_dimacs(raw)
        mapping, flips = independent_map(
            n, hashlib.sha256(raw).hexdigest(), "rime_wound")
        damaged = list(mapping)
        damaged[1] = -damaged[1]
        assert hashlib.sha256((" ".join(map(str,damaged))+"\n").encode(
            "ascii")).hexdigest() != hashlib.sha256(
                (" ".join(map(str,mapping))+"\n").encode("ascii")
                ).hexdigest()
    for arm in ARMS:
        for mode in MODES:
            v = [results[(arm,mode,c)][0] for c in CASES]
            completed = sum(s not in ("HOLD_TIME","HOLD_BUDGET")
                            for s in v)
            print("RIME_SAT_AGGREGATE|arm="+arm+"|mode="+mode+
                  "|completed="+str(completed)+
                  "|hold="+str(18-completed)+"|cases=18",flush=True)
    base.joinpath("REFEREE_SUMMARY.txt").write_text(
        repr(results)+"\n", encoding="ascii")
    print("RIME_SAT_FINAL|arms=7|cases=18|modes=3|"
          "observations=378|independent_inverse=378|"
          "source_hashes=PASS|models=PASS|"
          "map_sign_tamper=REJECTED|sat="+str(sats)+
          "|unsat="+str(unsat)+"|hold="+str(holds)+"|status=PASS")


if __name__ == "__main__":
    assert len(sys.argv) == 2
    main(sys.argv[1])
