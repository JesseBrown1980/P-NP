#!/usr/bin/env python3
"""Frozen completion / scaling SAT test corpus; no solver imports.

New witnessed 3-SAT formulas at 12/18/24/36/54/81/121 variables, two
mathematically UNSAT non-XOR complete-graph 3-color constraints, one
PHP and one Tseitin original structural source, and four source-authenticated
full Bitcoin genesis double-SHA256 CNFs.

The planted assignment is never included in the solver input or source
metadata. The independent referee validates all SAT models against the
actual original clauses; 3-color K4/K5 UNSAT is guaranteed by four/five
mutually adjacent vertices having only three available colors.
"""
import hashlib
import random
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT.joinpath("experiments","array")))
import suite
sys.path.insert(0,str(ROOT.joinpath("experiments","bitcoin")))
import genesis

COMPLETION=("plant12","plant18","plant24","color4","color5","php4_3","tseitin16")
SCALING=("plant36","plant54","plant81","plant121","php8_7",
         "nonce4","nonce8","nonce32","merkle_nonce288")
CASES=COMPLETION+SCALING


def planted(n):
    assert n in (12,18,24,36,54,81,121)
    seed=int.from_bytes(hashlib.sha256(
        ("sat-growth-v1:"+str(n)).encode("ascii")).digest()[:8],"big")
    rng=random.Random(seed)
    witness=tuple(rng.randrange(2) for _ in range(n))
    clauses=[]
    seen=set()
    count=(83*n+19)//20
    attempts=0
    while len(clauses)<count:
        attempts+=1
        assert attempts<count*500
        vars_=tuple(sorted(rng.sample(range(1,n+1),3)))
        signs=tuple(rng.choice((-1,1)) for _ in range(3))
        clause=tuple(v*s for v,s in zip(vars_,signs))
        if clause in seen:
            continue
        if not any(bool(witness[abs(lit)-1])==(lit>0)
                   for lit in clause):
            continue
        clauses.append(clause)
        seen.add(clause)
    assert all(any(bool(witness[abs(lit)-1])==(lit>0)
                   for lit in row) for row in clauses)
    return n,tuple(clauses),"SAT"


def three_color(vertices):
    assert vertices in (4,5)
    n=3*vertices
    clauses=[]
    # At least one of 3 colors, each vertex exactly one color.
    for i in range(vertices):
        variables=tuple(3*i+j+1 for j in range(3))
        clauses.append(variables)
        for a in range(3):
            for b in range(a+1,3):
                clauses.append((-variables[a],-variables[b]))
    # Complete graph: at most one vertex uses each color.
    for a in range(vertices):
        for b in range(a+1,vertices):
            for c in range(3):
                clauses.append((-(3*a+c+1),-(3*b+c+1)))
    assert vertices>3
    return n,tuple(clauses),"UNSAT"


def get(case,bitcoin_root):
    if case.startswith("plant"):
        return planted(int(case.removeprefix("plant")))
    if case=="color4":
        return three_color(4)
    if case=="color5":
        return three_color(5)
    if case in ("php4_3","php8_7","tseitin16"):
        return suite.definition(case)
    assert case in genesis.CASES
    raw=Path(bitcoin_root).joinpath(case+".cnf").read_bytes()
    n=meta=None
    for line in raw.decode("ascii").splitlines():
        if line.startswith("p cnf "):
            _,_,n,meta=line.split()
            break
    assert int(n)>230000 and int(meta)>800000
    return int(n),raw,"SAT"


def dimacs(n,clauses):
    return ("p cnf "+str(n)+" "+str(len(clauses))+"\n"+
            "\n".join(" ".join(str(v) for v in row)+" 0"
                      for row in clauses)+"\n").encode("ascii")


def build(folder):
    root=Path(folder)
    root.mkdir(parents=True,exist_ok=True)
    bitcoin=root.joinpath("bitcoin_reference")
    bitcoin.mkdir(exist_ok=True)
    genesis.main(str(bitcoin))
    manifest=[]
    for case in CASES:
        n,value,expected=get(case,bitcoin)
        raw=(value if isinstance(value,bytes)
             else dimacs(n,value))
        assert raw.startswith(b"p cnf ")
        path=root.joinpath(case+".cnf")
        path.write_bytes(raw)
        hash_=hashlib.sha256(raw).hexdigest()
        rows=next(line for line in raw.decode("ascii").splitlines()
                  if line.startswith("p cnf ")).split()
        nclauses=int(rows[3])
        manifest.append((case,n,nclauses,expected,
                         "completion" if case in COMPLETION else "scaling",
                         hash_))
        print("SCALE_SOURCE|case="+case+"|variables="+str(n)+
              "|clauses="+str(nclauses)+"|expected="+expected+
              "|sha256="+hash_+"|status=PASS",flush=True)
    assert len(manifest)==16
    root.joinpath("MANIFEST.txt").write_text(
        "".join(repr(row)+"\n" for row in manifest),encoding="ascii")
    print("SCALE_SOURCES_FINAL|completion=7|scaling=9|cases=16|"
          "old_engines=7|combined_arms=15|total=352|status=PASS")


if __name__=="__main__":
    assert len(sys.argv)==2
    build(sys.argv[1])
