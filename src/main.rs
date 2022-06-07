use std::fs;
use std::io;
use std::process;
use std::sync::mpsc::{channel, Sender};
use std::{error::Error, path::PathBuf};
use termcolor::Color;

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
    if &input == "y" {
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

        let (tx, rx) = channel();
        let mut handles = Vec::new();
        for paper in list {
            let download_directory = download_directory.clone();
            let tx = tx.clone();
            let handle = tokio::spawn(async move {
                download_paper(
                    paper.clone(),
                    download_directory.to_str().unwrap().to_string(),
                    tx,
                )
                .await;
            });
            handles.push(handle);
        }

        let printer = tokio::spawn(async move {
            while let Ok(message) = rx.recv() {
                match message.color {
                    Color::White => println!("{}", message.message),
                    _ => mfqp::print_in_color(&message.message, message.color),
                }
            }
        });

        // This is to prevent Tokio runtime from exiting
        // before all the downloads are completed
        for handle in handles {
            handle.await.unwrap();
        }
        drop(tx);
        printer.await.unwrap();
    } else {
        mfqp::print_in_color("Do you want to list files? (Y/n)", Color::Yellow);
        input = String::new();
        io::stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        if &input != "n" {
            for paper in &list {
                println!("{}\n", paper);
            }
        }
    }
    Ok(())
}

struct Message {
    message: String,
    color: Color,
}

async fn download_paper(paper: Paper, download_directory: String, tx: Sender<Message>) {
    match mfqp::download_pdf(
        paper.link().to_string(),
        paper.filename(),
        download_directory,
    )
    .await
    {
        Ok(_) => {
            tx.send(Message {
                message: format!("Downloaded {}", paper.filename()),
                color: Color::Green,
            })
            .unwrap();
        }
        Err(e) => {
            tx.send(Message {
                message: format!("Failed to download {} because: {}", paper.filename(), e),
                color: Color::Red,
            })
            .unwrap();
            tx.send(Message {
                message: format!("Link for manual download: {}", paper.link()),
                color: Color::White,
            })
            .unwrap();
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
