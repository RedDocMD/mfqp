use curl::easy::Easy;
use std::str;
use json;

fn main() {
    let data_url: String = String::from("https://qp.metakgp.org/data/data.json");
    println!("Request URL: {}", data_url);
    let json_string = get_json_string(&data_url);
    println!("Length of json response: {}", json_string.len());
    let parsed = json::parse(&json_string).unwrap();
    println!("Is array: {}", parsed.is_array());
    println!("Array length: {}", parsed.len());
    for (index, member) in parsed.members().enumerate() {
        if index == 1 {
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
