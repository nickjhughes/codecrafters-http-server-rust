use std::{io::Write, net::TcpListener};

use response::Response;
use status_code::StatusCode;

mod method;
mod request;
mod response;
mod status_code;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let response = Response::from_status_code(StatusCode::OK);
                stream.write_all(&response.encode().unwrap()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
