use atty;
use curl::easy::Easy;
use json;
use json::JsonValue;
use reqwest;
use std::fmt;
use std::io;
use std::io::Write;
use std::str;
use sublime_fuzzy;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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
        print_in_color("Do you want to download this? (y/N)", Color::Yellow);
        input = String::from("");
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        match input.trim() {
            "y" => {
                println!("Downloading...");
                download(&paper.name).expect("Error while downloading");
            }
            &_ => {}
        };
    }
}

#[tokio::main]
async fn download(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    match reqwest::get(path).await {
        Ok(resp) => match resp.text().await {
            Ok(text) => println!("RESPONSE: {} bytes from {}", text.len(), path),
            Err(_) => println!("ERROR reading {}", path),
        },
        Err(_) => println!("ERROR reading {}", path),
    }
    Ok(())
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

fn print_in_color(text: &str, color: Color) {
    let mut choice = ColorChoice::Never;
    if atty::is(atty::Stream::Stdout) {
        choice = ColorChoice::Auto;
    }
    let mut stdout = StandardStream::stdout(choice);
    stdout
        .set_color(ColorSpec::new().set_fg(Some(color)))
        .expect("Problem occurred");
    writeln!(&mut stdout, "{}", text).expect("Problem occurred");
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::White)))
        .expect("Problem occurred");
}
