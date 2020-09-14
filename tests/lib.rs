extern crate nom_sql_learning;

use std::fs::File;
use std::io::Read;

fn parse_queryset(queries: Vec<&str>) {
    let mut parsed_ok = Vec::new();
    let mut parsed_err = 0;

    for query in queries.iter() {
        match nom_sql_learning::parser::parse_query(query) {
            Ok(q) => {
                parsed_ok.push(q);
            }
            Err(_) => parsed_err += 1,
        }
    }
    println!("Parsing failed: {} queries", parsed_err);
    println!("Parsed successfully : {} queries", parsed_ok.len());
    for q in parsed_ok.iter() {
        println!("{:?}", q);
    }
}

#[test]
fn hotcrp_queries() {
    let mut f = File::open("tests/hotcrp-queries.txt").unwrap();

    let mut s = String::new();

    f.read_to_string(&mut s).unwrap();

    let lines: Vec<&str> = s.split("\n").collect();
    let lines: Vec<&str> = s.lines().filter(|l| !l.starts_with("#")).collect();

    println!("Loaded {} total queries", lines.len());

    parse_queryset(lines);
}


#[test]
fn hyrise_test_queries() {
    let mut file = File::open("tests/hyrise-test-queries.txt").unwrap();
    let mut s = String::new();

    file.read_to_string(&mut s).unwrap();

    let lines: Vec<&str> = s.split("\n").collect();
    let lines: Vec<&str> = s.lines().filter(|l| !l.starts_with("#")).collect();
    println!("Loaded {} Hyrise queries", lines.len());

    parse_queryset(lines);
}
