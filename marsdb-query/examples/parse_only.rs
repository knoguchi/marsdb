//! Parses (no execution) a file and reports statement count -- isolates
//! parse_many's own memory footprint from execute_batch's.
use std::env;
use std::fs;

fn main() {
    let path = env::args().nth(1).expect("usage: parse_only <file>");
    let input = fs::read_to_string(&path).unwrap();
    let stmts = marsdb_query::parse_many(&input).unwrap();
    println!("parsed {} statements", stmts.len());
}
