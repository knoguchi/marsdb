# Install & CLI

## Install

**CLI** — installs the `mars` binary:

```
cargo install marsdb-cli
```

Or on macOS/Linux via Homebrew ([tap](https://github.com/knoguchi/homebrew-marsdb)):

```
brew install knoguchi/marsdb/marsdb
```

## Usage

```
mars                                  # in-memory REPL
mars mydata.db                        # file-backed REPL
mars mydata.db "MATCH (n) RETURN n"   # run one query, exit
mars :memory: "..."                   # explicit in-memory, one-shot
mars mydata.db "CREATE (a); CREATE (b); MATCH (n) RETURN n"  # ;-separated batch
mars mydata.db < script.cypher        # piped stdin, same ;-separated batch
```

The REPL accepts any Cypher statement terminated by `;`. Ctrl-D exits.

```
$ mars :memory:
MarsDB graph database. Enter Cypher statements terminated by `;`. Ctrl-D to exit.
mars> CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'});
mars> MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name;
a.name | b.name
Alice | Bob
```
