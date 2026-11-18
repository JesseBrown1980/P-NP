#!/usr/bin/env python3
"""Sandboxed hosted Bitcoin SHA256d SAT contest, all seven arms.

Same raw original-CNF source and declared independent time budgets.
A SAT answer is independently tested against the original 80-byte header,
its double-SHA256 digest and every SAT clause. A false UNSAT is a failure.
Timeouts are preserved as HOLD, never recast into failure or success.
"""
import ast
import hashlib
import sys
from pathlib import Path
from subprocess import run
from time import monotonic_ns

import genesis


ARMS = ("hxf4", "ym_fischer", "rime_new", "rime_original",
        "minisat", "cadical", "picosat")
LIMIT_NATIVE = 20
LIMIT_EXTERNAL = 12


def parse_model(arm, stdout, minisat_path, variables):
    if arm == "minisat":
        lines = minisat_path.read_text(encoding="ascii").split()
        assert lines and lines[0] == "SAT"
        words = lines[1:]
    else:
        words = [word for line in stdout.splitlines()
                 if line.startswith("v ") for word in line.split()[1:]]
    assert words
    model = {}
    final = False
    for token in words:
        v = int(token)
        assert not final
        if v == 0:
            final = True
        else:
            assert 1 <= abs(v) <= variables and abs(v) not in model
            model[abs(v)] = v > 0
    assert final
    return model


def source_clauses(path):
    n = announced = None
    clauses, row = [], []
    for line in path.read_text(encoding="ascii").splitlines():
        fields = line.split()
        if not fields or fields[0] == "c":
            continue
        if fields[0] == "p":
            assert n is None and fields[1] == "cnf"
            n, announced = int(fields[2]), int(fields[3])
            continue
        for term in fields:
            v = int(term)
            if v == 0:
                assert row
                clauses.append(tuple(row))
                row = []
            else:
                assert 0 < abs(v) <= n
                row.append(v)
    assert n and len(clauses) == announced and not row
    return n, clauses


def original_model_check(case, manifest, cnf, model):
    n, clauses = source_clauses(cnf)
    assert n == manifest["variables"] and len(clauses) == manifest["clauses"]
    assert all(any(model.get(abs(v)) == (v > 0)
                   for v in clause if abs(v) in model)
               for clause in clauses)
    nonce_mask = (1 << manifest["nonce_free_bits"]) - 1
    recovered = sum(1 << bit for bit, var in manifest["nonce_variables"]
                    if model.get(var) is True)
    nonce = (genesis.GENESIS_NONCE & ~nonce_mask) | recovered
    header = bytearray(
        genesis.HEADER_PREFIX + nonce.to_bytes(4, "little"))
    if manifest["merkle_free_bits"]:
        merkle = sum(1 << bit for bit, var in
                     manifest["merkle_variables"] if model.get(var) is True)
        header[36:68] = merkle.to_bytes(32, "little")
    digest = hashlib.sha256(hashlib.sha256(
        header).digest()).digest()[::-1].hex()
    assert digest == genesis.GENESIS_HASH
    return nonce, bytes(header[36:68]).hex(), hashlib.sha256(
        header).hexdigest()


def main(arm, case, source_root, out_root):
    assert arm in ARMS and case in genesis.CASES
    source = Path(source_root)
    out = Path(out_root)
    out.mkdir(parents=True, exist_ok=True)
    cnf = source.joinpath(case + ".cnf")
    manifest = ast.literal_eval(
        (source.joinpath(case + ".txt")).read_text(encoding="ascii"))
    assert manifest["name"] == case
    raw_sha = hashlib.sha256(cnf.read_bytes()).hexdigest()
    assert raw_sha == manifest["input_sha256"]
    begin = monotonic_ns()
    if arm in ARMS[:4]:
        command = [sys.executable, "-s", "-B",
                   str(Path(__file__).resolve().parent.joinpath("arm.py")),
                   arm, case, str(cnf),
                   str(source.joinpath(case + ".txt"))]
        limit = LIMIT_NATIVE
    else:
        limit = LIMIT_EXTERNAL
        if arm == "minisat":
            command = ["minisat", "-verb=0", str(cnf), str(out.joinpath("model.sol"))]
        else:
            command = [arm, str(cnf)]
    command = ["/usr/bin/time", "-f", "%M", "-o", str(out.joinpath("rss.txt")),
               "timeout", "--kill-after=3s", str(limit) + "s"] + command
    result = run(command, capture_output=True, check=False)
    elapsed = monotonic_ns() - begin
    stdout = result.stdout.decode("ascii", errors="replace")
    stderr = result.stderr.decode("ascii", errors="replace")
    (out.joinpath("solver.stdout")).write_text(stdout, encoding="utf-8")
    (out.joinpath("solver.stderr")).write_text(stderr, encoding="utf-8")
    status = "HOLD"
    solver_receipt = None
    nonce = "NONE"
    witness_sha = "NONE"
    merkle_hex = "NONE"
    if result.returncode == 124:
        status = "HOLD_TIME"
    elif arm in ARMS[:4] and result.returncode == 0:
        reports = [line for line in stdout.splitlines()
                   if line.startswith("BITCOIN_ARM|")]
        assert len(reports) == 1 and reports[0].endswith("|json=0")
        solver_receipt = ast.literal_eval(reports[0].split("|", 2)[1])
        assert solver_receipt["arm"] == arm
        assert solver_receipt["case"] == case
        assert solver_receipt["cnf_sha256"] == raw_sha
        status = solver_receipt["status"]
        assert status in ("SAT", "HOLD_BUDGET")
        if status == "SAT":
            nonce = solver_receipt["recovered_nonce"]
            witness_sha = solver_receipt["header_witness_sha256"]
            merkle_hex = solver_receipt["recovered_merkle_raw_hex"]
    elif arm in ARMS[4:] and result.returncode == 10:
        model = parse_model(arm, stdout, out.joinpath("model.sol"),
                            manifest["variables"])
        nonce, merkle_hex, witness_sha = original_model_check(
            case, manifest, cnf, model)
        status = "SAT"
    elif arm in ARMS[4:] and result.returncode == 20:
        raise AssertionError("external UNSAT contradicts independently known SAT genesis nonce")
    else:
        raise RuntimeError(
            "solver failed " + arm + " " + case +
            " rc=" + str(result.returncode) +
            " last stderr=" + stderr[-2000:])
    memory_text = (out.joinpath("rss.txt")).read_text(encoding="ascii").splitlines()
    memory = int(memory_text[-1]) if memory_text and memory_text[-1].isdigit() else -1
    summary = dict(arm=arm, case=case, status=status,
                   cnf_sha256=raw_sha, source_bits=manifest["nonce_free_bits"],
                   variables=manifest["variables"],
                   clauses=manifest["clauses"],
                   solver_exit=result.returncode, elapsed_ns=elapsed,
                   memory_kib=memory, limit_seconds=limit,
                   recovered_nonce=nonce,
                   recovered_merkle_raw_hex=merkle_hex,
                   witness_sha256=witness_sha,
                   source_receipt=solver_receipt)
    (out.joinpath("receipt.txt")).write_text(repr(summary) + "\n",
                                      encoding="ascii")
    print("BTC_COMPETITION|" + repr({
        k: v for k, v in summary.items() if k != "source_receipt"
    }) + "|json=0", flush=True)


if __name__ == "__main__":
    assert len(sys.argv) == 5
    main(*sys.argv[1:])
