use csv::{Reader, Writer};
use std::{
    env,
    fs::{self, DirEntry},
    process,
};

use microslop_groups_extract::*;

fn main() {
    const ACCEPTED_FORMAT: [&str; 3] = ["json", "csv", "sqlite"];

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: convert <format> <format>");
        println!("Format can be on of [json, csv, sqlite]");
        process::exit(1);
    }

    let (from_format, into_format) = (args[1].clone(), args[2].clone());

    if !ACCEPTED_FORMAT
        .iter()
        .fold(false, |acc, x| acc || (from_format.as_str() == *x))
    {
        println!("From format invalid");
        process::exit(1);
    }
    if !ACCEPTED_FORMAT
        .iter()
        .fold(false, |acc, x| acc || (into_format.as_str() == *x))
    {
        println!("Into format invalid");
        process::exit(1);
    }

    // Prepare the input
    match fs::read_dir("data/") {
        Ok(_) => {}
        Err(_) => {
            let _ = fs::create_dir("data/");
        }
    };
    let valid_files: Vec<DirEntry> = fs::read_dir("data/")
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().unwrap().is_file())
        .filter(|entry| {
            entry
                .file_name()
                .into_string()
                .unwrap()
                .ends_with(from_format.as_str())
        })
        .collect();

    // Prepare the output
    match fs::read_dir("output/") {
        Ok(_) => {}
        Err(_) => {
            let _ = fs::create_dir("output/");
        }
    };

    for file_entry in &valid_files {
        let users: Vec<User> = match from_format.as_str() {
            "json" => {
                let from_file_content = fs::read_to_string(file_entry.path()).unwrap();
                serde_json::from_str(&from_file_content).unwrap()
            }
            "csv" => Reader::from_path(file_entry.path())
                .unwrap()
                .deserialize()
                .map(|entry| entry.unwrap())
                .collect(),
            _ => Vec::new(),
        };

        let full_file_name = file_entry.file_name().into_string().unwrap();
        let file_name = full_file_name
            .strip_suffix(from_format.as_str())
            .unwrap_or(&full_file_name);

        match into_format.as_str() {
            "json" => {
                fs::write(
                    format!("output/{}.json", file_name),
                    serde_json::to_string_pretty(&users).unwrap(),
                )
                .unwrap();
            }
            "csv" => {
                let mut writer = Writer::from_path(format!("output/{}.csv", file_name)).unwrap();
                for user in users {
                    writer.serialize(user).unwrap();
                }
            }
            _ => {}
        };
    }
}
