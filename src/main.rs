use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use std::thread;

const TEMPLATE_FOLDER: &'static str = "templates";

const HTML_RESPONSE_HEADER: &'static [u8] =
    b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n";
const CSS_RESPONSE_HEADER: &'static [u8] =
    b"HTTP/1.1 200 OK\r\nContent-Type: text/css; charset=UTF-8\r\n\r\n";

fn read_static_file(file_path: &str) -> Vec<u8> {
    println!("Reading file: {}", file_path);
    let mut file = std::fs::File::open(file_path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    buffer
}

fn read_template_file(file_path: &str) -> Vec<u8> {
    let file_path_ = format!("{}/{}", TEMPLATE_FOLDER, file_path);
    let mut file = std::fs::File::open(file_path_).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    buffer
}

fn handle_client(mut stream: std::net::TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let url = request.split_whitespace().nth(1).unwrap();

    let (file_path, response_header) = if url.starts_with("/static/") {
        (url.trim_start_matches('/'), CSS_RESPONSE_HEADER)
    } else {
        let file_path = "index.html";
        (file_path, HTML_RESPONSE_HEADER)
    };

    let response_body = if url.starts_with("/static/") {
        read_static_file(file_path)
    } else {
        read_template_file(file_path)
    };

    stream.write(response_header).unwrap();
    stream.write(&response_body).unwrap();
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Both).unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Listening on port 8080");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| {
            handle_client(stream);
        });
    }
}
