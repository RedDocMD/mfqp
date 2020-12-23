use atty;
use json;
use json::JsonValue;
use regex::Regex;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str;
use sublime_fuzzy;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct Paper {
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

    pub fn link(self: &Self) -> &str {
        self.link.as_str()
    }

    pub fn filename(self: &Self) -> String {
        format!(
            "{}_{}_{}_{}.pdf",
            Self::replace_spaces_with_underscore(&self.name.trim()),
            Self::replace_spaces_with_underscore(&self.department.trim()),
            Self::replace_spaces_with_underscore(&self.semester.trim()),
            Self::replace_spaces_with_underscore(&self.year.trim())
        )
    }

    fn replace_spaces_with_underscore(string: &str) -> String {
        let re = Regex::new(r"\s+").unwrap();
        re.replace_all(string, "_").into_owned()
    }
}

impl fmt::Display for Paper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Department: {}  Name: {}\nSemester: {}  Year: {}",
            self.department, self.name, self.semester, self.year
        )
    }
}

pub fn download_pdf(url: &str, filename: &str, directory: &str) -> Result<usize, Box<dyn Error>> {
    let response = reqwest::blocking::get(url)?.error_for_status()?;
    if response.status().is_redirection() {
        download_pdf(
            response.headers()["Location"].to_str()?,
            filename,
            directory,
        )
    } else {
        let content = response.bytes()?;
        let size = content.len();
        let path: PathBuf = [directory, filename].iter().collect();
        let mut file = File::create(&path)?;
        file.write_all(&content)?;
        Ok(size)
    }
}

pub fn get_json_string(url: &str) -> reqwest::Result<String> {
    reqwest::blocking::get(url)?.text()
}

pub fn interpret_json(parsed: &JsonValue, list: &mut Vec<Paper>, input: &str) {
    const CASE_INSENSITIVE: bool = true;
    const SCORE_THRESHOLD: isize = 500;

    for member in parsed.members() {
        for content in member.entries() {
            if content.0 == "Paper" {
                let val = content.1.as_str().unwrap_or_default();
                let mut matcher = sublime_fuzzy::FuzzySearch::new(input, val, CASE_INSENSITIVE);
                match matcher.best_match() {
                    Some(result) => {
                        if result.score() > SCORE_THRESHOLD {
                            let mut paper = Paper::new();
                            for content in member.entries() {
                                let val = content.1.as_str().unwrap_or_default();
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

fn print_in_color_private(text: &str, color: Color) -> Result<(), Box<dyn Error>> {
    let mut choice = ColorChoice::Never;
    if atty::is(atty::Stream::Stdout) {
        choice = ColorChoice::Auto;
    }
    let mut stdout = StandardStream::stdout(choice);
    stdout.set_color(ColorSpec::new().set_fg(Some(color)))?;
    writeln!(&mut stdout, "{}", text)?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    Ok(())
}

pub fn print_in_color(text: &str, color: Color) {
    print_in_color_private(text, color).unwrap_or_else(|_err| println!("{}", text));
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
            Ok(_) => println!("Successfully downloaded"),
            Err(e) => println!("Failed to download because: {}", e),
        };
    }

    #[test]
    fn test_download_pdf_wrong_link() {
        let directory = "/home/dm/Downloads";
        let filename = "ai.pdf";
        let link = "grabled nonsense";
        match download_pdf(link, filename, directory) {
            Ok(_) => println!("Successfully downloaded"),
            Err(e) => println!("Failed to download because: {}", e),
        };
    }

    #[test]
    fn test_download_pdf_non_existent_directory() {
        let directory = "/junk/dump";
        let filename = "ai.pdf";
        let link = "http://www.library.iitkgp.ac.in/pages/SemQuestionWiki/images/4/40/CS60045_Artificial_Intelligence_MA_2016.pdf";
        match download_pdf(link, filename, directory) {
            Ok(_) => println!("Successfully downloaded"),
            Err(e) => println!("Failed to download because: {}", e),
        };
    }
}
