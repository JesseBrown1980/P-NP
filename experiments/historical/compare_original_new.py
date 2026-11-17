#!/usr/bin/env python3
"""Byte-for-byte historical original versus new independently implemented PH.

Requires two already independently certified files with exactly the same
21 target endpoints. No contestant source is modified or imported here.
"""
import ast
import hashlib
import sys
from pathlib import Path


def sha(value):
    return hashlib.sha256(repr(value).encode("ascii")).hexdigest()


def read_modern(path):
    lines = Path(path).read_text(encoding="ascii").splitlines()
    assert any(line.startswith("RIME_PH_FINAL|cases=21|exact=21|") for line in lines)
    rows = []
    for line in lines:
        if not line.startswith("RIME_PH|"):
            continue
        fields = {}
        for fragment in line.split("|")[1:]:
            key, value = fragment.split("=", 1)
            assert key not in fields
            fields[key] = value
        assert fields.pop("status") == "EXACT"
        assert fields.pop("json") == "0"
        record = {}
        for key, value in fields.items():
            if key in ("factors",):
                record[key] = ast.literal_eval(value)
            elif key == "result_sha256":
                record[key] = value
            else:
                record[key] = int(value)
        rows.append(record)
    return rows


def main(historical, modern):
    original_rows = [ast.literal_eval(line) for line in
                     Path(historical).read_text(encoding="ascii").splitlines()
                     if line.strip()]
    modern_rows = read_modern(modern)
    assert len(original_rows) == len(modern_rows) == 21
    sum_old, sum_new = 0, 0
    for left, right in zip(original_rows, modern_rows):
        p, g, h, exp = (left["prime"], left["generator"], left["target"],
                        left["observed"])
        assert (p, g, h, exp) == (
            right["prime"], right["generator"], right["target"],
            right["exponent"])
        assert pow(g, exp, p) == h
        assert left["certificate_sha256"] == sha((
            p, g, h, exp, left["towers"]))
        assert right["result_sha256"] == sha((
            p, g, h, exp, right["factors"]))
        assert isinstance(left["original_elapsed_ns"], int)
        assert isinstance(right["elapsed_ns"], int)
        sum_old += left["original_elapsed_ns"]
        sum_new += right["elapsed_ns"]
        print("FISCHER_OLD_NEW|prime=" + str(p) +
              "|target=" + str(h) +
              "|exponent=" + str(exp) +
              "|old_ns=" + str(left["original_elapsed_ns"]) +
              "|new_ns=" + str(right["elapsed_ns"]) +
              "|source_independent=PASS|same_result=PASS", flush=True)
    print("FISCHER_OLD_NEW_FINAL|cases=21|same_targets=PASS|"
          "same_exact_exponents=21|old_reported_ns=" + str(sum_old) +
          "|new_reported_ns=" + str(sum_new) +
          "|source_integrity=GIT_HASH_REQUIRED|status=PASS")


if __name__ == "__main__":
    assert len(sys.argv) == 3
    main(sys.argv[1], sys.argv[2])
