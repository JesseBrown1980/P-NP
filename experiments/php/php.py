#!/usr/bin/env python3
"""Exact pigeonhole CNF generation + bounded native general-CNF DPLL + baselines.

Pigeon i / hole h is variable i*H+h+1 (zero-based i,h).
The basic encoding requires every pigeon to occupy >=1 hole and no
hole to contain two pigeons; it allows a pigeon to occupy several holes.
The optional functional encoding adds at-most-one-hole per pigeon.
SAT/UNSAT external claims are not certified here: see independent verify.py.
"""
import hashlib
import sys
from itertools import combinations
from pathlib import Path
from subprocess import run
from time import monotonic_ns

CASES = ((2, 1), (2, 2), (3, 2), (3, 3), (4, 3))
LARGE = (21, 20)
SOURCE_VERSION = "php-basic-v1"


def vid(i, hole, holes):
    return i * holes + hole + 1


def generate(pigeons, holes, functional=False):
    assert pigeons >= 2 and holes >= 1
    blocks = {"pigeon_at_least_one": [], "hole_pairwise_at_most_one": [],
              "pigeon_pairwise_at_most_one": []}
    for i in range(pigeons):
        blocks["pigeon_at_least_one"].append(
            tuple(vid(i, hole, holes) for hole in range(holes)))
    for hole in range(holes):
        for i, j in combinations(range(pigeons), 2):
            blocks["hole_pairwise_at_most_one"].append(
                (-vid(i, hole, holes), -vid(j, hole, holes)))
    if functional:
        for i in range(pigeons):
            for a, b in combinations(range(holes), 2):
                blocks["pigeon_pairwise_at_most_one"].append(
                    (-vid(i, a, holes), -vid(i, b, holes)))
    return tuple(blocks["pigeon_at_least_one"] +
                 blocks["hole_pairwise_at_most_one"] +
                 blocks["pigeon_pairwise_at_most_one"])


def counts(pigeons, holes, functional):
    alo = pigeons
    hole_amo = holes * pigeons * (pigeons - 1) // 2
    pigeon_amo = pigeons * holes * (holes - 1) // 2 if functional else 0
    return alo, hole_amo, pigeon_amo, alo + hole_amo + pigeon_amo


def encoding(pigeons, holes, clauses, functional=False):
    flags = "functional" if functional else "basic"
    lines = [
        "c P-NP original PHP source, generator " + SOURCE_VERSION,
        "c pigeons " + str(pigeons) + " holes " + str(holes) +
        " encoding " + flags,
        "p cnf " + str(pigeons * holes) + " " + str(len(clauses)),
    ]
    lines.extend(" ".join(str(x) for x in clause) + " 0"
                 for clause in clauses)
    return "\n".join(lines) + "\n"


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def parse(raw):
    n, announced = None, None
    clauses, pending = [], []
    for line in raw.splitlines():
        fields = line.split()
        if not fields or fields[0] == "c":
            continue
        if fields[0] == "p":
            assert len(fields) == 4 and fields[1] == "cnf"
            assert n is None
            n, announced = int(fields[2]), int(fields[3])
            assert n > 0 and announced >= 0
            continue
        assert n is not None
        for word in fields:
            lit = int(word)
            if lit:
                assert 1 <= abs(lit) <= n
                pending.append(lit)
            else:
                assert pending and len(set(map(abs, pending))) == len(pending)
                clauses.append(tuple(pending))
                pending = []
    assert n and not pending and len(clauses) == announced
    return n, tuple(clauses)


def holds(clauses, bits):
    return all(any(bool(bits & (1 << (abs(lit) - 1))) == (lit > 0)
                   for lit in clause) for clause in clauses)


def native_dpll(n, clauses, limit=300000):
    """Exhaustive exact bounded general-CNF SAT decision, no borrowed witness."""
    start = monotonic_ns()
    degree = [0] * n
    for clause in clauses:
        for lit in clause:
            degree[abs(lit) - 1] += 1
    order = tuple(sorted(range(n), key=lambda i: (-degree[i], i)))
    prep_ns = monotonic_ns() - start
    started = monotonic_ns()
    nodes = [0]

    def visit(assigned):
        nodes[0] += 1
        if nodes[0] > limit:
            return None, "HOLD"
        state = dict(assigned)
        while True:
            changed, all_satisfied = False, True
            for clause in clauses:
                satisfied, unassigned = False, []
                for lit in clause:
                    var = abs(lit) - 1
                    if var not in state:
                        unassigned.append(lit)
                    elif state[var] == (lit > 0):
                        satisfied = True
                        break
                if satisfied:
                    continue
                if not unassigned:
                    return None, "UNSAT"
                all_satisfied = False
                if len(unassigned) == 1:
                    forced = unassigned[0]
                    v = abs(forced) - 1
                    val = forced > 0
                    if v in state:
                        if state[v] != val:
                            return None, "UNSAT"
                    else:
                        state[v] = val
                        changed = True
            if all_satisfied:
                witness = sum((1 << k) for k, val in state.items() if val)
                return witness, "SAT"
            if not changed:
                break
        v = next(v for v in order if v not in state)
        for value in (False, True):
            next_state = dict(state)
            next_state[v] = value
            witness, verdict = visit(next_state)
            if verdict == "SAT" or verdict == "HOLD":
                return witness, verdict
        return None, "UNSAT"

    witness, status = visit({})
    search_ns = monotonic_ns() - started
    start_cert = monotonic_ns()
    if status == "SAT":
        assert witness is not None and holds(clauses, witness)
    else:
        assert witness is None
    cert_ns = monotonic_ns() - start_cert
    return dict(verdict=status, witness=witness, search_nodes=nodes[0],
                preprocess_ns=prep_ns, search_ns=search_ns,
                witness_check_ns=cert_ns)


def write(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="ascii")


def build(root):
    root.mkdir(parents=True, exist_ok=True)
    for p, h in CASES + (LARGE,):
        start = monotonic_ns()
        clauses = generate(p, h)
        raw = encoding(p, h, clauses)
        generation_ns = monotonic_ns() - start
        path = root / ("php-" + str(p) + "-" + str(h) + ".cnf")
        write(path, raw)
        assert parse(raw) == (p * h, clauses)
        a, b, c, total = counts(p, h, False)
        assert total == len(clauses)
        print("PHP_CNF|instance=" + path.stem + "|variables=" +
              str(p * h) + "|pigeon_ALO=" + str(a) +
              "|hole_AMO=" + str(b) + "|pigeon_AMO=" + str(c) +
              "|clauses=" + str(total) + "|source_sha256=" +
              sha_bytes(raw.encode("ascii")) + "|generation_ns=" +
              str(generation_ns) + "|status=PASS", flush=True)
    p, h = LARGE
    start = monotonic_ns()
    clauses = generate(p, h, True)
    raw = encoding(p, h, clauses, True)
    generation_ns = monotonic_ns() - start
    path = root / "php-21-20-functional.cnf"
    write(path, raw)
    a, b, c, total = counts(p, h, True)
    assert len(clauses) == total and parse(raw) == (p * h, clauses)
    print("PHP_FUNCTIONAL|variables=420|pigeon_ALO=" + str(a) +
          "|hole_AMO=" + str(b) + "|pigeon_AMO=" + str(c) +
          "|clauses=" + str(total) + "|source_sha256=" +
          sha_bytes(raw.encode("ascii")) + "|generation_ns=" +
          str(generation_ns) + "|status=PASS", flush=True)


def native(root):
    for p, h in CASES:
        name = "php-" + str(p) + "-" + str(h)
        raw = (root / (name + ".cnf")).read_text(encoding="ascii")
        t0 = monotonic_ns()
        n, clauses = parse(raw)
        parse_ns = monotonic_ns() - t0
        result = native_dpll(n, clauses)
        assert result["verdict"] == ("UNSAT" if p > h else "SAT")
        record = dict(instance=name, input_sha256=sha_bytes(raw.encode("ascii")),
                      parse_ns=parse_ns, **result)
        write(root / "native" / (name + ".txt"), repr(record) + "\n")
        print("PHP_NATIVE|instance=" + name + "|verdict=" + result["verdict"] +
              "|nodes=" + str(result["search_nodes"]) +
              "|preprocessing_ns=" + str(parse_ns + result["preprocess_ns"]) +
              "|search_ns=" + str(result["search_ns"]) +
              "|certificate_ns=" + str(result["witness_check_ns"]) +
              "|status=PASS", flush=True)


def execute(program, args, output_path, memory_path):
    # Child exit code 10/20 is standard SAT/UNSAT, so do not use check=True.
    cmd = ["/usr/bin/time", "-f", "%M", "-o", str(memory_path),
           program] + args
    began = monotonic_ns()
    result = run(cmd, capture_output=True, text=True, check=False)
    elapsed = monotonic_ns() - began
    write(output_path, result.stdout + result.stderr)
    assert result.returncode in (10, 20), (program, result.returncode,
                                           result.stderr[-1000:])
    memory = memory_path.read_text(encoding="ascii").splitlines()[-1]
    assert memory.isdigit()
    return result.returncode, elapsed, int(memory)


def crosscheck(root, drat_trim):
    assert drat_trim.exists()
    rows = []
    for p, h in CASES:
        name = "php-" + str(p) + "-" + str(h)
        source = root / (name + ".cnf")
        assert source.exists()
        native_record = root / "native" / (name + ".txt")
        assert native_record.exists()
        logs = root / "solver" / name
        logs.mkdir(parents=True, exist_ok=True)
        minisat_solution = logs / "minisat.sol"
        mini_rc, mini_ns, mini_kib = execute(
            "minisat", ["-verb=0", str(source), str(minisat_solution)],
            logs / "minisat.log", logs / "minisat.rss")
        cad_rc, cad_ns, cad_kib = execute(
            "cadical", [str(source)], logs / "cadical.log",
            logs / "cadical.rss")
        assert mini_rc == cad_rc
        expect = 10 if p <= h else 20
        assert mini_rc == expect
        proof_gen_ns, proof_bytes, proof_verify_ns = 0, 0, 0
        if cad_rc == 20:
            proof = logs / "cadical.drat"
            r, proof_gen_ns, _ = execute(
                "cadical", ["--no-binary", str(source), str(proof)],
                logs / "cadical-proof.log", logs / "cadical-proof.rss")
            assert r == 20 and proof.is_file() and proof.stat().st_size > 0
            proof_bytes = proof.stat().st_size
            began = monotonic_ns()
            checked = run([str(drat_trim), str(source), str(proof)],
                          capture_output=True, text=True, check=False)
            proof_verify_ns = monotonic_ns() - began
            write(logs / "drat-trim.log", checked.stdout + checked.stderr)
            assert checked.returncode == 0
            assert "s VERIFIED" in checked.stdout
        row = dict(instance=name, pigeons=p, holes=h,
                   source_sha256=sha_bytes(source.read_bytes()),
                   native_receipt_sha256=sha_bytes(native_record.read_bytes()),
                   minisat_verdict=mini_rc, cadical_verdict=cad_rc,
                   minisat_search_ns=mini_ns, cadical_search_ns=cad_ns,
                   minisat_peak_kib=mini_kib, cadical_peak_kib=cad_kib,
                   proof_generation_ns=proof_gen_ns,
                   proof_validation_ns=proof_verify_ns,
                   drat_bytes=proof_bytes)
        rows.append(row)
        write(logs / "receipt.txt", repr(row) + "\n")
        print("PHP_EXTERNAL|instance=" + name + "|minisat_rc=" + str(mini_rc) +
              "|cadical_rc=" + str(cad_rc) +
              "|minisat_search_ns=" + str(mini_ns) +
              "|cadical_search_ns=" + str(cad_ns) +
              "|proof_generation_ns=" + str(proof_gen_ns) +
              "|proof_validation_ns=" + str(proof_verify_ns) +
              "|drat_bytes=" + str(proof_bytes) + "|status=PASS", flush=True)
    assert len(rows) == 5
    print("PHP_EXTERNAL_FINAL|five_cases=PASS|two_solvers_agree=PASS|"
          "drat_produced_for_all_three_unsat=PASS|status=PASS")


def main():
    assert len(sys.argv) >= 3
    mode, root = sys.argv[1], Path(sys.argv[2])
    if mode == "build":
        assert len(sys.argv) == 3
        build(root)
    elif mode == "native":
        assert len(sys.argv) == 3
        native(root)
    elif mode == "crosscheck":
        assert len(sys.argv) == 4
        crosscheck(root, Path(sys.argv[3]))
    else:
        raise AssertionError("unknown mode")


if __name__ == "__main__":
    main()
