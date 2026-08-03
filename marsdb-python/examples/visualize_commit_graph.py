#!/usr/bin/env python3
"""Renders the commit/file graph built by `commit_graph.py` (see DEMO.md)
as a PNG — a bipartite layout, Commit nodes on the left, File nodes on
the right (only the highest-churn ones, or the graph is an unreadable
hairball — 50+ files touched more than once turns out to be too many to
read individual edges from, even though it renders fine), sized by how
many commits touched them.

Uses the `marsdb` package directly (the actual Python bindings, not the
CLI) -- `pip install marsdb networkx matplotlib`.

Usage:
    python3 examples/commit_graph.py > commit_graph.cypher
    marsdb commit_graph.db "$(cat commit_graph.cypher)"
    python3 examples/visualize_commit_graph.py commit_graph.db commit_graph.png [min-touches]
"""

import sys

import marsdb
import matplotlib.pyplot as plt
import networkx as nx


def short_labels(paths: list[str]) -> dict[str, str]:
    """Shortest trailing-path-component suffix that's still unique across
    `paths` -- this repo has several distinct files that share a 2-component
    suffix (e.g. `marsdb/src/lib.rs` and `marsdb-storage/src/lib.rs` are
    both `src/lib.rs`), so a fixed "last two components" rule silently
    collides two different File nodes onto one label. Grows the suffix
    per-path only as far as needed to stay unique."""
    labels: dict[str, str] = {}
    for depth in range(1, max(p.count("/") for p in paths) + 2):
        candidates = {p: "/".join(p.split("/")[-depth:]) for p in paths if p not in labels}
        counts: dict[str, int] = {}
        for label in candidates.values():
            counts[label] = counts.get(label, 0) + 1
        for path, label in candidates.items():
            if counts[label] == 1:
                labels[path] = label
        if len(labels) == len(paths):
            break
    labels.update({p: p for p in paths if p not in labels})  # fallback: full path
    return labels


def main() -> None:
    if len(sys.argv) not in (3, 4):
        print(f"usage: {sys.argv[0]} <db-path> <output-png> [min-touches=5]", file=sys.stderr)
        sys.exit(1)
    db_path, out_path = sys.argv[1], sys.argv[2]
    min_touches = int(sys.argv[3]) if len(sys.argv) == 4 else 5

    db = marsdb.Database.open(db_path)

    # Below this threshold there are too many single/double-edge File
    # nodes for the layout to stay readable -- see the module docstring.
    churned = db.execute(
        f"MATCH (c:Commit)-[:TOUCHES]->(f:File) "
        f"WITH f.path AS path, count(c) AS touches WHERE touches > {min_touches} "
        f"RETURN path, touches"
    )
    touches = {row["path"]: row["touches"] for row in churned}
    labels = short_labels(list(touches.keys()))

    edges = db.execute("MATCH (c:Commit)-[:TOUCHES]->(f:File) RETURN c.hash, f.path")

    g = nx.Graph()
    for row in edges:
        path = row["f.path"]
        if path not in touches:
            continue
        commit_node = f"commit:{row['c.hash']}"
        file_node = f"file:{path}"
        g.add_node(commit_node, kind="commit")
        g.add_node(file_node, kind="file", touches=touches[path], label=labels[path])
        g.add_edge(commit_node, file_node)

    commit_nodes = [n for n, d in g.nodes(data=True) if d["kind"] == "commit"]
    file_nodes = sorted(
        (n for n, d in g.nodes(data=True) if d["kind"] == "file"),
        key=lambda n: -g.nodes[n]["touches"],
    )
    pos = nx.bipartite_layout(g, commit_nodes)

    plt.figure(figsize=(16, max(8, 0.35 * len(file_nodes))))
    nx.draw_networkx_edges(g, pos, alpha=0.35, width=1.2)
    nx.draw_networkx_nodes(g, pos, nodelist=commit_nodes, node_color="#4C72B0", node_size=60, label="Commit")
    file_sizes = [100 + 40 * g.nodes[n]["touches"] for n in file_nodes]
    nx.draw_networkx_nodes(g, pos, nodelist=file_nodes, node_color="#DD8452", node_size=file_sizes, label="File")
    nx.draw_networkx_labels(
        g,
        pos,
        labels={n: g.nodes[n]["label"] for n in file_nodes},
        font_size=8,
        horizontalalignment="left",
    )
    plt.legend(scatterpoints=1, loc="upper left")
    plt.title(
        f"Commit -> File graph ({len(commit_nodes)} commits, "
        f"{len(file_nodes)} files touched >{min_touches}x, {g.number_of_edges()} edges)"
    )
    plt.axis("off")
    plt.xlim(-1.1, 2.0)  # room for right-side labels past bipartite_layout's default [-1,1] span
    plt.tight_layout()
    plt.savefig(out_path, dpi=150)
    print(f"wrote {out_path} ({len(commit_nodes)} commits, {len(file_nodes)} files, {g.number_of_edges()} edges)")


if __name__ == "__main__":
    main()
