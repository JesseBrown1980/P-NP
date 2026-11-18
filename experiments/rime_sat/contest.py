#!/usr/bin/env python3
"""All 18 archived SAT tests, three source-aligned representations, seven arms.

Run a same-wall-clock bounded exact SAT engine on each transformed source,
and independently check every SAT model after inverse RIME relabeling.
54 runs per arm; 378 total; the random null has matched within-group
permutation radius and matched sign-flip cardinality.
"""
import ast
import hashlib
import sys
from pathlib import Path
from subprocess import run, TimeoutExpired
from time import monotonic_ns

import transform

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT.joinpath("experiments", "bitcoin")))
import genesis

SMALL = (
    "php3_2", "php4_3", "php5_4", "php6_5",
    "php8_7", "php12_11", "php21_20",
    "php3_3", "php4_4", "php6_6",
    "tseitin16", "tseitin32", "tseitin64", "tseitin128",
)
CRYPTO = ("nonce4", "nonce8", "nonce32", "merkle_nonce288")
CASES = SMALL + CRYPTO
ARMS = ("hxf4", "ym_fischer", "rime_new", "rime_original",
        "minisat", "cadical", "picosat")
MODES = transform.MODES
TIME_LIMIT = 12


def model_from_solver(arm, stdout, model_path, n):
    if arm == "minisat":
        terms = model_path.read_text(encoding="ascii").split()
        assert terms and terms[0] == "SAT"
        terms = terms[1:]
    elif arm in ARMS[-2:]:
        terms = [value for line in stdout.splitlines()
                 if line.startswith("v ")
                 for value in line.split()[1:]]
    else:
        terms = model_path.read_text(encoding="ascii").split()
        assert all(word.isdigit() for word in terms)
        assert all(1 <= int(word) <= n for word in terms)
        assert len(set(terms)) == len(terms)
        return set(map(int, terms))
    assert terms
    result = set()
    seen = set()
    terminated = False
    for item in terms:
        value = int(item)
        assert not terminated
        if value == 0:
            terminated = True
        else:
            assert 1 <= abs(value) <= n and abs(value) not in seen
            seen.add(abs(value))
            if value > 0:
                result.add(value)
    assert terminated
    return result


def bitcoin_digest_witness(bits, manifest):
    nonce = genesis.GENESIS_NONCE
    nonce_mask = (1 << manifest["nonce_free_bits"]) - 1
    free_nonce = sum(1 << bit for bit, variable in
                     manifest["nonce_variables"]
                     if bits & (1 << (variable - 1)))
    nonce = (nonce & ~nonce_mask) | free_nonce
    header = bytearray(
        genesis.HEADER_PREFIX + nonce.to_bytes(4, "little"))
    if manifest["merkle_free_bits"]:
        merkle = sum(1 << bit for bit, variable in
                     manifest["merkle_variables"]
                     if bits & (1 << (variable - 1)))
        header[36:68] = merkle.to_bytes(32, "little")
    actual = hashlib.sha256(hashlib.sha256(
        header).digest()).digest()[::-1].hex()
    assert actual == genesis.GENESIS_HASH
    return nonce, bytes(header[36:68]).hex(), hashlib.sha256(
        header).hexdigest()


def run_single(arm, mode, case, inputs, output):
    output.mkdir(parents=True, exist_ok=True)
    original_path = inputs.joinpath("array" if case in SMALL else
                                    "bitcoin", case + ".cnf")
    assert original_path.is_file()
    original = original_path.read_bytes()
    original_sha = hashlib.sha256(original).hexdigest()
    n, clauses = transform.parse(original)
    native = case in SMALL
    expectation = ("SAT" if case in ("php3_3", "php4_4", "php6_6")
                   or case in CRYPTO else "UNSAT")
    description = {"kind":"small", "source_sha256":original_sha}
    if not native:
        manifest = ast.literal_eval(inputs.joinpath(
            "bitcoin", case + ".txt").read_text(encoding="ascii"))
        assert manifest["name"] == case
        assert manifest["input_sha256"] == original_sha
        description = {
            "kind":"bitcoin", "source_sha256":original_sha,
            "original_bitcoin_manifest":manifest,
        }
    td = output.joinpath("transformed.cnf")
    map_path = output.joinpath("mapping.txt")
    source_meta = transform.make(original_path, mode, td, map_path)
    desc = output.joinpath("case.description")
    desc.write_text(repr(description) + "\n", encoding="ascii")
    assert source_meta["original_sha256"] == original_sha
    wn, transformed = transform.parse(td.read_bytes())
    mapping = tuple(map(int, map_path.read_text(encoding="ascii").split()))
    assert wn == n and transform.unchange(transformed, mapping) == clauses
    command = []
    model_path = output.joinpath("solver.model")
    if arm in ARMS[:4]:
        command = [
            sys.executable, "-s", "-B",
            str(ROOT.joinpath("experiments","rime_sat","arm.py")),
            arm, str(td), str(map_path), str(desc), str(model_path)]
    elif arm == "minisat":
        command = ["minisat", "-verb=0", str(td), str(model_path)]
    else:
        command = [arm, str(td)]
    began = monotonic_ns()
    try:
        child = run(command, capture_output=True, check=False,
                    timeout=TIME_LIMIT)
        exit_code = child.returncode
        stdout = child.stdout.decode("ascii", errors="replace")
        stderr = child.stderr.decode("ascii", errors="replace")
    except TimeoutExpired as exc:
        exit_code = 124
        stdout = (exc.stdout or b"").decode("ascii", errors="replace")
        stderr = (exc.stderr or b"").decode("ascii", errors="replace")
    elapsed = monotonic_ns() - began
    status = "HOLD_TIME"
    nodes = "NONE"
    checks = "NONE"
    solver_receipt = "NONE"
    inverse_model_sha = "NONE"
    recovered_nonce = "NONE"
    recovered_merkle = "NONE"
    if exit_code == 124:
        pass
    elif arm in ARMS[:4] and exit_code == 0:
        lines = [line for line in stdout.splitlines()
                 if line.startswith("RIME_SAT_ARM|")]
        assert len(lines) == 1 and lines[0].endswith("|json=0")
        solver_receipt = ast.literal_eval(lines[0].split("|",2)[1])
        status = solver_receipt["status"]
        nodes = solver_receipt["search_nodes"]
        checks = solver_receipt["literal_checks"]
    elif arm in ARMS[4:] and exit_code == 10:
        status = "SAT"
    elif arm in ARMS[4:] and exit_code == 20:
        status = "UNSAT_REPORTED"
    else:
        raise RuntimeError("solver error arm=" + arm + " case=" + case +
                           " mode=" + mode + " exit=" + str(exit_code) +
                           " stderr=" + stderr[-1200:])
    if status == "SAT":
        model = model_from_solver(arm, stdout, model_path, n)
        original_bits = transform.transfer_witness(model, mapping)
        assert transform.valid_model(clauses, original_bits)
        assert transform.valid_model(transformed, sum(
            1 << (v - 1) for v in model))
        if case in CRYPTO:
            recovered_nonce, recovered_merkle, candidate_sha = (
                bitcoin_digest_witness(original_bits, manifest))
        else:
            candidate_sha = "NOT_BITCOIN"
        byte_model = original_bits.to_bytes((n + 7) // 8, "little")
        save = output.joinpath("original-model.bin")
        save.write_bytes(byte_model)
        inverse_model_sha = hashlib.sha256(byte_model).hexdigest()
        assert expectation == "SAT"
    elif status in ("UNSAT_HALL", "UNSAT_XOR",
                    "UNSAT_DPLL", "UNSAT_REPORTED"):
        assert expectation == "UNSAT"
    else:
        assert status in ("HOLD_TIME", "HOLD_BUDGET")
    receipt = dict(arm=arm, case=case, mode=mode,
                   status=status, source_expected=expectation,
                   variables=n, clauses=len(clauses),
                   original_sha256=original_sha,
                   transformed_sha256=source_meta["transformed_sha256"],
                   map_sha256=source_meta["map_sha256"],
                   flips=source_meta["flips"],
                   source_transform_ns=source_meta["elapsed_ns"],
                   clauses_unwound_exact=source_meta["inverse_clauses"],
                   solver_elapsed_ns=elapsed,
                   native_nodes=nodes, native_literal_checks=checks,
                   wall_limit_seconds=TIME_LIMIT,
                   exit_code=exit_code,
                   original_model_sha256=inverse_model_sha,
                   recovered_nonce=recovered_nonce,
                   recovered_merkle_raw_hex=recovered_merkle,
                   solver_receipt=solver_receipt)
    output.joinpath("receipt.txt").write_text(repr(receipt)+"\n",
                                              encoding="ascii")
    output.joinpath("stdout.txt").write_text(stdout[-4000:],encoding="utf-8")
    output.joinpath("stderr.txt").write_text(stderr[-1200:],encoding="utf-8")
    for temporary in (td, map_path, desc, model_path):
        if temporary.exists():
            temporary.unlink()
    print("RIME_SAT_RESULT|arm=" + arm + "|case=" + case +
          "|mode=" + mode + "|status=" + status +
          "|nodes=" + str(nodes) +
          "|elapsed_ns=" + str(elapsed) +
          "|exact_unwind=PASS", flush=True)
    return receipt


def main(arm, root, output):
    assert arm in ARMS
    inputs = Path(root)
    out = Path(output)
    records = []
    for case in CASES:
        for mode in MODES:
            folder = out.joinpath(arm, mode, case)
            records.append(run_single(arm, mode, case, inputs, folder))
    assert len(records) == 54
    assert len(list(out.joinpath(arm).glob("**/receipt.txt"))) == 54
    print("RIME_SAT_ARM_FINAL|arm=" + arm + "|cases=18|"
          "modes=3|observations=54|unwind=54|status=PASS")


if __name__ == "__main__":
    assert len(sys.argv) == 4
    main(*sys.argv[1:])
