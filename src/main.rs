use json;
use std::error::Error;
use std::io;
use std::process;
use termcolor::Color;

use mfqp;

fn main() -> Result<(), Box<dyn Error>> {
    let data_url = String::from("https://qp.metakgp.org/data/data.json");
    println!("Fetching JSON file from {} ...", data_url);

    let json_string = mfqp::get_json_string(&data_url).unwrap_or_else(|_err| {
        mfqp::print_in_color("Failed to fetch JSON file", Color::Red);
        process::exit(1)
    });
    println!("Fetched JSON file.");

    let parsed = json::parse(&json_string).unwrap_or_else(|_err| {
        mfqp::print_in_color("Failed to parse JSON file", Color::Red);
        process::exit(1)
    });

    let mut input = String::new();
    mfqp::print_in_color("Enter the name of the paper to search", Color::Yellow);
    io::stdin().read_line(&mut input)?;
    input = input.trim().to_string();

    println!("\nReading through {} entries ...", parsed.len());
    let mut list = Vec::new();
    mfqp::interpret_json(&parsed, &mut list, &input);

    println!("\nFound {} matches.\n", list.len());
    for paper in &list {
        println!("{}", paper);
        println!("--------------------------------");
    }

    Ok(())
}
