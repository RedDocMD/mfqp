use atty;
use curl::easy::Easy;
use json;
use json::JsonValue;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str;
use sublime_fuzzy;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct Paper {
    pub department: String,
    pub link: String,
    pub name: String,
    pub semester: String,
    pub year: String,
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

pub fn download_pdf(url: &str, filename: &str, directory: &str) -> Result<(), Box<dyn Error>> {
    let mut easy = Easy::new();
    easy.url(url).unwrap();
    let mut dst = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                dst.extend_from_slice(data);
                Ok(data.len())
            })?;
        transfer.perform()?;
    }

    let path = Path::new(directory).join(filename);
    let mut file = File::create(&path)?;
    file.write_all(&dst)?;

    Ok(())
}

pub fn get_json_string(url: &str) -> String {
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

pub fn interpret_json(parsed: &JsonValue, list: &mut Vec<Paper>, input: &str) {
    const CASE_INSENSITIVE: bool = true;
    const SCORE_THRESHOLD: isize = 500;

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
                        if result.score() > SCORE_THRESHOLD {
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

pub fn print_in_color(text: &str, color: Color) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_pdf_successful() {
        let directory = "/home/dm/Downloads";
        let filename = "ai.pdf";
        let link = "http://www.library.iitkgp.ac.in/pages/SemQuestionWiki/images/4/40/CS60045_Artificial_Intelligence_MA_2016.pdf";
        match download_pdf(link, filename, directory) {
            Ok(()) => println!("Successfully downloaded"),
            Err(e) => println!("Failed to download because: {}", e)
        };
    }

    #[test]
    fn test_download_pdf_wrong_link() {
        let directory = "/home/dm/Downloads";
        let filename = "ai.pdf";
        let link = "grabled nonsense";
        match download_pdf(link, filename, directory) {
            Ok(()) => println!("Successfully downloaded"),
            Err(e) => println!("Failed to download because: {}", e)
        };
    }

    #[test]
    fn test_download_pdf_non_existent_directory() {
        let directory = "/junk/dump";
        let filename = "ai.pdf";
        let link = "http://www.library.iitkgp.ac.in/pages/SemQuestionWiki/images/4/40/CS60045_Artificial_Intelligence_MA_2016.pdf";
        match download_pdf(link, filename, directory) {
            Ok(()) => println!("Successfully downloaded"),
            Err(e) => println!("Failed to download because: {}", e)
        };
    }
}
