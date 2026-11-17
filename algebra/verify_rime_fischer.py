#!/usr/bin/env python3
"""Independent BSGS benchmark/checker for classical PH digit-lifting receipts.

This checker implements full-order BSGS rather than calling Pohlig--Hellman.
Finite-group evidence never substitutes for SAT evidence.
"""
import ast
import hashlib
import sys
from math import isqrt
from pathlib import Path


def digest(value):
    return hashlib.sha256(repr(value).encode("ascii")).hexdigest()


def bsgs_full(g, h, p):
    """Independent full-order Shanks method, without prime-power lifting."""
    n = p - 1
    m = isqrt(n - 1) + 1
    seen = {}
    current = 1
    for j in range(m):
        seen.setdefault(current, j)
        current = current * g % p
    inverse_step = pow(g, (-m) % n, p)
    current = h
    for k in range(m + 1):
        if current in seen:
            answer = k * m + seen[current]
            if answer < n and pow(g, answer, p) == h:
                return answer
        current = current * inverse_step % p
    raise AssertionError("unreachable target")


def prime(n):
    if n < 2:
        return False
    i = 2
    while i * i <= n:
        if n % i == 0:
            return False
        i += 1
    return True


def all_cases():
    for p, g in ((97, 5), (193, 5), (1000081, 7)):
        assert prime(p)
        n = p - 1
        for secret in (0, 1, 2, n // 3, n // 2, n - 1,
                       int.from_bytes(hashlib.sha256(
                           ("rime-heldout:" + str(p)).encode("ascii"))
                           .digest()[:6], "big") % n):
            yield p, g, pow(g, secret, p), secret


def parse(line):
    assert line.startswith("RIME_PH|")
    result = {}
    for item in line.strip().split("|")[1:]:
        key, value = item.split("=", 1)
        assert key not in result
        result[key] = value
    assert result["json"] == "0" and result["status"] == "EXACT"
    return result


def verify(row, expected):
    p, g, h, secret = expected
    assert int(row["prime"]) == p and int(row["generator"]) == g
    assert int(row["target"]) == h and int(row["order"]) == p - 1
    observed = int(row["exponent"])
    assert 0 <= observed < p - 1
    assert observed == secret and pow(g, observed, p) == h
    assert observed == bsgs_full(g, h, p)
    factor_rows = ast.literal_eval(row["factors"])
    assert tuple(r[2] for r in factor_rows) == (
        (32, 3) if p == 97 else (64, 3) if p == 193 else (16, 27, 5, 463))
    modulus = 1
    for q, e, tower, residue, digits in factor_rows:
        assert prime(q) and tower == pow(q, e)
        assert all(0 <= x < q for x in digits)
        assert len(digits) == e
        assert sum(x * pow(q, j) for j, x in enumerate(digits)) == residue
        assert residue == observed % tower
        modulus *= tower
    assert modulus == p - 1
    assert row["result_sha256"] == digest((p, g, h, observed, factor_rows))
    assert all(int(row[k]) >= 0 for k in (
        "baby_steps", "giant_steps", "digit_verifications",
        "modular_multiplies", "modular_pow_calls", "elapsed_ns"))


def main(path):
    lines = Path(path).read_text(encoding="ascii").splitlines()
    assert any(line.startswith("RIME_PH_FINAL|cases=21|exact=21|") for line in lines)
    rows = [parse(line) for line in lines if line.startswith("RIME_PH|")]
    expected = tuple(all_cases())
    assert len(rows) == len(expected) == 21
    for row, case in zip(rows, expected):
        verify(row, case)
    negative = dict(rows[0])
    negative["exponent"] = "1"
    try:
        verify(negative, expected[0])
    except AssertionError:
        pass
    else:
        raise AssertionError("forged PH exponent accepted")
    negative = dict(rows[0])
    negative["result_sha256"] = "0" * 64
    try:
        verify(negative, expected[0])
    except AssertionError:
        pass
    else:
        raise AssertionError("forged PH SHA accepted")
    print("RIME_PH_INDEPENDENT|cases=21|full_order_BSGS_agreement=21|"
          "prime_power_digit_CRT=PASS|original_pow_witness=PASS|"
          "corrupted_exponent_and_digest=REJECTED|status=PASS")


if __name__ == "__main__":
    assert len(sys.argv) == 2
    main(sys.argv[1])
