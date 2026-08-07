# Install & CLI

## Install

**CLI** — installs the `marsdb` binary:

```
cargo install marsdb-cli
```

Or on macOS/Linux via Homebrew ([tap](https://github.com/knoguchi/homebrew-marsdb)):

```
brew install knoguchi/marsdb/marsdb
```

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

The REPL accepts any Cypher statement terminated by `;`. Ctrl-D exits.

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

```
$ marsdb :memory:
MarsDB graph database. Enter Cypher statements terminated by `;`. Ctrl-D to exit.
marsdb> CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'});
marsdb> MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name;
a.name | b.name
Alice | Bob
```
