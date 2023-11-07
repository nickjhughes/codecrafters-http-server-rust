use anyhow::Result;
use std::{
    io::{Read, Write},
    net::TcpListener,
};

use request::Request;
use response::Response;
use status_code::StatusCode;

mod method;
mod request;
mod response;
mod status_code;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = Vec::new();
                stream.read(&mut buf)?;
                println!("Parsing requset...");
                let request = Request::parse(&buf)?;
                let response = if request.target == "/" {
                    Response::from_status_code(StatusCode::OK)
                } else {
                    Response::from_status_code(StatusCode::NotFound)
                };
                stream.write_all(&response.encode()?)?;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
