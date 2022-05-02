use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn main() {
    // panic if cant bind
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // a single stream represents an open connection between the client and server.
    // a connection = full request and response process.
    // so process each connection in turn and produce a series of streams for us to handle
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

// the b transforms into a byte string so we can compare it with the buffer
const GET: &[u8; 16] = b"GET / HTTP/1.1\r\n";

// stream has to be mutable because internal state for it might change
fn handle_connection(mut stream: TcpStream) {
    // 1024 bytes in size, big enough to hold a basic request.
    // bytes are pretty much universally used as chars
    let mut buffer = [0; 1024];
    // read bytes from stream and put them into buffer
    stream.read(&mut buffer).unwrap();


    let (status_line, filename) = if buffer.starts_with(GET) {
        ("HTTP/1.1 200 OK", "views/index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "views/404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    // produce a string from &[u8]
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}
