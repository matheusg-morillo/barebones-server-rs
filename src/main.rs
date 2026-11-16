use core::result::Result;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str::Split;
use std::thread;
use time::OffsetDateTime;
use time::macros::format_description;

#[derive(Debug)]
struct Request {
    method: String,
    target: String,
    version: String,
    headers: HashMap<String, String>,
}

const SERVER_NAME: &str = "custom/0.1.0";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        // TODO: Implement thread pool
        thread::spawn(move || handle_connection(&mut stream));
    }
}

fn handle_connection(stream: &mut TcpStream) {
    println!("Handling connection");

    let request = read_request(&stream).unwrap();
    println!("{:#?}", request);

    let _ = handle_request(stream, &request);
}

fn read_request(stream: &TcpStream) -> Result<Request, &'static str> {
    let mut buf_reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    let mut headers_complete = false;
    let mut headers: HashMap<String, String> = HashMap::new();

    let bytes = buf_reader.read_line(&mut line).unwrap();

    if bytes == 0 {
        return Err("Empty request");
    }

    let (method, target, version) = {
        let mut request: Split<&str> = line.split(" ");
        match (
            request.next(),
            request.next(),
            request.next(),
            request.next(),
        ) {
            (Some(m), Some(t), Some(v), None) => {
                Ok((m.to_string(), t.to_string(), v.trim().to_string()))
            }
            _ => Err("malformed request"),
        }
    }?;

    line.clear();

    while !headers_complete {
        let lines = buf_reader.read_line(&mut line).unwrap();

        if line == "\r\n" || lines == 0 {
            headers_complete = true
        } else {
            line = line.trim_end().to_string();
            let header_option = line.split_once(":");
            if let Some((k, v)) = header_option {
                headers.insert(k.trim().to_string(), v.trim().to_string());
            }
        }

        line.clear();
    }
    return Ok(Request {
        method: method.to_string(),
        target: target.to_string(),
        version: version.to_string(),
        headers: headers,
    });
}

fn handle_request(stream: &mut TcpStream, request: &Request) -> Result<(), std::io::Error> {
    let mut response_success = "HTTP/1.1 200 OK\r\n".to_string();
    let mut response_fail = "HTTP/1.1 404 OK\r\n\r\n".to_string();

    add_headers(&mut response_success);

    println!("Response: {}", response_success);

    match (
        request.method.as_str(),
        request.target.as_str(),
        request.version.as_str(),
    ) {
        ("GET", "/", "HTTP/1.1") => stream.write_all(response_success.as_bytes()),
        _ => stream.write_all(response_fail.as_bytes()),
    }
}

fn add_headers(response: &mut String) {
    let date = get_date().unwrap();

    response.push_str(&format!("Date: {}\r\n", date));
    response.push_str(&format!("Server: {}\r\n", SERVER_NAME));
    // TODO: implement content-type negotiation
    response.push_str(&format!("Content-Type: {}\r\n", "text/plain"));
    // TODO: implement content-length calculation
    response.push_str(&format!("Content-Length: {}\r\n", 0));
    // TODO: get this from static config
    response.push_str(&format!("Cache-Control: {}\r\n", "no-store"));
    response.push_str("\r\n");
}

fn get_date() -> Result<String, time::error::Format> {
    let now = OffsetDateTime::now_utc();
    let format = format_description!(
        "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] GMT"
    );
    return now.format(&format);
}
