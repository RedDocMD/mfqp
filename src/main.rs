use curl::easy::Easy;
use json;
use json::JsonValue;
use std::fmt;
use std::io;
use std::str;
use sublime_fuzzy;

struct Paper {
    department: String,
    link: String,
    name: String,
    semester: String,
    year: String,
}

impl Paper {
    pub fn new() -> Self {
        Paper {
            department: String::new(),
            link: String::new(),
            name: String::new(),
            semester: String::new(),
            year: String::new(),
        }
    }
}

impl fmt::Display for Paper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Department: {}  Name: {}\nSemester: {}  Year: {}\nLink: {}",
            self.department, self.name, self.semester, self.year, self.link
        )
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
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input = input.trim().to_string();
    println!("Reading through {} entries ...", parsed.len());
    let mut list = Vec::new();
    interpret_json(&parsed, &mut list, &input);
    println!("Found {} matches.\n", list.len());
    for paper in &list {
        println!("{}", paper);
        println!("--------------------------------");
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

fn interpret_json(parsed: &JsonValue, list: &mut Vec<Paper>, input: &str) {
    const CASE_INSENSITIVE: bool = true;
    for member in parsed.members() {
        for content in member.entries() {
            if content.0 == "Paper" {
                let val = match content.1.as_str() {
                    Some(s) => s,
                    None => "",
                };
                let mut matcher = sublime_fuzzy::FuzzySearch::new(input, val, CASE_INSENSITIVE);
                match matcher.best_match() {
                    Some(result) => {
                        if result.score() > 500 {
                            let mut paper = Paper::new();
                            for content in member.entries() {
                                let val = match content.1.as_str() {
                                    Some(s) => s,
                                    None => "",
                                };
                                match content.0 {
                                    "Department" => paper.department.push_str(val),
                                    "Link" => paper.link.push_str(val),
                                    "Paper" => paper.name.push_str(val),
                                    "Semester" => paper.semester.push_str(val),
                                    "Year" => paper.year.push_str(val),
                                    &_ => {}
                                }
                            }
                            list.push(paper);
                        }
                    }
                    None => {}
                };
                break;
            }
        }
    }
}
