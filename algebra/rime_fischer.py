#!/usr/bin/env python3
"""Rime Fischer number-theoretic candidate.

Classical Pohlig--Hellman with prime-power digit lifting, order-q Shanks
baby-step/giant-step, and exact CRT. Derived from Asolaria Law 14, separately
from the later Fischer CPL anti-blunder evaluator. Exact integers only.
"""
import hashlib
from math import isqrt
from time import monotonic_ns


def sha(value):
    return hashlib.sha256(repr(value).encode("ascii")).hexdigest()


def is_prime(n):
    if n < 2:
        return False
    if n % 2 == 0:
        return n == 2
    d = 3
    while d * d <= n:
        if n % d == 0:
            return False
        d += 2
    return True


def factor(n):
    assert n > 1
    factors = []
    q = 2
    while q * q <= n:
        power = 0
        while n % q == 0:
            n //= q
            power += 1
        if power:
            assert is_prime(q)
            factors.append((q, power))
        q = 3 if q == 2 else q + 2
    if n > 1:
        assert is_prime(n)
        factors.append((n, 1))
    return tuple(factors)


def primitive_root(p):
    assert is_prime(p) and p >= 3
    order = p - 1
    factors = factor(order)
    for g in range(2, p):
        if all(pow(g, order // q, p) != 1 for q, _ in factors):
            return g
    raise AssertionError("primitive root missing")


def bsgs(base, target, p, order_q, work):
    """Solve one order-q discrete log, including an exact group check."""
    assert pow(base, order_q, p) == 1
    assert pow(target, order_q, p) == 1
    m = isqrt(order_q - 1) + 1
    baby = {}
    value = 1
    for j in range(m):
        baby.setdefault(value, j)
        value = value * base % p
        work["baby_steps"] += 1
        work["modular_multiplies"] += 1
    jump = pow(pow(base, m, p), -1, p)
    value = target
    for i in range(m + 1):
        work["giant_steps"] += 1
        if value in baby:
            x = i * m + baby[value]
            if x < order_q and pow(base, x, p) == target:
                return x
        value = value * jump % p
        work["modular_multiplies"] += 1
    raise AssertionError("digit not in generated subgroup")


def combine_crt(congruences):
    """Exact CRT; independent pairwise-coprime prime-power residues."""
    modulus = 1
    for residue, m in congruences:
        assert 0 <= residue < m and m > 0
        for _, previous_m in congruences:
            if previous_m == m and len(congruences) > 1:
                raise AssertionError("repeated modulus")
        modulus *= m
    x = 0
    for residue, m in congruences:
        part = modulus // m
        x += residue * part * pow(part, -1, m)
    x %= modulus
    assert all(x % m == residue for residue, m in congruences)
    return x, modulus


def recover(p, g, h):
    assert is_prime(p) and 1 < g < p and 0 < h < p
    order = p - 1
    factors = factor(order)
    assert all(pow(g, order // q, p) != 1 for q, _ in factors)
    start = monotonic_ns()
    work = {"baby_steps": 0, "giant_steps": 0, "modular_multiplies": 0,
            "digit_verifications": 0, "modular_pow_calls": 0}
    recovered = []
    for q, e in factors:
        tower = pow(q, e)
        base = pow(g, order // tower, p)
        value = pow(h, order // tower, p)
        order_q = pow(base, pow(q, e - 1), p)
        x = 0
        digits = []
        for j in range(e):
            remainder = value * pow(pow(base, x, p), -1, p) % p
            observed = pow(remainder, pow(q, e - j - 1), p)
            work["modular_pow_calls"] += 4
            digit = bsgs(order_q, observed, p, q, work)
            assert pow(order_q, digit, p) == observed
            work["digit_verifications"] += 1
            digits.append(digit)
            x += digit * pow(q, j)
        assert pow(base, x, p) == value
        recovered.append((q, e, tower, x, tuple(digits)))
    exponent, modulus = combine_crt(tuple((row[3], row[2]) for row in recovered))
    assert modulus == order and pow(g, exponent, p) == h
    output = dict(prime=p, generator=g, target=h, order=order,
                  factors=tuple(recovered), exponent=exponent,
                  baby_steps=work["baby_steps"], giant_steps=work["giant_steps"],
                  digit_verifications=work["digit_verifications"],
                  modular_multiplies=work["modular_multiplies"],
                  modular_pow_calls=work["modular_pow_calls"],
                  elapsed_ns=monotonic_ns() - start)
    output["result_sha256"] = sha((p, g, h, exponent, output["factors"]))
    return output


def probe_targets():
    # Same inputs for every discrete-log comparator.
    for p in (97, 193, 1000081):
        g = primitive_root(p)
        order = p - 1
        chosen = (0, 1, 2, order // 3, order // 2, order - 1,
                  int.from_bytes(hashlib.sha256(
                      ("rime-heldout:" + str(p)).encode("ascii")).digest()[:6],
                      "big") % order)
        for secret in chosen:
            yield p, g, pow(g, secret, p), secret


if __name__ == "__main__":
    for prime, base, target, expected in probe_targets():
        answer = recover(prime, base, target)
        assert answer["exponent"] == expected
        print("RIME_PH|" + "|".join(str(k) + "=" + str(v)
              for k, v in answer.items()) + "|status=EXACT|json=0")
    print("RIME_PH_FINAL|cases=21|exact=21|verification=POW_MOD_P|"
          "independent_checker=REQUIRED|status=RESEARCH_PILOT")
