#!/usr/bin/env python3
"""Generates a Cypher batch (one `;`-separated statement per line, see
DEMO.md) that builds a graph of this repo's own git history: one `Commit`
node per commit, one `File` node per file ever touched, and a `TOUCHES`
edge from a commit to every file it modified.

Usage:
    python3 examples/commit_graph.py > commit_graph.cypher

MarsDB's string literals have no escape mechanism (no `\\'`), so this
strips single quotes out of commit subjects/author names rather than
escaping them -- see DEMO.md for why.
"""

import subprocess
import sys


def esc(s: str) -> str:
    return s.replace("'", "")


def main() -> None:
    out = subprocess.run(
        ["git", "log", "--pretty=format:@@%h|%an|%s", "--all", "--name-only"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout

    commits = []
    cur = None
    for line in out.splitlines():
        if line.startswith("@@"):
            if cur:
                commits.append(cur)
            h, author, subject = line[2:].split("|", 2)
            cur = {"hash": h, "author": author, "subject": subject, "files": []}
        elif line.strip():
            cur["files"].append(line.strip())
    if cur:
        commits.append(cur)

    files = sorted({f for c in commits for f in c["files"]})

    stmts = []

    # One CREATE for all Commit nodes -- independent patterns, no shared
    # variables needed, so a single comma-separated CREATE is enough.
    commit_patterns = [
        f"(:Commit {{hash: '{c['hash']}', author: '{esc(c['author'])}', subject: '{esc(c['subject'])}'}})"
        for c in commits
    ]
    stmts.append("CREATE " + ", ".join(commit_patterns))

    # One CREATE for all File nodes.
    file_patterns = [f"(:File {{path: '{esc(f)}'}})" for f in files]
    stmts.append("CREATE " + ", ".join(file_patterns))

    # Standalone CREATE can never connect two nodes that already exist --
    # every node token it sees always becomes a fresh node. MATCH...CREATE
    # (added specifically because this demo needed it) is what makes this
    # possible: each MATCH binds one already-existing node, WITH carries
    # it into a second MATCH that binds the other, and CREATE reuses both
    # bound variables instead of creating new nodes.
    for c in commits:
        for f in c["files"]:
            stmts.append(
                f"MATCH (c:Commit {{hash: '{c['hash']}'}}) WITH c "
                f"MATCH (f:File {{path: '{esc(f)}'}}) "
                f"CREATE (c)-[:TOUCHES]->(f)"
            )

    print("; ".join(stmts))


if __name__ == "__main__":
    sys.exit(main())
