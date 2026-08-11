#!/usr/bin/env python3
"""Gremlin translation of marsdb/benches/ldbc_ops.rs, runnable against any
TinkerPop-compatible endpoint (local Gremlin Server or AWS Neptune).

Dataset: the same chain shape the Rust bench builds --
(n0:Item {idx:0})-[:R]->(n1:Item {idx:1})-> ... ->(n_hops:Item {idx:hops})

Query groups (Cypher -> Gremlin):

  with_chaining:
    MATCH (n:Item)-[:R]->(m:Item)
    WITH m, m.idx AS idx ORDER BY idx DESC LIMIT 10
    MATCH (m)-[:R]->(k:Item) RETURN idx, k.idx

  optional_match:
    MATCH (n:Item) OPTIONAL MATCH (n)-[:R]->(m:Item) RETURN n.idx, m.idx

  undirected_1hop:
    MATCH (n:Item)-[:R]-(m:Item) RETURN m.idx LIMIT 10

  variable_length:
    MATCH (n:Item {idx: 0})-[:R*1..5]->(m:Item) RETURN m.idx   (chain 1000)
    MATCH (n:Item {idx: 0})-[:R*1..30]->(m:Item) RETURN m.idx  (chain 1000)
    MATCH (n:Item {idx: 0})-[:R*0..]->(m:Item) RETURN m.idx    (chain 25)

Usage:
  local:   python bench_gremlin.py
  neptune: python bench_gremlin.py \
             --endpoint wss://<cluster>.<region>.neptune.amazonaws.com:8182/gremlin
"""

import argparse
import statistics
import sys
import time

from gremlin_python.driver.driver_remote_connection import DriverRemoteConnection
from gremlin_python.process.anonymous_traversal import traversal
from gremlin_python.process.graph_traversal import __
from gremlin_python.process.traversal import Order

VERTEX_CHUNK = 100
EDGE_CHUNK = 50


def load_chain(g, hops):
    """Create the (n0)-[:R]->(n1)->...->(n_hops) chain; returns load seconds."""
    t0 = time.perf_counter()
    n = hops + 1  # node count, matching the Rust bench's `(n0)...(n_hops)`
    for start in range(0, n, VERTEX_CHUNK):
        t = g
        for i in range(start, min(start + VERTEX_CHUNK, n)):
            t = t.add_v("Item").property("idx", i)
        t.iterate()
    # idx is unique per chain; fetch ids in idx order to wire edges by id
    # (avoids per-edge property lookups, which TinkerGraph doesn't index).
    ids = g.V().has_label("Item").order().by("idx").id_().to_list()
    assert len(ids) == n, f"expected {n} vertices, got {len(ids)}"
    for start in range(0, hops, EDGE_CHUNK):
        t = g
        for i in range(start, min(start + EDGE_CHUNK, hops)):
            t = t.V(ids[i]).as_("a").V(ids[i + 1]).add_e("R").from_("a")
        t.iterate()
    return time.perf_counter() - t0


def drop_all(g):
    g.V().drop().iterate()


def q_with_chaining(g):
    return (
        g.V().has_label("Item")
        .out("R").has_label("Item")
        .order().by("idx", Order.desc).limit(10).as_("m")
        .out("R").has_label("Item").as_("k")
        .select("m", "k").by("idx").by("idx")
        .to_list()
    )


def q_optional_match(g):
    return (
        g.V().has_label("Item").as_("n")
        .coalesce(
            __.out("R").has_label("Item").values("idx"),
            __.constant(-1),  # stands in for Cypher's null m.idx
        ).as_("midx")
        .select("n", "midx").by("idx").by()
        .to_list()
    )


def q_undirected_1hop(g):
    return (
        g.V().has_label("Item")
        .both("R").has_label("Item")
        .values("idx").limit(10)
        .to_list()
    )


def q_var_bounded(g, max_hops):
    return (
        g.V().has("Item", "idx", 0)
        .repeat(__.out("R")).emit().times(max_hops)
        .values("idx")
        .to_list()
    )


def q_var_unbounded(g):
    # *0.. -- emit() before repeat() includes the depth-0 start vertex
    return (
        g.V().has("Item", "idx", 0)
        .emit().repeat(__.out("R"))
        .values("idx")
        .to_list()
    )


def expected_rows(query, hops):
    """Row counts the Cypher versions produce on a chain of `hops` hops."""
    if query == "with_chaining":
        # top-10 m by idx desc = hops..hops-9; m=idx hops has no outgoing R
        return min(9, hops - 1) if hops >= 1 else 0
    if query == "optional_match":
        return hops + 1  # one row per node, tail row with null m.idx
    if query == "undirected_1hop":
        return min(10, 2 * hops)
    raise ValueError(query)


def bench(name, fn, warmup, iters, expect=None):
    for _ in range(warmup):
        rows = fn()
    if expect is not None and len(rows) != expect:
        print(f"  {name}: SEMANTICS MISMATCH -- {len(rows)} rows, expected {expect}")
        sys.exit(1)
    samples = []
    for _ in range(iters):
        t0 = time.perf_counter()
        fn()
        samples.append((time.perf_counter() - t0) * 1e3)
    samples.sort()
    mean = statistics.fmean(samples)
    p50 = samples[len(samples) // 2]
    p95 = samples[int(len(samples) * 0.95)]
    print(f"  {name}: mean {mean:.2f} ms  p50 {p50:.2f} ms  p95 {p95:.2f} ms  ({len(rows)} rows)")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--endpoint", default="ws://localhost:8182/gremlin")
    ap.add_argument("--iterations", type=int, default=20)
    ap.add_argument("--warmup", type=int, default=5)
    ap.add_argument("--sizes", type=int, nargs="+", default=[100, 1_000, 10_000])
    args = ap.parse_args()

    conn = DriverRemoteConnection(args.endpoint, "g")
    g = traversal().with_remote(conn)
    print(f"endpoint: {args.endpoint}")
    print(f"iterations: {args.iterations} (warmup {args.warmup})\n")

    try:
        for hops in args.sizes:
            drop_all(g)
            secs = load_chain(g, hops)
            print(f"chain {hops} (load {secs:.1f}s):")
            for name, fn in [
                ("with_chaining", lambda: q_with_chaining(g)),
                ("optional_match", lambda: q_optional_match(g)),
                ("undirected_1hop", lambda: q_undirected_1hop(g)),
            ]:
                bench(name, fn, args.warmup, args.iterations,
                      expect=expected_rows(name, hops))
            print()

        drop_all(g)
        secs = load_chain(g, 1_000)
        print(f"chain 1000 (load {secs:.1f}s):")
        bench("var_length 1..5", lambda: q_var_bounded(g, 5),
              args.warmup, args.iterations, expect=5)
        bench("var_length 1..30", lambda: q_var_bounded(g, 30),
              args.warmup, args.iterations, expect=30)
        print()

        drop_all(g)
        secs = load_chain(g, 25)
        print(f"chain 25 (load {secs:.1f}s):")
        bench("var_length 0..unbounded", lambda: q_var_unbounded(g),
              args.warmup, args.iterations, expect=26)
        drop_all(g)
    finally:
        conn.close()


if __name__ == "__main__":
    main()
