use json;
use std::error::Error;
use std::io;
use std::process;
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

    let mut download_directory = String::from(".");
    mfqp::print_in_color("Do you want to download files? (y/N)", Color::Yellow);
    input = String::new();
    io::stdin().read_line(&mut input)?;
    input = input.trim().to_string();
    if input == String::from("y") {
        mfqp::print_in_color(
            format!(
                "Enter the directory to download to? (default: {})",
                download_directory
            )
            .as_str(),
            Color::Yellow,
        );
        download_directory = String::new();
        io::stdin().read_line(&mut download_directory)?;
        download_directory = download_directory.trim().to_string();
        for paper in list {
            let download_directory = download_directory.clone();
            // println!("Hell Ya!");
            tokio::spawn(async move {
                download_paper(paper.clone(), download_directory).await;
            });
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
            println!("{}", paper);
            println!("--------------------------------");
            mfqp::print_in_color(
                format!("Downloaded {}", paper.filename()).as_str(),
                Color::Green,
            );
            println!("--------------------------------");
        }
        Err(e) => {
            println!("{}", paper);
            println!("--------------------------------");
            mfqp::print_in_color(
                format!("Failed to download because: {}", e).as_str(),
                Color::Red,
            );
            println!("Link for manual download: {}", paper.link());
        }
    };
}
