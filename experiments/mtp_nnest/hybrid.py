#!/usr/bin/env python3
"""CNF-native adaptation: MTP frozen +/- order-2 + N-Nest corrective watchers.

Original historical sources:
  ASOLARIA players/rime_agents.py: frozen order-2 forward/backward MTP
  N-Nest-Prime-INFINITE-SELF-REFLECT-AGENTS-NESTED:
    node_green = (reported == watcher.recomputed) & all child_green
  ASOLARIA 27-point RIME prism and smooth Fischer wind/unwind sphere.

This is an integer-only, finite, source-blind SAT branch-order prototype,
NOT a claim to execute unlimited agents or use text-trained MTP weights.
"""
import hashlib
import sys
from collections import Counter, defaultdict
from pathlib import Path
from time import monotonic_ns

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT.joinpath("experiments","rime_sat")))
import transform

LEVELS = (3, 9, 27, 81)
MAX_SAMPLED_CLAUSES = 4096
MAX_CANDIDATES = 81
ALPHABET = 64
ARMS = ("baseline", "mtp", "fischer",
        "nest3", "nest9", "nest27", "nest81",
        "mtp_nest3", "mtp_nest9", "mtp_nest27", "mtp_nest81",
        "all3", "all9", "all27", "all81")


def sample(clauses):
    """Fixed evenly spaced source-stratified sample, no answer leakage."""
    step = max(1, len(clauses) // MAX_SAMPLED_CLAUSES)
    return tuple(clauses[i] for i in range(0, len(clauses), step)
                 )[:MAX_SAMPLED_CLAUSES]


def token(lit):
    return ((abs(lit) - 1) % 32) + (32 if lit < 0 else 0)


def training_sequences(rows):
    return tuple(tuple(token(x) for x in row[:12]) for row in rows)


def freeze_order2(sequences):
    """Frozen direction + and -; Laplace numerator/denominator integers."""
    three = Counter()
    two = Counter()
    for sentence in sequences:
        for i in range(2, len(sentence)):
            key = (sentence[i - 2], sentence[i - 1])
            two[key] += 1
            three[(key[0], key[1], sentence[i])] += 1
    return three, two


def mtp_metrics(candidates, clauses):
    """Frozen order-two models; even rows train, odd rows score variables."""
    began = monotonic_ns()
    rows = sample(clauses)
    sequences = training_sequences(rows)
    forward = freeze_order2(sequences[::2])
    backward = freeze_order2(tuple(reversed(x) for x in sequences[::2]))
    forward_weight = Counter()
    backward_weight = Counter()
    eligible = set(candidates)
    for clause, words in zip(rows[1::2], sequences[1::2]):
        reverse = tuple(reversed(words))
        for i in range(2, len(words)):
            lit = clause[i]
            key = (words[i-2],words[i-1])
            frequency = forward[0][(key[0],key[1],words[i])]
            denom = forward[1][key] + ALPHABET
            if abs(lit) in eligible:
                forward_weight[abs(lit)] += ((frequency + 1) * 1024) // denom
            ri = len(words)-1-i
            if ri >= 2:
                rkey = (reverse[ri-2],reverse[ri-1])
                rfrequency = backward[0][(rkey[0],rkey[1],reverse[ri])]
                rdenom = backward[1][rkey] + ALPHABET
                if abs(lit) in eligible:
                    backward_weight[abs(lit)] += (
                        (rfrequency+1)*1024)//rdenom
    return (forward_weight, backward_weight,
            len(forward[0]),len(backward[0]),monotonic_ns()-began)


def candidates_from_source(n, clauses):
    rows = sample(clauses)
    activity = Counter(abs(lit) for row in rows for lit in row)
    if n <= MAX_CANDIDATES:
        return tuple(range(1,n+1)),rows
    selected = sorted(activity,key=lambda v:(-activity[v],v))
    return tuple(selected[:MAX_CANDIDATES]),rows


def fischer_order(candidates, n, sha):
    """Historical RIME point and exact inverse, used as SOURCE-only ordering."""
    started=monotonic_ns()
    ordering={}
    for old in candidates:
        group=(old-1)//27
        local=(old-1)%27
        source=hashlib.sha256(bytes.fromhex(sha)+group.to_bytes(
            8,"little")).digest()[:27]
        spectrum=transform.ntt(tuple(source))
        assert transform.intt(spectrum)==tuple(source)
        addr=(spectrum[local]+256*(local%3)+768*(group%27))%transform.SPHERE_N
        point=transform.SPHERE[addr]
        assert transform.UNWIND[point]==addr
        ordering[old]=(point,spectrum[local])
    ranked=tuple(sorted(candidates,key=lambda v:(ordering[v][0],
                                                  ordering[v][1],v)))
    return ranked,monotonic_ns()-started


def leaf_work(rows,candidates,leaves):
    """Claimed agent work: deterministic shards over sampled clauses."""
    ids={v:i for i,v in enumerate(candidates)}
    vectors=[[0]*len(candidates) for _ in range(leaves)]
    for j,clause in enumerate(rows):
        target=vectors[j%leaves]
        for lit in clause:
            pos=ids.get(abs(lit))
            if pos is not None:
                target[pos] += 2 if lit>0 else 1
    return tuple(tuple(v) for v in vectors)


def independent_leaf_watch(rows,candidates,leaves):
    """Independent recomputation, independently scanning each shard."""
    out=[]
    for seat in range(leaves):
        terms=Counter()
        for idx in range(seat,len(rows),leaves):
            for lit in rows[idx]:
                variable=abs(lit)
                if variable in candidates:
                    terms[variable] += (2 if lit>0 else 1)
        out.append(tuple(terms[v] for v in candidates))
    return tuple(out)


def add_vectors(vectors):
    return tuple(map(sum,zip(*vectors)))


def seal(addr,reported,children):
    raw=(addr+"|"+repr(reported)+"|"+
         "|".join(child for child in children)).encode("ascii")
    return hashlib.sha256(raw).hexdigest()


def tree_gate(claims,expected,leaves,inject=None):
    """3-child tree; every node separately recomputed, fail propagates."""
    assert leaves in LEVELS
    assert len(claims)==len(expected)==leaves
    depth={3:1,9:2,27:3,81:4}[leaves]
    executed=0
    watchers=0
    first_failure=None
    leaf_votes=Counter()

    def node(addr,level,start,stop):
        nonlocal executed,watchers,first_failure
        executed+=1
        watchers+=1
        if level==depth:
            assert stop-start==1
            reported=claims[start]
            calculated=expected[start]
            if inject==addr:
                reported=tuple(x+1 if i==0 else x for i,x in enumerate(reported))
            # Each leaf independently gives its preferred candidate.
            vote=max(range(len(reported)),key=lambda i:(reported[i],-i))
            leaf_votes[vote]+=1
            labels=()
            okay=reported==calculated
            return reported,calculated,seal(addr,reported,labels),okay
        width=(stop-start)//3
        children=[node(addr+"."+str(i),level+1,
                       start+i*width,start+(i+1)*width)
                  for i in range(3)]
        raw=add_vectors(tuple(child[0] for child in children))
        check=add_vectors(tuple(child[1] for child in children))
        if inject==addr:
            raw=tuple(x+1 if i==0 else x for i,x in enumerate(raw))
        ids=tuple(child[2] for child in children)
        fine=(raw==check) and all(child[3] for child in children)
        return raw,check,seal(addr,raw,ids),fine

    root=node("R",0,0,leaves)
    assert executed==(pow(3,depth+1)-1)//2
    assert watchers==executed
    assert root[0]==root[1] if inject is None else not root[3]
    assert root[3] == (inject is None)
    return dict(depth=depth,leaves=leaves,nodes=executed,
                watcher_checks=watchers,accepted=root[3],
                root_sha256=root[2],leaf_votes=dict(leaf_votes))


def order_for_source(n,clauses,sha,arm):
    """Return branch hints and auditable, source-only preparation receipt."""
    assert arm in ARMS
    began=monotonic_ns()
    candidates,rows=candidates_from_source(n,clauses)
    base=Counter(abs(v) for row in rows for v in row)
    active=len(candidates)
    mtp=arm=="mtp" or arm.startswith("mtp_nest") or arm.startswith("all")
    fischer=arm=="fischer" or arm.startswith("all")
    nest=arm.startswith("nest") or arm.startswith("mtp_nest") or arm.startswith("all")
    levels=next((int(arm[len(p):]) for p in ("nest","mtp_nest","all")
                 if arm.startswith(p)),0)
    if nest:
        assert levels in LEVELS
    forward=Counter()
    backward=Counter()
    freeze_ns=0
    fwd_patterns=bwd_patterns=0
    if mtp:
        forward,backward,fwd_patterns,bwd_patterns,freeze_ns=mtp_metrics(
            candidates,clauses)
    fischer_ns=0
    prism_ranks={}
    if fischer:
        ranked,fischer_ns=fischer_order(candidates,n,sha)
        prism_ranks={v:len(candidates)-i for i,v in enumerate(ranked)}
    nest_receipt=None
    votes={}
    nest_ns=0
    if nest:
        before=monotonic_ns()
        claims=leaf_work(rows,candidates,levels)
        watcher=independent_leaf_watch(rows,candidates,levels)
        assert claims==watcher
        nest_receipt=tree_gate(claims,watcher,levels)
        votes={candidates[k]:v for k,v in nest_receipt["leaf_votes"].items()}
        nest_ns=monotonic_ns()-before
    # Integer, deterministic mixed score. No output labels involved.
    def key(v):
        return (-(base[v]*1000 +
                  (forward[v]+backward[v])*40 +
                  votes.get(v,0)*250 +
                  prism_ranks.get(v,0)*10),v)
    if arm=="baseline":
        ordered=tuple(candidates)
    else:
        ordered=tuple(sorted(candidates,key=key))
    assert set(ordered)==set(candidates)
    result=dict(arm=arm,candidates=active,
                sampled_clauses=len(rows),mtp_fwd_patterns=fwd_patterns,
                mtp_bwd_patterns=bwd_patterns,
                mtp_freeze_ns=freeze_ns,fischer_ns=fischer_ns,
                nest_ns=nest_ns,coordination_nodes=(
                    nest_receipt["nodes"] if nest_receipt else 0),
                watcher_checks=(
                    nest_receipt["watcher_checks"] if nest_receipt else 0),
                root_sha256=(
                    nest_receipt["root_sha256"] if nest_receipt else "NONE"),
                prep_ns=monotonic_ns()-began,
                hint_sha256=hashlib.sha256(
                    repr(ordered).encode("ascii")).hexdigest(),
                levels=levels)
    return ordered,result


def test_tamper():
    rows=((1,-2,3),(-1,2,-3),(1,2,3),(-1,-2,3))*27
    candidates=(1,2,3)
    results=[]
    for leaves in LEVELS:
        claims=leaf_work(rows,candidates,leaves)
        watcher=independent_leaf_watch(rows,candidates,leaves)
        assert claims==watcher
        clean=tree_gate(claims,watcher,leaves)
        assert clean["accepted"]
        for step in range(1,clean["depth"]+1):
            addr="R"+".0"*step
            bad=tree_gate(claims,watcher,leaves,inject=addr)
            assert not bad["accepted"]
            results.append((leaves,step,addr))
    assert len(results)==10
    print("NNEST_TAMPER|levels=3,9,27,81|injected_levels=10|"
          "all_blocked=PASS|operator_policy=BOUNDED|status=PASS")


if __name__=="__main__":
    test_tamper()
