# marsdb-nl2cypher

Natural-language -> Cypher translation for [MarsDB](https://github.com/knoguchi/marsdb):
introspects a database's actual schema, builds a grounded prompt, calls a
caller-supplied LLM client, and validates the result by really parsing it —
with one bounded repair attempt if the first try doesn't parse.

```rust
use marsdb::Database;
use marsdb_nl2cypher::{translate_and_run, LlmClient};

let db = Database::in_memory()?;
db.execute("CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'})")?;

let (cypher, result) = translate_and_run(&db, &my_llm_client, "who does Alice know?")?;
```

## Why this exists

MarsDB's Cypher subset is narrower than full Neo4j Cypher — no `RETURN
DISTINCT`, `MERGE` capped at one relationship hop, named-path capture is
fixed-hop only, and so on (see the [main README's Cypher-coverage
section](../README.md#cypher-coverage)). That narrower grammar is actually
a *good* target for LLM-generated queries: fewer ways to generate something
unparseable than against Neo4j's much larger surface. This crate leans into
that — the prompt tells the model both what's supported and, explicitly,
what to avoid, since an LLM trained on general Cypher will otherwise reach
for syntax MarsDB doesn't have (the bare `-->` shorthand being the most
common one).

## No HTTP/LLM-SDK dependency

The core crate depends on nothing network-related — bring your own
`LlmClient`:

```rust
pub trait LlmClient {
    fn complete(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>>;
}
```

Implement it against OpenAI, Anthropic, a local Ollama instance, or
anything else that turns a prompt into text.

## What it does

1. **`introspect_schema(&db)`** — runs `MATCH (n) RETURN n` and `MATCH
   ()-[r]->() RETURN r` against the existing public `Database` API and
   aggregates: which labels/relationship-types exist, how many of each, and
   the union of property keys observed on each. No new storage-layer API —
   this is an `O(graph size)` scan, same cost a dedicated schema index would
   still pay for something that only needs to run once before a
   translation, not per query.
2. **`build_prompt(&schema, question, prior_attempt)`** — schema summary +
   a hand-written, deliberately short capability/gap list + the question.
   `prior_attempt` turns this into a repair prompt instead of a fresh one.
3. **`translate(&client, &schema, question)`** — calls the LLM, strips a
   markdown code fence if present, tries `marsdb_query::parse`. If that
   fails, feeds the *exact* parse error back to the LLM for one repair
   attempt (this codebase's parse errors are written to be genuinely
   readable, which this leans on directly) — not an open-ended retry loop.
   Fails with both attempts included if the repair also doesn't parse.
4. **`translate_and_run(&db, &client, question)`** — the common case in one
   call: introspect, translate, execute.

## Example

A real, runnable demo against a local [Ollama](https://ollama.com) instance
(`examples/ollama_demo.rs`) — nothing above depends on Ollama specifically,
this is just one concrete `LlmClient`:

```
ollama serve &
ollama pull llama3.2
cargo run -p marsdb-nl2cypher --example ollama_demo

# or use a model you already have pulled:
OLLAMA_MODEL=qwen3-coder:30b cargo run -p marsdb-nl2cypher --example ollama_demo
```

## What this doesn't do

The repair loop is scoped to **parse failures**, not semantic correctness —
it can't tell you the LLM matched `'engineer'` when your data actually has
`'Engineer'` and got zero rows back. That's a normal, expected text2cypher
accuracy limitation, not something a syntax-validation loop can fix; it
would need a fundamentally different mechanism (few-shot examples, data
normalization, a correctness oracle) to address, and isn't attempted here.

There's also no list-valued `$parameter` support yet (tracked in the main
README's roadmap), so schema values can't currently be handed to the LLM's
generated query as a batch parameter — the LLM has to inline literals
directly into the Cypher text it produces.

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE)
or [MIT license](../LICENSE-MIT) at your option, same as the rest of
MarsDB.
