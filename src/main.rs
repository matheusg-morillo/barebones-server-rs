use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str::Split;
use std::thread;

struct Request {
    method: String,
    target: String,
    version: String,
    headers: HashMap<String, String>,
}

fn main() {
    println!("Hello, world!");
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| handle_connection(stream));
    }
}

fn handle_connection(stream: TcpStream) {
    let request = read_request(&stream);
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
