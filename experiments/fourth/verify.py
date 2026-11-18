#!/usr/bin/env python3
"""Independent source-only verifier for fourth engine's emitted certificates.

Does NOT import the fourth engine, any of its inference routines, any old
contender, the fixture generator, or an assumed expected model answer.
Checks the raw DIMACS and the explicitly archived certificate.
"""
import ast
import hashlib
import sys
from collections import Counter
from pathlib import Path


def parsed(raw):
    n, nclauses, clauses, pending = None, None, [], []
    for line in raw.decode("ascii").splitlines():
        terms = line.split()
        if not terms or terms[0] == "c":
            continue
        if terms[0] == "p":
            assert n is None and len(terms) == 4 and terms[1] == "cnf"
            n, nclauses = int(terms[2]), int(terms[3])
            continue
        assert n is not None
        for term in terms:
            literal = int(term)
            if literal:
                assert 1 <= abs(literal) <= n
                pending.append(literal)
            else:
                assert pending and len({abs(x) for x in pending}) == len(pending)
                clauses.append(tuple(pending))
                pending = []
    assert n and nclauses == len(clauses) and not pending
    return n, tuple(clauses)


def checked_witness(clauses, integer):
    return all(any(bool(integer & (1 << (abs(v) - 1))) == (v > 0)
                   for v in clause) for clause in clauses)


def hall_proof(n, clauses, record):
    """Check partition, every input clause, and Hall deficit exactly."""
    groups = record["hall_groups"]
    slots = record["hall_slots"]
    assert groups and slots
    all_variables = [v for group in groups for v in group]
    all_slot_variables = [v for slot in slots for v in slot]
    assert sorted(all_variables) == list(range(1, n + 1))
    assert sorted(all_slot_variables) == list(range(1, n + 1))
    assert all(len(group) >= 2 and len(group) == len(set(group))
               for group in groups)
    assert all(len(slot) >= 1 and len(slot) == len(set(slot))
               for slot in slots)
    expected = Counter(tuple(sorted(group)) for group in groups)
    for slot in slots:
        for i, a in enumerate(slot):
            for b in slot[i + 1:]:
                expected[(-max(a, b), -min(a, b))] += 1
    actual = Counter(tuple(sorted(clause)) for clause in clauses)
    assert expected == actual
    slot_map = {var: j for j, slot in enumerate(slots) for var in slot}
    neighbors = [{slot_map[v] for v in group} for group in groups]
    matching = dict(record["hall_matching"])
    assert len(matching) == record["matching_size"]
    assert len(matching) == len(set(matching.values()))
    assert all(s in neighbors[g] for g, s in matching.items())
    if record["status"] == "UNSAT_HALL":
        subset = set(record["hall_subset"])
        assert subset and all(0 <= g < len(groups) for g in subset)
        neighborhood = set(record["hall_neighborhood"])
        assert neighborhood == {s for g in subset for s in neighbors[g]}
        assert len(subset) > len(neighborhood)
        assert record["witness_integer"] == "NONE"
    else:
        assert record["status"] == "SAT"
        assert len(matching) == len(groups)
        assert not record["hall_subset"]
        assert checked_witness(clauses, record["witness_integer"])


def xor_proof(n, clauses, record):
    grouped = {}
    for clause in clauses:
        assert len(clause) == 3
        key = tuple(sorted(abs(v) for v in clause))
        assert len(set(key)) == 3
        grouped.setdefault(key, []).append(clause)
    reconstructed = []
    for scope, parts in sorted(grouped.items()):
        assert len(parts) == 4
        allowed = []
        for mask in range(8):
            values = {v: bool((mask >> j) & 1)
                      for j, v in enumerate(scope)}
            if all(any(values[abs(lit)] == (lit > 0)
                       for lit in clause) for clause in parts):
                allowed.append(mask)
        assert len(allowed) == 4
        parities = {mask.bit_count() % 2 for mask in allowed}
        assert len(parities) == 1
        reconstructed.append((
            sum(1 << (v - 1) for v in scope),
            next(iter(parities))))
    assert tuple(reconstructed) == record["xor_rows"]
    if record["status"] == "UNSAT_XOR":
        indices = record["xor_subset"]
        assert indices and len(set(indices)) == len(indices)
        lhs = rhs = 0
        for i in indices:
            mask, parity = reconstructed[i]
            lhs ^= mask
            rhs ^= parity
        assert lhs == 0 and rhs == 1
        assert record["witness_integer"] == "NONE"
    else:
        assert record["status"] == "SAT"
        assert checked_witness(clauses, record["witness_integer"])


def verify(raw, record):
    n, clauses = parsed(raw)
    assert record["method"] == "FOURTH_STRUCTURAL_ENGINE"
    assert record["variables"] == n and record["clauses"] == len(clauses)
    sha = hashlib.sha256(repr(clauses).encode("ascii")).hexdigest()
    assert record["source_sha256"] == sha
    assert record["node_cap"] == 2000
    assert all(isinstance(record[key], int) and record[key] >= 0
               for key in ("search_nodes", "literal_checks",
                           "preprocess_ns", "total_ns"))
    family = record["family"]
    if family == "HALL":
        hall_proof(n, clauses, record)
        assert record["search_nodes"] == 0
    elif family == "XOR":
        xor_proof(n, clauses, record)
        assert record["search_nodes"] == 0
    else:
        assert family == "GENERIC"
        assert n <= 6
        true_count = sum(checked_witness(clauses, assignment)
                         for assignment in range(1 << n))
        actual = record["status"]
        assert actual in ("SAT", "UNSAT_DPLL", "HOLD_BUDGET")
        if actual == "SAT":
            assert true_count and checked_witness(
                clauses, record["witness_integer"])
        elif actual == "UNSAT_DPLL":
            assert true_count == 0
        else:
            assert record["search_nodes"] == record["node_cap"]
    return record["status"], family


def main(directory):
    folder = Path(directory)
    receipts = folder / "receipts"
    fixtures = folder / "fixtures"
    records = []
    for line in (fixtures / "FIXTURES.txt").read_text(
            encoding="ascii").splitlines():
        name, n, clauses_count, expectation, input_sha = ast.literal_eval(line)
        raw = (fixtures / (name + ".cnf")).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == input_sha
        assert parsed(raw)[0] == n and len(parsed(raw)[1]) == clauses_count
        rec = ast.literal_eval((receipts / (name + ".txt")).read_text(
            encoding="ascii"))
        status, family = verify(raw, rec)
        if expectation in ("SAT", "UNSAT"):
            assert ((status == "SAT") if expectation == "SAT"
                    else status in ("UNSAT_HALL", "UNSAT_XOR", "UNSAT_DPLL"))
        else:
            assert family == "GENERIC"
        records.append((name, status, family, rec["search_nodes"],
                        rec["total_ns"]))
        print("FOURTH_REFEREE|fixture=" + name + "|status=" + status +
              "|route=" + family + "|nodes=" +
              str(rec["search_nodes"]) + "|independent=PASS",
              flush=True)
    assert len(records) == 61
    original = [x for x in records if x[0].endswith("__original")]
    metamorphic = [x for x in records if "__permutation" in x[0]]
    controls = [x for x in records if x[0].startswith("negative__")]
    assert len(original) == 14 and len(metamorphic) == 42
    assert len(controls) == 5
    assert all(x[3] == 0 for x in original + metamorphic)
    assert all(x[2] in ("HALL", "XOR") for x in original + metamorphic)
    assert all(x[2] == "GENERIC" for x in controls)
    (folder / "FOURTH_REFEREE_ROWS.txt").write_text(
        "".join(repr(x) + "\n" for x in records), encoding="ascii")
    print("FOURTH_REFEREE_FINAL|original=14|permuted=42|"
          "adversarial=5|total=61|original_exact=14|"
          "metamorphic_exact=42|independent_proofs=56|"
          "fallback_controls=5|status=PASS")


if __name__ == "__main__":
    assert len(sys.argv) == 2
    main(sys.argv[1])
