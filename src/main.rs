use json;
use std::io;
use termcolor::Color;

use mfqp;


fn main() {
    let data_url: String = String::from("https://qp.metakgp.org/data/data.json");
    println!("Fetching JSON file from {} ...", data_url);
    let json_string = mfqp::get_json_string(&data_url);
    println!("Fetched JSON file.");
    let parsed = json::parse(&json_string).unwrap();
    let mut input = String::new();
    mfqp::print_in_color("Enter the name of the paper to search", Color::Yellow);
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input = input.trim().to_string();
    println!("Reading through {} entries ...", parsed.len());
    let mut list = Vec::new();
    mfqp::interpret_json(&parsed, &mut list, &input);
    println!("Found {} matches.\n", list.len());
    for paper in &list {
        println!("{}", paper);
        println!("--------------------------------");
    }
}
