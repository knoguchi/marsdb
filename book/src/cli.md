# CLI Reference

`marsdb` is a single binary: a REPL, a one-shot query runner, and a batch
runner, depending on the arguments you give it. (Not installed yet? See
[Getting Started](./getting-started.md).)

## Usage

```
marsdb                                  # in-memory REPL
marsdb mydata.db                        # file-backed REPL
marsdb mydata.db "MATCH (n) RETURN n"   # run one query, exit
marsdb :memory: "..."                   # explicit in-memory, one-shot
marsdb mydata.db "CREATE (a); CREATE (b); MATCH (n) RETURN n"  # ;-separated batch
marsdb mydata.db < script.cypher        # piped stdin, same ;-separated batch
marsdb mydata.db --nl "who does Alice know?"  # plain-English question via Ollama
```

Arguments:

| Argument | Meaning |
|---|---|
| `file` (positional, optional) | Database file path, or `:memory:` for a transient in-memory database. Omit entirely for in-memory. |
| `query` (positional, optional) | A Cypher query to run once, non-interactively. Omit to start the REPL — or, if stdin isn't a terminal, to read and run a `;`-separated batch from it instead. |
| `--memory` | Shorthand for an in-memory database, same as passing `:memory:` as `file`. |
| `--nl QUESTION` | Ask a plain-English question instead of Cypher; translates it via a local Ollama instance and runs it. Read-only. |

## The REPL

Starts whenever no `query` argument is given and stdin is a terminal.
Enter any Cypher statement terminated by `;`:

```
$ marsdb :memory:
MarsDB graph database. Enter Cypher statements terminated by `;`. Ctrl-D to exit.
marsdb> CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'});
marsdb> MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name;
a.name | b.name
Alice | Bob
```

Ctrl-D exits.

## Meta-commands

The REPL also has sqlite-style dot commands for quick schema
introspection. Each is a thin formatter over a built-in `CALL db.*`
procedure (so the same introspection is available from plain Cypher too,
in any binding). Meta-commands run immediately — no trailing `;` needed,
though one is tolerated.

| Command | Shows |
|---|---|
| `.help` | This list. |
| `.labels` | Node labels with node counts. |
| `.types` | Relationship types with edge counts. |
| `.props` | Property keys in use. |
| `.indexes` | Declared indexes. |
| `.schema` | Labels, relationship types, and indexes in one view. |

Example, against a database with one `:Person`, one `:Movie`, a
`WATCHED` relationship between them, and a unique index on
`:Person(name)`:

```
marsdb> .labels
Movie  (1)
Person  (1)
marsdb> .types
WATCHED  (1)
marsdb> .props
age
name
stars
title
marsdb> .indexes
:Person(name) UNIQUE
marsdb> .schema
labels:
  Movie  (1)
  Person  (1)

relationship types:
  WATCHED  (1)

indexes:
  :Person(name) UNIQUE
```

An unrecognized `.command` prints the same list as `.help`.

## Natural language queries

`--nl` translates a plain-English question into Cypher and runs it, using a
local [Ollama](https://ollama.com) instance:

```
ollama serve &
ollama pull llama3.2
marsdb mydata.db --nl "how many people are there?"
```

Set `OLLAMA_MODEL` to use a model other than the default `llama3.2`. The
generated Cypher is printed before its results. Generated writes are
rejected — `--nl` only ever runs read-only queries. The translation itself
validates the generated Cypher's syntax and variable/type binding against
the database's actual schema, retrying once with the validation error fed
back if the first attempt doesn't parse — see the `marsdb-nl2cypher` crate
for the underlying library.
