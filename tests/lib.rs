extern crate nom_sql_learning;

use std::fs::File;
use std::io::Read;

#[test]
fn hotcrp_queries() {
    let mut f = File::open("tests/hotcrp-queries.txt").unwrap();

    let mut s = String::new();

    f.read_to_string(&mut s).unwrap();

    let lines: Vec<&str> = s.split("\n").collect();

    println!("Loaded {} total queries", lines.len());

    let mut parsed_ok = 0;
    let mut parsed_err = 0;

    for query in lines.iter() {
        match nom_sql_learning::parser::parse_query(query) {
            Ok(_) => parsed_ok += 1,
            Err(_) => parsed_err += 1,
        }
    }

    println!("Parsed successfully : {} queries", parsed_ok);
    println!("Parsing failed: {} queries", parsed_err);
}

