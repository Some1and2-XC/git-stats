use std::{fs, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, path::{Path, PathBuf}};
use regex::Regex;
use anyhow::{Context, Result};

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

pub fn handle_connection(mut stream: TcpStream, path: &str) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|res| res.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let mut out_path = get_path(&http_request).unwrap_or("/404".to_string());

    if out_path == "/" {
        out_path  = "/index.html".to_string();
    }

    let file_path = format!("{}/{}", path, out_path.trim_start_matches("/"));

    println!("{file_path:?}");

    let (status_line, contents) = match fs::read_to_string(file_path) {
        Ok(v) => ("HTTP/1.1 200 OK", v),
        Err(_) => ("HTTP/1.1 404 OK", "404, not found!".to_string()),
    };
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();

    println!("Request: {:#?}", http_request);
}
