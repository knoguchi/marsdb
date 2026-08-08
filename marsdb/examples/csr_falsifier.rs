//! Falsifier for the build-on-demand in-RAM CSR idea (DuckPGQ-style):
//! is array-slice traversal enough faster than B-tree range traversal to
//! justify building a per-statement CSR, and what does the build cost?
//!
//! Workload: crimson_tide_collaborative_filtering's expansion core --
//! Movie<-[:RATED]-User-[:RATED]->Movie, count per recommended movie --
//! run (a) through `neighbors_in_txn` (the composite-key B-tree range
//! path v2 ships) and (b) through a CSR built at "statement start"
//! from the same store. Same algorithm, same counts, only the adjacency
//! representation differs. Not shipped -- scratch measurement tool.
//!
//! Usage: csr_falsifier <db-path> <iters>

use marsdb_graph::{Direction, GraphStore, NodeId, PropertyValue, Txn};
use std::env;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let store = GraphStore::open_file(&args[1])?;
    let iters: usize = args.get(2).map(|s| s.parse()).transpose()?.unwrap_or(100);

    let crimson = *store
        .lookup_by_index(
            "Movie",
            "title",
            &PropertyValue::String("Crimson Tide".into()),
        )?
        .first()
        .expect("Crimson Tide present");

    let read = store.begin_read()?;
    let txn = Txn::Read(&read);

    // ---- (a) B-tree range path: the shipped v2 expansion ----
    let t = Instant::now();
    let mut btree_top = Vec::new();
    for _ in 0..iters {
        btree_top = top5_btree(txn, crimson)?;
    }
    let btree_time = t.elapsed();

    // ---- build CSR for RATED, both directions, at "statement start" ----
    let t = Instant::now();
    let csr = RatedCsr::build(txn)?;
    let build_time = t.elapsed();

    // ---- (b) CSR path: identical algorithm over array slices ----
    let t = Instant::now();
    let mut csr_top = Vec::new();
    for _ in 0..iters {
        csr_top = top5_csr(&csr, crimson.0);
    }
    let csr_time = t.elapsed();

    assert_eq!(btree_top, csr_top, "both paths must agree on the result");
    println!(
        "expansion result (movie node id, raters-in-common): {:?}",
        btree_top
    );
    println!(
        "CSR build: {:?} ({} users, {} rated edges)",
        build_time,
        csr.user_offsets.len().saturating_sub(1),
        csr.user_to_movie.len()
    );
    println!(
        "btree path: {:?} total, {:.3} ms/iter",
        btree_time,
        btree_time.as_secs_f64() * 1e3 / iters as f64
    );
    println!(
        "csr path:   {:?} total, {:.3} ms/iter",
        csr_time,
        csr_time.as_secs_f64() * 1e3 / iters as f64
    );
    println!(
        "traversal speedup: {:.1}x; build amortizes over {:.1} statement(s)",
        btree_time.as_secs_f64() / csr_time.as_secs_f64(),
        build_time.as_secs_f64()
            / ((btree_time.as_secs_f64() - csr_time.as_secs_f64()) / iters as f64)
    );
    Ok(())
}

/// Top-5 recommendations via the shipped B-tree adjacency.
fn top5_btree(txn: Txn, crimson: NodeId) -> Result<Vec<(u64, u32)>, Box<dyn std::error::Error>> {
    let mut counts: std::collections::HashMap<u64, u32> = std::collections::HashMap::new();
    for rater in GraphStore::neighbors_in_txn(txn, crimson, Direction::In, Some("RATED"))? {
        for rec in GraphStore::neighbors_in_txn(txn, rater.other, Direction::Out, Some("RATED"))? {
            if rec.other != crimson {
                *counts.entry(rec.other.0).or_insert(0) += 1;
            }
        }
    }
    Ok(top5(counts.into_iter()))
}

/// Dense-id CSR over RATED, both directions. Node ids are dense (next_id
/// counter, never reused), so plain Vec indexing by id works -- the same
/// property the direct-addressing discussion identified.
struct RatedCsr {
    /// movie id -> [user ids] (incoming RATED)
    movie_offsets: Vec<u32>,
    movie_to_user: Vec<u64>,
    /// user id -> [movie ids] (outgoing RATED)
    user_offsets: Vec<u32>,
    user_to_movie: Vec<u64>,
}

impl RatedCsr {
    fn build(txn: Txn) -> Result<Self, Box<dyn std::error::Error>> {
        // One pass over every User's outgoing RATED range -- realistic
        // "statement start" build using only public API. Max id bounds the
        // dense arrays.
        let users = GraphStore::all_node_ids_limited_in_txn(txn, Some("User"), usize::MAX)?;
        let movies = GraphStore::all_node_ids_limited_in_txn(txn, Some("Movie"), usize::MAX)?;
        let max_id = users
            .iter()
            .chain(movies.iter())
            .map(|n| n.0)
            .max()
            .unwrap_or(0) as usize;

        let mut user_adj: Vec<Vec<u64>> = vec![Vec::new(); max_id + 1];
        let mut movie_in_count = vec![0u32; max_id + 1];
        for &u in &users {
            let entries = GraphStore::neighbors_in_txn(txn, u, Direction::Out, Some("RATED"))?;
            let list = &mut user_adj[u.0 as usize];
            list.reserve(entries.len());
            for e in entries {
                list.push(e.other.0);
                movie_in_count[e.other.0 as usize] += 1;
            }
        }

        // Prefix-sum both directions into packed arrays.
        let mut user_offsets = vec![0u32; max_id + 2];
        for i in 0..=max_id {
            user_offsets[i + 1] = user_offsets[i] + user_adj[i].len() as u32;
        }
        let mut user_to_movie = Vec::with_capacity(user_offsets[max_id + 1] as usize);
        for adj in &user_adj {
            user_to_movie.extend_from_slice(adj);
        }

        let mut movie_offsets = vec![0u32; max_id + 2];
        for i in 0..=max_id {
            movie_offsets[i + 1] = movie_offsets[i] + movie_in_count[i];
        }
        let mut movie_to_user = vec![0u64; movie_offsets[max_id + 1] as usize];
        let mut cursor = movie_offsets.clone();
        for (uid, adj) in user_adj.iter().enumerate() {
            for &m in adj {
                movie_to_user[cursor[m as usize] as usize] = uid as u64;
                cursor[m as usize] += 1;
            }
        }

        Ok(RatedCsr {
            movie_offsets,
            movie_to_user,
            user_offsets,
            user_to_movie,
        })
    }

    fn raters_of(&self, movie: u64) -> &[u64] {
        &self.movie_to_user[self.movie_offsets[movie as usize] as usize
            ..self.movie_offsets[movie as usize + 1] as usize]
    }

    fn movies_of(&self, user: u64) -> &[u64] {
        &self.user_to_movie[self.user_offsets[user as usize] as usize
            ..self.user_offsets[user as usize + 1] as usize]
    }
}

fn top5_csr(csr: &RatedCsr, crimson: u64) -> Vec<(u64, u32)> {
    let mut counts = vec![0u32; csr.movie_offsets.len()];
    for &user in csr.raters_of(crimson) {
        for &rec in csr.movies_of(user) {
            if rec != crimson {
                counts[rec as usize] += 1;
            }
        }
    }
    top5(
        counts
            .iter()
            .enumerate()
            .filter(|&(_, &c)| c > 0)
            .map(|(id, &c)| (id as u64, c)),
    )
}

/// Deterministic top-5: by count desc, then id asc as tiebreak.
fn top5(counts: impl Iterator<Item = (u64, u32)>) -> Vec<(u64, u32)> {
    let mut all: Vec<(u64, u32)> = counts.collect();
    all.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    all.truncate(5);
    all
}
