use curl::easy::Easy;
use std::str;
use json;
use json::JsonValue;
use std::io;

struct Paper {
    department: String,
    link: String,
    name: String,
    semester: String,
    year: String,
}

impl Paper {
    pub fn new(department: String, link: String, name: String, semester: String, year: String) -> Self {
        Paper { department, link, name, semester, year }
    }
}

fn main() {
    let data_url: String = String::from("https://qp.metakgp.org/data/data.json");
    println!("Fetching JSON file from {} ...", data_url);
    let json_string = get_json_string(&data_url);
    println!("Fetched JSON file.");
    let parsed = json::parse(&json_string).unwrap();
    let mut input = String::new();
    println!("Enter the name of the paper to search");
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input = input.trim().to_string();
    println!("Reading through {} entries ...", parsed.len());
    let mut list = Vec::new();
    interpret_json(&parsed, &mut list, &input);
}

fn get_json_string(url: &str) -> String {
    let mut easy = Easy::new();
    easy.url(url).unwrap();
    let mut dst = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                dst.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    str::from_utf8(&dst).unwrap().to_string()
}

fn interpret_json(parsed: &JsonValue, list: &mut Vec<Paper>, input: &str) {
    for member in parsed.members() {
        let mut add_to_list = false;
        for content in member.entries() {
            if content.0 == "Paper" {
                let val = match content.1.as_str() {
                    Some(s) => s,
                    None => ""
                };
            }
        }
    }
}
