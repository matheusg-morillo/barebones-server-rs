use core::result::Result;
use core::result::Result::Ok;
use std::collections::HashMap;
use std::fs;
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
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    headers: HashMap<String, String>,
}

struct Response {
    status: u16,
    body: String,
}

const SERVER_NAME: &str = "matheus-server/0.1.0";
const CRLF: &str = "\r\n";

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

    let response = handle_request(&request).unwrap();

    let response = format!(
        "HTTP/1.1 {} {}{}\
                            Date: {}{}\
                            Server: {}{}\
                            Content-Type: {}{}\
                            Content-Length: {}{}\
                            Cache-Control: {}{}\
                            Connection: {}{}\
                            {}\
                            {}",
        response.status,
        reason_phrase(response.status),
        CRLF,
        get_date().unwrap(),
        CRLF,
        SERVER_NAME,
        CRLF,
        "text/html;charset=utf-8",
        CRLF,
        response.body.len(),
        CRLF,
        "no-store",
        CRLF,
        "keep-alive",
        CRLF,
        CRLF,
        response.body,
    );

    stream.write_all(response.as_bytes()).unwrap();
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

fn handle_request(request: &Request) -> Result<Response, std::io::Error> {
    return match (request.method.as_str(), request.target.as_str()) {
        ("GET", "/") => {
            let index_page = get_index()?;
            Ok(Response {
                status: 200,
                body: index_page,
            })
        }

        _ => Ok(Response {
            status: 404,
            body: String::new(),
        }),
    };
}

fn get_index() -> Result<String, std::io::Error> {
    return fs::read_to_string("resources/index.html");
}

fn get_date() -> Result<String, time::error::Format> {
    let now = OffsetDateTime::now_utc();
    let format = format_description!(
        "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] GMT"
    );
    return now.format(&format);
}

fn reason_phrase(status_code: u16) -> &'static str {
    match status_code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "", // HTTP/1.1 allows an empty reason phrase after the space
    }
}
