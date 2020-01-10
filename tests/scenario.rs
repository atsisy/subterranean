extern crate suzu;

use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[test]
fn check_scenario_parsing() {
    let content = std::fs::read_to_string("./tests/scenario_parsing_test.toml").unwrap();
    let root = content.parse::<toml::Value>().unwrap();

    let array = root["script"].as_array().unwrap();
    println!("{:?}", array);

    for elem in array {
        let table = elem.as_table().unwrap();
        println!("fpc = {}, text = {}", table["fpc"].as_integer().unwrap(), table["text"].as_str().unwrap());
    }

    let tachie_array = root["using-tachie"].as_array().unwrap();
    for elem in tachie_array {
        println!("{}", elem.as_str().unwrap());
    }
}
