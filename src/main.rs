use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

fn main() {
    println!("Hello, world!");
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| handle_connection(stream));
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let headers = extract_headers(&stream);

    println!("{:?}", headers)
}

fn extract_headers(stream: &TcpStream) -> Vec<String> {
    let mut buf_reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    let mut vec = Vec::new();
    let mut headers_complete = false;

    while !headers_complete {
        let lines = buf_reader.read_line(&mut line).unwrap();

        if line == "\r\n" || lines == 0 {
            headers_complete = true
        } else {
            vec.push(line.trim_end().to_string());
        }

        line.clear();
    }
    return vec;
}
