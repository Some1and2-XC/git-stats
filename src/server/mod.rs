use std::{fs, io::{self, BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}, path::{Path, PathBuf}};
use regex::Regex;
use anyhow::{anyhow, Context, Result};
use httparse;

use crate::{cli::cli, OutputValue};

fn get_path(stream: &mut TcpStream) -> Result<String> {

    // Gets IP addr
    let ip = match stream.peer_addr() {
        Ok(v) => Some(v),
        Err(_) => None,
    };

    log::info!("Processing getting request from '{ip:?}'");

    // Creates reader for stream
    let mut reader = BufReader::new(stream);

    // fills buffer
    let mut buf: Vec<u8> = Vec::new();
    let _buf_len = reader.read_until(b'\r', &mut buf).unwrap();

    // Sets header stuff
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);

    // Parses headers
    let _ = req.parse(&buf).unwrap();

    // Tries to resolve header
    match req.path {
        Some(path) => {
            log::info!("Parsed path: '{path}' from request.");

            return Ok(path.to_string());
        },
        None => {
            return Err(anyhow!("Path not found in request body! Request: '{:?}'.", buf));
        }
    }
}

/// Function for ensuring that a given path is inside of the configured server directory
fn sanitize_path(dir: &str, args: &cli::CliArgs) -> Result<()> {
    let canonical_target = fs::canonicalize(dir).map_err(anyhow::Error::from)?;
    let canonical_src = fs::canonicalize(&args.server_directory).map_err(anyhow::Error::from)?;

    if canonical_target.starts_with(canonical_src) {
        return Ok(());
    } else {
        return Err(anyhow!("Failed to validate path: '{canonical_target:?}'"));
    }
}

enum OutputType {
    File(String),
    GetData,
}

pub fn handle_connection(mut stream: TcpStream, path: &str, args: &cli::CliArgs) {

    let out_path = get_path(&mut stream).unwrap_or("/404".to_string());
    let output_value: OutputType;

    if out_path == args.server_uri {
        output_value = OutputType::GetData;
    } else if out_path == "/" {
        output_value = OutputType::File("/index.html".to_string());
    } else {
        output_value = OutputType::File(out_path.clone());
    }

    let (status_line, contents) = match output_value {
        OutputType::GetData => {
            match super::get_data(args) {
                Ok(data_content) => {
                    let mut flattened_content: Vec<OutputValue> = vec![];
                    for value in data_content {
                        for entry in value {
                            flattened_content.push(entry);
                        }
                    }
                    let data_string = serde_json::to_string(&flattened_content).unwrap();
                    ("HTTP/1.1 200 OK", data_string)
                },
                Err(v) => {
                    ("HTTP/1.1 500 INTERNAL SERVER ERROR", format!("{v}"))
                },
            }
        },
        OutputType::File(filename) => {

            let file_path = format!("{}/{}", path, filename.trim_start_matches("/"));
            let cleaned_path = sanitize_path(&file_path, args);

            if let Err(_) = cleaned_path {
                ("HTTP/1.1 403 FORBIDDEN", "403, forbidden & directory traversal is bad!".to_string())
            } else {
                match fs::read_to_string(file_path) {
                    Ok(v) => ("HTTP/1.1 200 OK", v),
                    Err(_) => ("HTTP/1.1 404 NOT FOUND", "404, not found!".to_string()),
                }
            }
        },
    };
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
