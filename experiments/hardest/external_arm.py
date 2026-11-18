#!/usr/bin/env python3
"""Externally metered SAT decisions and independently checked CNF witnesses.

CaDiCaL UNSAT results are CERTIFIED only when drat-trim verifies the exact
raw original CNF against its produced proof. Other UNSAT decisions remain
unverified claims in this benchmark, never silently promoted to certificates.
"""
import hashlib
import sys
from pathlib import Path
from subprocess import run
from time import monotonic_ns

import challenge


LIMIT_SECONDS = 50
CERT_LIMIT_SECONDS = 30


def timed(command, stdout_path, stderr_path, memory_path, seconds):
    args = ["/usr/bin/time", "-f", "%M", "-o", str(memory_path),
            "timeout", "--kill-after=5s", str(seconds) + "s"] + command
    t = monotonic_ns()
    with stdout_path.open("wb") as output, stderr_path.open("wb") as errors:
        p = run(args, stdout=output, stderr=errors, check=False)
    elapsed = monotonic_ns() - t
    memory = memory_path.read_text(encoding="ascii").strip().splitlines()
    rss = int(memory[-1]) if memory and memory[-1].isdigit() else -1
    return p.returncode, elapsed, rss


def parse_model(tokens, n):
    assigned = {}
    stop = False
    for token in tokens:
        lit = int(token)
        assert not stop
        if lit == 0:
            stop = True
        else:
            assert 1 <= abs(lit) <= n and abs(lit) not in assigned
            assigned[abs(lit)] = lit > 0
    assert stop and assigned
    return assigned


def witness_check(clauses, model):
    # Partial assignments are accepted only when every original clause
    # contains an assigned literal that is true.
    return all(any(model.get(abs(lit)) == (lit > 0)
                   for lit in clause if abs(lit) in model)
               for clause in clauses)


def original_model(arm, folder, n):
    if arm == "minisat":
        tokens = folder.joinpath("model.sol").read_text(encoding="ascii").split()
        assert tokens and tokens[0] == "SAT"
        return parse_model(tokens[1:], n)
    stdout = folder.joinpath("solver.stdout").read_text(encoding="ascii")
    vectors = [line.split()[1:] for line in stdout.splitlines()
               if line.startswith("v ")]
    assert vectors
    return parse_model([v for line in vectors for v in line], n)


def run_external(arm, case, cnf, archive, drat_path):
    original = cnf.read_bytes()
    n, clauses = challenge.parse(original.decode("ascii"))
    assert (n, clauses) == challenge.original(case)
    archive.mkdir(parents=True, exist_ok=True)
    stdout = archive / "solver.stdout"
    stderr = archive / "solver.stderr"
    memory = archive / "solver.rss"
    proof = archive / "unsat.drat"
    if arm == "minisat":
        args = ["minisat", "-verb=0", str(cnf), str(archive / "model.sol")]
    elif arm == "picosat":
        args = ["picosat", str(cnf)]
    else:
        assert arm == "cadical"
        args = ["cadical", "--no-binary", str(cnf), str(proof)]
    status_code, elapsed, peak = timed(
        args, stdout, stderr, memory, LIMIT_SECONDS)
    status = "HOLD"
    proof_sha = "NONE"
    proof_bytes = 0
    cert_ns = 0
    cert_checked = "NONE"
    witness_sha = "NONE"
    witness_vars = 0
    if status_code == 10:
        status = "SAT"
        before = monotonic_ns()
        model = original_model(arm, archive, n)
        assert witness_check(clauses, model)
        cert_ns = monotonic_ns() - before
        witness_sha = hashlib.sha256(
            repr(tuple(sorted(model.items()))).encode("ascii")).hexdigest()
        witness_vars = len(model)
        cert_checked = "ORIGINAL_CNF_WITNESS"
    elif status_code == 20:
        status = "UNSAT_UNCERTIFIED"
        if arm == "cadical" and proof.is_file() and proof.stat().st_size:
            proof_bytes = proof.stat().st_size
            proof_sha = hashlib.sha256(proof.read_bytes()).hexdigest()
            before = monotonic_ns()
            with (archive / "drat.stdout").open("wb") as proof_out, (
                    archive / "drat.stderr").open("wb") as proof_err:
                verified = run(
                    ["timeout", "--kill-after=5s", str(CERT_LIMIT_SECONDS) + "s",
                     str(drat_path), str(cnf), str(proof)],
                    stdout=proof_out, stderr=proof_err, check=False)
            cert_ns = monotonic_ns() - before
            if verified.returncode == 0 and "s VERIFIED" in (
                    archive / "drat.stdout").read_text(encoding="ascii"):
                cert_checked = "DRAT_TRIM_VERIFIED"
                status = "UNSAT_CERTIFIED"
            else:
                status = "HOLD_UNSAT_PROOF"
    elif status_code in (124, 137, 143, -9):
        status = "HOLD_TIME"
    else:
        # A crash is a failure, not a budget HOLD.
        raise AssertionError("unexpected solver status " + arm + " " +
                             case + " " + str(status_code) + " " +
                             stderr.read_text(encoding="ascii")[-1000:])
    result = dict(case=case, arm=arm, raw_sha256=hashlib.sha256(original).hexdigest(),
                  variables=n, clauses=len(clauses), solver_exit=status_code,
                  result=status, proof_checker=cert_checked,
                  witness_sha256=witness_sha, witness_variables=witness_vars,
                  proof_sha256=proof_sha, proof_bytes=proof_bytes,
                  total_solver_ns=elapsed, certificate_ns=cert_ns,
                  peak_rss_kib=peak, limit_seconds=LIMIT_SECONDS)
    (archive / "receipt.txt").write_text(repr(result) + "\n", encoding="ascii")
    print("HARD_EXTERNAL|" + repr(result) + "|json=0", flush=True)


if __name__ == "__main__":
    assert len(sys.argv) == 6
    arm, case, cnf_path, archive_path, drat = sys.argv[1:]
    assert arm in ("minisat", "cadical", "picosat")
    assert case in ("php21", "php8", "tseitin128", "php4sat")
    run_external(arm, case, Path(cnf_path), Path(archive_path), Path(drat))
