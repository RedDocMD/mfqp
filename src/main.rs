use json;
use std::fs;
use std::io;
use std::process;
use std::{error::Error, path::PathBuf};
use termcolor::Color;

use mfqp;
use mfqp::Paper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let data_url = String::from("https://qp.metakgp.org/data/data.json");
    println!("Fetching JSON file from {} ...", data_url);

    let json_string = mfqp::get_json_string(&data_url)
        .await
        .unwrap_or_else(|_err| {
            mfqp::print_in_color("Failed to fetch JSON file", Color::Red);
            process::exit(1)
        });
    mfqp::print_in_color("Fetched JSON file.", Color::Green);

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

    mfqp::print_in_color("Do you want to download files? (y/N)", Color::Yellow);
    input = String::new();
    io::stdin().read_line(&mut input)?;
    input = input.trim().to_string();
    if input == String::from("y") {
        let mut download_directory = get_default_dir();
        mfqp::print_in_color(
            format!(
                "Enter the directory to download to? (default: {})",
                download_directory.to_str().unwrap()
            )
            .as_str(),
            Color::Yellow,
        );
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        download_directory.push(input);

        if !download_directory.exists() {
            fs::create_dir_all(&download_directory).unwrap_or_else(|err| {
                mfqp::print_in_color("Failed create download dir", Color::Red);
                println!("{}", err);
                process::exit(1);
            });
        }

        let mut handles = Vec::new();
        for paper in list {
            let download_directory = download_directory.clone();
            let handle = tokio::spawn(async move {
                download_paper(
                    paper.clone(),
                    download_directory.to_str().unwrap().to_string(),
                )
                .await;
            });
            handles.push(handle);
        }

        // This is to prevent Tokio runtime from exiting
        // before all the downloads are completed
        for handle in handles {
            handle.await.unwrap();
        }
    } else {
        mfqp::print_in_color("Do you want to list files? (Y/n)", Color::Yellow);
        input = String::new();
        io::stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        if input != String::from("n") {
            for paper in &list {
                println!("{}\n", paper);
            }
        }
    }
    Ok(())
}

async fn download_paper(paper: Paper, download_directory: String) {
    match mfqp::download_pdf(
        paper.link().to_string(),
        paper.filename(),
        download_directory,
    )
    .await
    {
        Ok(_) => {
            mfqp::print_in_color(
                format!("Downloaded {}", paper.filename()).as_str(),
                Color::Green,
            );
        }
        Err(e) => {
            mfqp::print_in_color(
                format!("Failed to download because: {}", e).as_str(),
                Color::Red,
            );
            println!("Link for manual download: {}", paper.link());
        }
    };
}

fn get_default_dir() -> PathBuf {
    if let Some(dir) = dirs::download_dir() {
        dir
    } else if let Some(dir) = &mut dirs::home_dir() {
        dir.push("Downloads");
        dir.clone()
    } else {
        PathBuf::from(".")
    }
}
