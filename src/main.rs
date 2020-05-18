use curl::easy::Easy;
use std::str;
use json;
use json::JsonValue;

struct Paper {
    department: String,
    link: String,
    name: String,
    semester: String,
    year: String,
}

fn main() {
    let data_url: String = String::from("https://qp.metakgp.org/data/data.json");
    println!("Fetching JSON file from {} ...", data_url);
    let json_string = get_json_string(&data_url);
    println!("Fetched JSON file.");
    let parsed = json::parse(&json_string).unwrap();
    println!("Reading through {} entries ...", parsed.len());
    interpret_json(&parsed);
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

fn interpret_json(parsed: &JsonValue) {
    for (index, member) in parsed.members().enumerate() {
        if index == 2534 {
            for content in member.entries() {
                let val = match content.1.as_str() {
                    Some(s) => s,
                    None => ""
                };
                println!("{}: {}", content.0, val);
            }
        }
    }
}
