# Gremlin bench (AWS Neptune / TinkerPop)

Gremlin translation of `marsdb/benches/ldbc_ops.rs` — same chain dataset,
same four query groups (WITH-chaining, OPTIONAL MATCH, undirected 1-hop,
variable-length). The harness self-checks row counts against what the
Cypher versions return, so a translation drift fails loudly instead of
benchmarking the wrong query.

## Setup

```bash
python3 -m venv .venv
.venv/bin/pip install "gremlinpython==3.7.3"
```

gremlinpython must stay on 3.7.x: Neptune's Gremlin engine implements
TinkerPop 3.7, and 3.8+ clients emit steps (e.g. `discard()`) the server
rejects with `599: Could not locate method`.

## Local (TinkerPop Gremlin Server)

```bash
docker run -d --name mars-gremlin-bench -p 8182:8182 tinkerpop/gremlin-server:3.7.3
.venv/bin/python bench_gremlin.py
```

## Neptune

```bash
.venv/bin/python bench_gremlin.py \
  --endpoint wss://<cluster-endpoint>.<region>.neptune.amazonaws.com:8182/gremlin
```

Neptune is VPC-only. Run the harness from an EC2 instance in the same
subnet/AZ as the cluster — from outside the VPC (SSH tunnel, laptop) the
numbers are dominated by WAN round-trip, not query cost. In-AZ RTT is a
few hundred µs; every reported latency includes one round-trip plus
serialization, so sub-millisecond marsdb in-process numbers and these
client-observed numbers are not directly comparable — compare
Neptune-vs-marsdb only through a client that pays the same network hop,
or treat the RTT floor as a stated constant.

If the cluster enforces IAM auth, the request must be SigV4-signed;
this harness doesn't sign (use a cluster with IAM auth disabled for
benching, or add SigV4 via `boto3` session headers).

## Flags

- `--endpoint` (default `ws://localhost:8182/gremlin`)
- `--iterations` / `--warmup` (default 20 / 5)
- `--sizes` chain sizes for the three scaling groups (default 100 1000 10000)
