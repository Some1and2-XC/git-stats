use std::{fs, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, path::{Path, PathBuf}};
use regex::Regex;
use anyhow::{Context, Result};

use crate::{cli::cli, OutputValue};

fn get_path(in_str: &Vec<String>) -> Result<String> {
    let re = Regex::new("GET (?<path>.+?) HTTP/1.1").unwrap();
    return Ok(
        re.captures(
            in_str
                .first()
                .with_context(|| "Expected first value")?
            )
            .with_context(|| "Expected regex match")?
            .name("path")
            .unwrap()
            .as_str().to_string());
}

enum OutputType {
    File(String),
    GetData,
}

pub fn handle_connection(mut stream: TcpStream, path: &str, args: &cli::CliArgs) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|res| res.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let out_path = get_path(&http_request).unwrap_or("/404".to_string());
    let output_value: OutputType;

    if out_path == "/api/data" {
        output_value = OutputType::GetData;
    } else if out_path == "/" {
        output_value = OutputType::File("/index.html".to_string());
    } else {
        output_value = OutputType::File(out_path);
    }

    let (status_line, contents) = match output_value {
        OutputType::GetData => {
            let data_content = super::get_data(args).unwrap();
            let mut flattened_content: Vec<OutputValue> = vec![];
            for value in data_content {
                for entry in value {
                    flattened_content.push(entry);
                }
            }
            let data_string = serde_json::to_string(&flattened_content).unwrap();
            ("HTTP/1.1 200 OK", data_string)
        },
        OutputType::File(filename) => {
            let file_path = format!("{}/{}", path, filename.trim_start_matches("/"));

            match fs::read_to_string(file_path) {
                Ok(v) => ("HTTP/1.1 200 OK", v),
                Err(_) => ("HTTP/1.1 404 OK", "404, not found!".to_string()),
            }
        },
    };
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();

    println!("Request: {:#?}", http_request);
}
