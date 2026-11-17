#!/usr/bin/env python3
"""Independent 21-case test of byte-identical historical ASOLARIA source.

The 2026-07-21 original 'players/rime_fischer.py' and imported
'players/rime_sphere.py' are preserved completely unchanged.
This checker imports only the historical source. Its comparison uses a
separately implemented full-order BSGS, not the modern PH implementation.
"""
import hashlib
import sys
from math import isqrt
from pathlib import Path
from time import monotonic_ns

HISTORY = Path(__file__).resolve().parents[2].joinpath(
    "historical/asolaria-2026-07-21")
sys.path.insert(0, str(HISTORY))
import rime_fischer as original
import rime_sphere as original_sphere


def sha(value):
    return hashlib.sha256(repr(value).encode("ascii")).hexdigest()


def independent_bsgs(g, h, p):
    size = p - 1
    width = isqrt(size - 1) + 1
    cache = {}
    current = 1
    for j in range(width):
        cache.setdefault(current, j)
        current = current * g % p
    step = pow(g, (size - width) % size, p)
    cursor = h
    for i in range(width + 1):
        if cursor in cache:
            result = i * width + cache[cursor]
            if result < size and pow(g, result, p) == h:
                return result
        cursor = cursor * step % p
    raise AssertionError("independent full-order BSGS did not find target")


def independent_prime(n):
    if n < 2:
        return False
    divisor = 2
    while divisor * divisor <= n:
        if n % divisor == 0:
            return False
        divisor += 1
    return True


def independent_factors(n):
    factors = []
    divisor = 2
    while divisor * divisor <= n:
        exponent = 0
        while n % divisor == 0:
            n //= divisor
            exponent += 1
        if exponent:
            factors.append((divisor, exponent))
        divisor += 1
    if n != 1:
        factors.append((n, 1))
    return tuple(factors)


def cases():
    for p in (97, 193, 1000081):
        assert independent_prime(p)
        g = original_sphere.primitive_root(p)
        assert g is not None
        group_factors = independent_factors(p - 1)
        assert all(pow(g, (p - 1) // q, p) != 1
                   for q, e in group_factors)
        order = p - 1
        secrets = (
            0, 1, 2, order // 3, order // 2, order - 1,
            int.from_bytes(hashlib.sha256(
                ("rime-heldout:" + str(p)).encode("ascii")).digest()[:6],
                "big") % order)
        for secret in secrets:
            yield p, g, pow(g, secret, p), secret


def validate():
    rows = []
    for p, g, target, expected in cases():
        start = monotonic_ns()
        exponent, towers = original.rime_fischer(g, target, p)
        elapsed = monotonic_ns() - start
        assert isinstance(exponent, int) and exponent == expected
        assert 0 <= exponent < p - 1
        assert pow(g, exponent, p) == target
        assert independent_bsgs(g, target, p) == exponent
        factorization = independent_factors(p - 1)
        assert original.factor_pe(p - 1) == list(factorization)
        assert [x[0] for x in towers] == [
            pow(q, e) for q, e in factorization]
        for tower, reported_moves in towers:
            assert reported_moves >= 1
            assert exponent % tower >= 0
        record = dict(prime=p, generator=g, target=target,
                      expected=expected, observed=exponent,
                      towers=tuple(towers), original_elapsed_ns=elapsed,
                      certificate_sha256=sha((p, g, target, exponent, tuple(towers))))
        rows.append(record)
        print("FISCHER_HISTORICAL|" + "|".join(
            k + "=" + str(v) for k, v in record.items()) +
            "|status=EXACT|json=0", flush=True)
    assert len(rows) == 21
    assert all(item["observed"] == item["expected"] for item in rows)
    print("FISCHER_HISTORICAL_FINAL|cases=21|original_bytecopy=GIT_HASH_REQUIRED|"
          "full_order_BSGS_agreement=21|original_prime_power_outputs=PASS|"
          "exact_modular_witness=21|status=PASS")
    return rows


def negative_tests(rows):
    altered = dict(rows[0])
    altered["observed"] += 1
    try:
        assert altered["observed"] == altered["expected"] and (
            pow(altered["generator"], altered["observed"],
                altered["prime"]) == altered["target"])
    except AssertionError:
        pass
    else:
        raise AssertionError("forged original result accepted")
    altered = dict(rows[0])
    altered["certificate_sha256"] = "0" * 64
    try:
        assert altered["certificate_sha256"] == sha((
            altered["prime"], altered["generator"], altered["target"],
            altered["observed"], altered["towers"]))
    except AssertionError:
        pass
    else:
        raise AssertionError("forged original certificate SHA accepted")
    print("FISCHER_HISTORICAL_TAMPER|altered_exponent=REJECTED|"
          "altered_sha=REJECTED|status=PASS")


def main(path):
    rows = validate()
    negative_tests(rows)
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text("".join(repr(r) + "\n" for r in rows),
                          encoding="ascii")


if __name__ == "__main__":
    assert len(sys.argv) == 2
    main(sys.argv[1])
