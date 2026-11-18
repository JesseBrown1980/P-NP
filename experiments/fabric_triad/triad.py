#!/usr/bin/env python3
"""Third fabric witness for the N-Nest SAT architecture (local source-backed pilot).

Historical three-seat protocol:
  worker         -> TRIADWORK        : independent proposed shard counts
  sister         -> TRIADREFLECT     : separate candidate-major verification
  fabric witness -> TRIADFABRICVERDICT: reads both reports and queries a
                    *local CNF incidence graph*, returning a third verdict
  supervisor     -> TRIADDISTANCE / TRIADCONSENSUS: exact comparison at
                    every 3-ary N-Nest depth, fail-closed to the root.

The archival ASOLARIA neural-fabric API is NOT accessed in this offline
GitHub runner. Its third seat is represented by a reproducible local graph
snapshot and must be explicitly labelled LOCAL_CNF_GRAPH in every receipt.
A live GNN/BigPickle fabric result would be a distinct experiment.

All seats produce proposals/verdicts, not SAT proofs or autonomous actions.
Operator approval remains held. Only source-derived, independently checked
candidate ordering may be passed to a separate SAT solver.
"""
import hashlib
import sys
from collections import Counter
from pathlib import Path

LEVELS=(3,9,27,81)
FABRIC_KIND="LOCAL_CNF_GRAPH"
MAX_CANDIDATES=81
MAX_ROWS=4096


def hash_string(text):
    return hashlib.sha256(text.encode("ascii")).hexdigest()


def row_subset(rows,leaves,start,stop):
    return tuple((i,row) for i,row in enumerate(rows)
                 if start <= (i % leaves) < stop)


def worker(rows,candidates):
    """Row-major independent worker proposal."""
    slot={v:i for i,v in enumerate(candidates)}
    sums=[0]*len(candidates)
    for _,clause in rows:
        for lit in clause:
            j=slot.get(abs(lit))
            if j is not None:
                sums[j]+=2 if lit>0 else 1
    return tuple(sums)


def sister_reflection(rows,candidates):
    """Candidate-major independent watcher; never calls worker."""
    sums=[]
    for v in candidates:
        total=0
        for _,clause in rows:
            for lit in clause:
                if abs(lit)==v:
                    total+=2 if lit>0 else 1
        sums.append(total)
    return tuple(sums)


def graph_snapshot(rows,candidates):
    """Snapshot incidence graph from the source clauses; not a GNN claim."""
    selected=set(candidates)
    postings={v:[] for v in candidates}
    neighbors={v:set() for v in candidates}
    for index,clause in rows:
        local=[abs(lit) for lit in clause if abs(lit) in selected]
        for lit in clause:
            v=abs(lit)
            if v in postings:
                postings[v].append((index,1 if lit>0 else -1))
        for v in local:
            neighbors[v].update(x for x in local if x!=v)
    return postings,neighbors


def fabric_witness(worker_report,reflection_report,graph,candidates):
    """Third seat *reads both reports*, asks local graph, emits its own."""
    postings,neighbors=graph
    graph_projection=tuple(sum(2 if sign>0 else 1
                               for _,sign in postings[v])
                           for v in candidates)
    graph_routes=tuple(len(neighbors[v]) for v in candidates)
    disagreement=tuple(i for i,(a,b,c) in enumerate(
        zip(worker_report,reflection_report,graph_projection))
        if not (a==b==c))
    return (graph_projection,graph_routes,disagreement)


def magnitude(a,b):
    return sum(abs(x-y) for x,y in zip(a,b))


def seal(addr,work,reflection,fabric,children):
    return hash_string(addr+"|"+repr(work)+"|"+repr(reflection)+"|"+
                       repr(fabric)+"|"+repr(tuple(children)))


def nested_triad(rows,candidates,leaves,inject=None):
    """Return complete HBP receipts, root verdict and optional safe ordering.

    inject=(tree_address, one of WORK/REFLECT/FABRIC) changes one report
    after calculation; every planted inconsistency must reach the apex.
    """
    assert leaves in LEVELS and 0<len(candidates)<=MAX_CANDIDATES
    assert len(rows)<=MAX_ROWS
    assert len(set(candidates))==len(candidates)
    depth={3:1,9:2,27:3,81:4}[leaves]
    evidence=[]
    attempts=0
    held=0

    def node(addr,level,start,stop):
        nonlocal attempts,held
        attempts+=1
        children=[]
        if level<depth:
            width=(stop-start)//3
            assert width>=1
            for k in range(3):
                children.append(node(addr+"."+str(k),level+1,
                                     start+k*width,start+(k+1)*width))
        selected=row_subset(rows,leaves,start,stop)
        work=worker(selected,candidates)
        reflect=sister_reflection(selected,candidates)
        graph=graph_snapshot(selected,candidates)
        fabric,route,disagree=fabric_witness(work,reflect,graph,candidates)
        if inject is not None and addr==inject[0]:
            assert inject[1] in ("WORK","REFLECT","FABRIC")
            mutated=(lambda values: (values[0]+1,)+values[1:])
            if inject[1]=="WORK":
                work=mutated(work)
            elif inject[1]=="REFLECT":
                reflect=mutated(reflect)
            else:
                fabric=mutated(fabric)
            # The third agent's actual comparison must reflect mutation.
            disagree=tuple(i for i,(a,b,c) in enumerate(
                zip(work,reflect,fabric)) if not (a==b==c))
        d_wr=magnitude(work,reflect)
        d_wf=magnitude(work,fabric)
        d_rf=magnitude(reflect,fabric)
        local_ok=(d_wr==d_wf==d_rf==0 and not disagree)
        child_ok=all(child[0] for child in children)
        okay=local_ok and child_ok
        if not okay:
            held+=1
        root_sha=seal(addr,work,reflect,fabric,(
            c[1] for c in children))
        work_sha=hash_string(repr(work))
        reflect_sha=hash_string(repr(reflect))
        fabric_sha=hash_string(repr(fabric)+"|"+repr(route))
        evidence.extend((
            "TRIADWORK|triad_id="+addr+"|worker_sha16="+work_sha[:16]+
                "|source=CNF|json=0",
            "TRIADREFLECT|triad_id="+addr+"|work_sha16="+work_sha[:16]+
                "|reflect_sha16="+reflect_sha[:16]+"|json=0",
            "TRIADFABRICVERDICT|triad_id="+addr+"|fabric="+FABRIC_KIND+
                "|reads_worker_sha16="+work_sha[:16]+
                "|reads_reflect_sha16="+reflect_sha[:16]+
                "|verdict_sha16="+fabric_sha[:16]+
                "|agreement="+("PASS" if local_ok else "HOLD")+
                "|json=0",
            "TRIADDISTANCE|triad_id="+addr+
                "|d_worker_reflect="+str(d_wr)+
                "|d_worker_fabric="+str(d_wf)+
                "|d_reflect_fabric="+str(d_rf)+"|json=0",
            "TRIADCONSENSUS|triad_id="+addr+
                "|state="+("HELD_FOR_OPERATOR" if okay else "HELD_CORRECTION")+
                "|children="+str(len(children))+"|auto_fire_allowed=false"+
                "|root_sha16="+root_sha[:16]+"|json=0",
        ))
        return okay,root_sha,work,route

    okay,root_sha,root_projection,root_routes=node("R",0,0,leaves)
    assert attempts==(pow(3,depth+1)-1)//2
    if inject is None:
        assert okay and held==0
    else:
        assert not okay
    order=(tuple(sorted(candidates,
                        key=lambda v:(-(root_projection[
                            candidates.index(v)]*100+
                            root_routes[candidates.index(v)]),v)))
           if okay else ())
    result=dict(leaves=leaves,depth=depth,nodes=attempts,
                worker_reports=attempts,reflections=attempts,
                fabric_queries=attempts,comparisons=attempts,
                root_green=okay,held_nodes=held,
                root_sha256=root_sha, fabric_kind=FABRIC_KIND,
                selected_order=order,
                auto_fire_allowed=False,
                authorization="HELD_FOR_OPERATOR" if okay
                              else "HELD_CORRECTION",
                records_sha256=hash_string("\n".join(evidence)+"\n"))
    return result,tuple(evidence)


def fixture_tests():
    rows=tuple((i,clause) for i,clause in enumerate((
        (1,-2,3),(-1,2,-3),(1,2,3),(-1,-2,3),
        (2,3,-1),(3,-2,1),(1,2,-3)) * 81))
    candidates=(1,2,3)
    covered=0
    for leaves in LEVELS:
        good,records=nested_triad(rows,candidates,leaves)
        assert good["root_green"]
        assert good["fabric_queries"]==good["nodes"]
        assert len(records)==5*good["nodes"]
        assert not good["auto_fire_allowed"]
        assert good["selected_order"]
        # For every nesting depth, corrupt all three distinct seats.
        depth=good["depth"]
        for level in range(1,depth+1):
            addr="R"+".0"*level
            for role in ("WORK","REFLECT","FABRIC"):
                bad,rows_=nested_triad(
                    rows,candidates,leaves,inject=(addr,role))
                assert not bad["root_green"]
                assert bad["authorization"]=="HELD_CORRECTION"
                assert bad["selected_order"]==()
                assert bad["held_nodes"]>=level+1
                assert any("TRIADFABRICVERDICT|triad_id="+addr in r
                           for r in rows_)
                covered+=1
        print("FABRIC_TRIAD_TEST|leaves="+str(leaves)+
              "|nodes="+str(good["nodes"])+
              "|fabric_queries="+str(good["fabric_queries"])+
              "|three_seats=PASS|root_held=PASS|json=0",flush=True)
    assert covered==30
    print("FABRIC_TRIAD_FINAL|levels=3,9,27,81|"
          "corrupted_roles_and_depths=30|caught=30|"
          "real_fabric_api=NOT_CALLED|fabric_mode=LOCAL_CNF_GRAPH|"
          "auto_fire_allowed=false|status=PASS")


if __name__=="__main__":
    fixture_tests()
