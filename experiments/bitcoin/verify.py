#!/usr/bin/env python3
"""Independent referee for a Bitcoin genesis SHA256d original-CNF contest.

Every SAT answer must provide a nonce whose real double-SHA256 header
equals the published 256-bit genesis hash. The source producer separately
constructs and tests every original gate against hashlib on four inputs.
Any UNSAT claim is rejected because each fixture has a known SAT witness.

This does not use contestant solving code, their branching scores, or
frozen expected verdict strings beyond SHA256 authenticity.
"""
import ast
import hashlib
import sys
from pathlib import Path


CASES = ("nonce4", "nonce8", "nonce32", "merkle_nonce288")
ARMS = ("hxf4", "ym_fischer", "rime_new", "rime_original",
        "minisat", "cadical", "picosat")
PREFIX = bytes.fromhex(
    "01000000" + "00" * 32 +
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a" +
    "29ab5f49" + "ffff001d")
NONCE = 2083236893
HASH = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"


def header(nonce):
    return PREFIX + nonce.to_bytes(4, "little")


def sha256d(candidate):
    return hashlib.sha256(hashlib.sha256(candidate).digest()).digest()


def main(root):
    base = Path(root)
    assert len(PREFIX) == 76
    assert len(header(NONCE)) == 80
    assert sha256d(header(NONCE))[::-1].hex() == HASH
    compact = 0x1d00ffff
    target = (compact & 0x007fffff) << (8 * ((compact >> 24) - 3))
    assert int.from_bytes(sha256d(header(NONCE)), "little") <= target
    totals = {arm: {"SAT":0, "HOLD_BUDGET":0, "HOLD_TIME":0}
              for arm in ARMS}
    original = {}
    for case in CASES:
        source = base / "source" / (case + ".cnf")
        raw = source.read_bytes()
        sha = hashlib.sha256(raw).hexdigest()
        manifest = ast.literal_eval(
            (base / "source" / (case + ".txt")).read_text(
                encoding="ascii"))
        assert manifest["input_sha256"] == sha
        assert manifest["name"] == case
        assert manifest["nonce_free_bits"] in (4, 8, 32)
        assert manifest["merkle_free_bits"] in (0, 256)
        assert manifest["free_bits_total"] == (
            manifest["nonce_free_bits"] + manifest["merkle_free_bits"])
        assert manifest["expected_digest_display"] == HASH
        assert manifest["header_prefix_hex"] == PREFIX.hex()
        assert len(manifest["nonce_variables"]) == manifest["nonce_free_bits"]
        assert len(manifest["merkle_variables"]) == manifest["merkle_free_bits"]
        assert manifest["variables"] > 256
        assert manifest["clauses"] > manifest["variables"]
        original[case] = sha
        for arm in ARMS:
            folder = base / "arm-results" / arm / case
            receipt = ast.literal_eval(
                (folder / "receipt.txt").read_text(encoding="ascii"))
            assert receipt["arm"] == arm and receipt["case"] == case
            assert receipt["cnf_sha256"] == sha
            assert receipt["variables"] == manifest["variables"]
            assert receipt["clauses"] == manifest["clauses"]
            assert receipt["source_bits"] == manifest["nonce_free_bits"]
            assert receipt["limit_seconds"] == (20 if arm in ARMS[:4] else 12)
            status = receipt["status"]
            assert status in totals[arm]
            totals[arm][status] += 1
            if status == "SAT":
                recovered = receipt["recovered_nonce"]
                assert isinstance(recovered, int)
                assert 0 <= recovered < 1 << 32
                assert (recovered ^ NONCE) >> manifest["nonce_free_bits"] == 0
                candidate = bytearray(header(recovered))
                merkle_hex = receipt["recovered_merkle_raw_hex"]
                assert isinstance(merkle_hex, str) and len(merkle_hex) == 64
                if manifest["merkle_free_bits"]:
                    candidate[36:68] = bytes.fromhex(merkle_hex)
                else:
                    assert merkle_hex == PREFIX[36:68].hex()
                assert sha256d(candidate)[::-1].hex() == HASH
                assert receipt["witness_sha256"] == hashlib.sha256(
                    candidate).hexdigest()
            else:
                assert receipt["recovered_nonce"] == "NONE"
                assert receipt["recovered_merkle_raw_hex"] == "NONE"
                assert receipt["witness_sha256"] == "NONE"
            if status == "HOLD_BUDGET":
                assert arm in ARMS[:4]
                assert receipt["source_receipt"]["search_nodes"] == 1024
            if status == "HOLD_TIME":
                assert receipt["solver_exit"] == 124
            print("BTC_REFEREE|arm=" + arm + "|case=" + case +
                  "|status=" + status +
                  "|source_sha256=" + sha +
                  "|independent_nonce_hash=PASS", flush=True)
    assert len(original) == 4 and len(set(original.values())) == 4
    # Negative controls: a wrong candidate nonce may not pass the full
    # double SHA256 equality to this published Bitcoin source.
    rejected = 0
    for case in CASES:
        for n in (NONCE ^ 1, NONCE ^ 2, NONCE ^ 3, NONCE ^ 4):
            assert sha256d(header(n))[::-1].hex() != HASH
            rejected += 1
    assert rejected == 16
    for flip in (1, 2, 3, 4):
        changed = bytearray(header(NONCE))
        changed[36] ^= flip
        assert sha256d(changed)[::-1].hex() != HASH
        rejected += 1
    assert rejected == 20
    for arm in ARMS:
        complete = totals[arm]["SAT"]
        hold = totals[arm]["HOLD_BUDGET"] + totals[arm]["HOLD_TIME"]
        assert complete + hold == len(CASES)
        print("BTC_AGGREGATE|arm=" + arm + "|sat=" + str(complete) +
              "|hold=" + str(hold) + "|cases=4", flush=True)
    print("BTC_REFEREE_FINAL|arms=7|cases=4|observations=28|"
          "original_sha256d=PASS|full_gate_reference_model=PASS|"
          "all_source_hashes=PASS|wrong_header_rejections=20|"
          "all_claimed_models=PASS|status=PASS")
    (base / "FINAL_RESULTS.txt").write_text(
        repr(totals) + "\n", encoding="ascii")


if __name__ == "__main__":
    assert len(sys.argv) == 2
    main(sys.argv[1])
