use json;
use std::error::Error;
use std::io;
use std::process;
use termcolor::Color;

use mfqp;

fn main() -> Result<(), Box<dyn Error>> {
    let data_url = String::from("https://qp.metakgp.org/data/data.json");
    println!("Fetching JSON file from {} ...", data_url);

    let json_string = match mfqp::get_json_string(&data_url) {
        Ok(s) => s,
        Err(_) => {
            mfqp::print_in_color("Failed to fetch JSON file", Color::Red)
                .unwrap_or_else(|_err| println!("Failed to fetch JSON file"));
            process::exit(1)
        }
    };
    println!("Fetched JSON file.");

    let parsed = match json::parse(&json_string) {
        Ok(p) => p,
        Err(_) => {
            mfqp::print_in_color("Failed to parse JSON file", Color::Red)
                .unwrap_or_else(|_err| println!("Failed to parse JSON file"));
            process::exit(1)
        }
    };

    let mut input = String::new();
    mfqp::print_in_color("Enter the name of the paper to search", Color::Yellow)
        .unwrap_or_else(|_err| println!("Enter the name of the paper to search"));
    io::stdin().read_line(&mut input)?;
    input = input.trim().to_string();

    println!("Reading through {} entries ...", parsed.len());

    let mut list = Vec::new();
    mfqp::interpret_json(&parsed, &mut list, &input);

    println!("\nFound {} matches.\n", list.len());
    for paper in &list {
        println!("{}", paper);
        println!("--------------------------------");
    }

    Ok(())
}
