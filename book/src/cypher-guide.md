# The Cypher Guide

A walkthrough of Cypher itself, one clause at a time, building up a
small graph as we go. Every example runs as-is in the CLI REPL
(`marsdb :memory:`) or through any language binding — the Cypher text
is identical everywhere. For the full list of what's implemented, see
the [Cypher Language Reference](./cypher-support.md).

## Creating nodes

```cypher
CREATE (:Person {name: 'Alice', age: 30, city: 'Boston'})
CREATE (:Person {name: 'Bob', age: 27, city: 'Boston'})
CREATE (:Person {name: 'Carol', age: 35, city: 'Seattle'})
```

`:Person` is a label; `{...}` is the node's property map. A node needs
neither — `CREATE (n)` works — but an unlabeled node is rarely useful.

## Creating relationships

A relationship is written inside a pattern, between two nodes:

```cypher
CREATE (a:Person {name: 'Dave'})-[:KNOWS]->(b:Person {name: 'Eve'})
```

This creates *both* nodes and the relationship between them in one
statement. That matters: reusing a variable name across two separate
`CREATE` patterns creates two different nodes, not one — `CREATE
(a:Person {name: 'Alice'}), (a)-[:KNOWS]->(b:Person {name: 'Bob'})`
does not mean "connect the Alice created above." To connect nodes that
already exist, match them first:

```cypher
MATCH (a:Person {name: 'Alice'}), (b:Person {name: 'Carol'})
CREATE (a)-[:KNOWS]->(b)
```

```cypher
MATCH (c:Person {name: 'Carol'}), (d:Person {name: 'Dave'})
CREATE (c)-[:KNOWS]->(d)
```

The graph so far: `Alice -> Carol -> Dave -> Eve`, plus Bob, connected to
no one.

## Reading data: MATCH and RETURN

```cypher
MATCH (p:Person) RETURN p.name, p.age
MATCH (p:Person {city: 'Boston'}) RETURN p.name
MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name
```

`RETURN p` (no property access) returns the whole node; `RETURN *`
returns every variable bound so far. Relationships work the same way as
nodes: `MATCH ()-[r:KNOWS]->() RETURN r` binds the relationship itself.

## Filtering: WHERE

```cypher
MATCH (p:Person) WHERE p.age > 30 RETURN p.name
MATCH (p:Person) WHERE p.name STARTS WITH 'A' RETURN p.name
MATCH (a:Person), (b:Person)
WHERE a.city = b.city AND a.name <> b.name
RETURN a.name, b.name
```

A pattern itself can be a filter condition:

```cypher
MATCH (a:Person)
WHERE exists { (a)-[:KNOWS]->(:Person {city: 'Seattle'}) }
RETURN a.name
```

## Traversing further: variable-length paths

`[:KNOWS*1..3]` matches 1 to 3 `KNOWS` hops in a row — "friends,
friends of friends, up to 3 hops out":

```cypher
MATCH (:Person {name: 'Alice'})-[:KNOWS*1..3]->(f:Person)
RETURN DISTINCT f.name
```

Bounds can be open (`*1..`, `*..3`) or omitted entirely (`*`, unbounded).
`shortestPath` finds the shortest connection between two nodes over a
variable-length relationship. Both endpoints must already be bound by
an earlier `MATCH` — a fresh label/property filter can't be declared
inline inside `shortestPath(...)`:

```cypher
MATCH (a:Person {name: 'Alice'}) MATCH (b:Person {name: 'Eve'})
MATCH p = shortestPath((a)-[:KNOWS*]-(b))
RETURN length(p)
```

## Updating: SET

```cypher
MATCH (p:Person {name: 'Alice'}) SET p.age = 31
MATCH (p:Person {name: 'Alice'}) SET p += {city: 'Cambridge', verified: true}
MATCH (p:Person {name: 'Alice'}) SET p:VIP
```

`SET p.field = value` sets one property. `SET p += {...}` merges a map
in, leaving properties not mentioned untouched. `SET p = {...}`
(no `+`) replaces the whole property map. `SET p:Label` adds a label
without touching properties.

## Create-or-update: MERGE

`MERGE` matches a pattern if it exists, or creates it if it doesn't —
useful for "insert this node unless it's already there":

```cypher
MERGE (p:Person {name: 'Alice'})
ON CREATE SET p.firstSeen = 'today'
ON MATCH SET p.lastSeen = 'today'
```

`MERGE` is capped at one relationship hop per statement (`MERGE (a)-[:KNOWS]->(b)`
is fine; a longer chain in one `MERGE` isn't).

## Deleting

```cypher
MATCH (p:Person {name: 'Bob'}) DELETE p
```

Bob has no relationships, so plain `DELETE` works. Carol does — deleting
her the same way fails:

```cypher
MATCH (p:Person {name: 'Carol'}) DELETE p
MATCH (p:Person {name: 'Carol'}) DETACH DELETE p
```

`DETACH DELETE` removes the attached relationships first, then the node.

## Aggregating

```cypher
MATCH (p:Person) RETURN p.city, count(*) AS people
MATCH (p:Person) RETURN p.city, collect(p.name) AS names
MATCH (:Person)-[:KNOWS]->(f:Person) RETURN count(DISTINCT f) AS unique_friends
```

Any bare (non-aggregating) expression in the same `RETURN` becomes an
implicit `GROUP BY` key — there's no separate `GROUP BY` clause.

## Ordering, paging, and dedup

```cypher
MATCH (p:Person) RETURN p.name ORDER BY p.age DESC LIMIT 2
MATCH (p:Person) RETURN DISTINCT p.city
```

## Parameters

```cypher
MATCH (p:Person {name: $name}) RETURN p.age
```

Send `$name` as a bound parameter rather than interpolating it into the
query text — see the per-language binding pages
([Rust](./embedding-rust.md), [Python](./python.md), [Go](./go.md)) for
exactly how to pass parameters from each.

## Where to go next

- [Cypher Language Reference](./cypher-support.md) — everything
  implemented, and the current known gaps.
- [Embedding in Rust](./embedding-rust.md), [Python bindings](./python.md),
  [Go bindings](./go.md) — running these queries from a program.
