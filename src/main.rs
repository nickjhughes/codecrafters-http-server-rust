use anyhow::Result;
use tokio::{
    io::{self, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use request::Request;
use response::Response;
use status_code::StatusCode;

mod method;
mod request;
mod response;
mod status_code;

async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buf = Vec::with_capacity(1024);

    loop {
        match stream.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(_) => {}
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                anyhow::bail!("failed to read from stream: {:?}", e)
            }
        }
    }

    let request = Request::parse(&buf)?;
    let response = if request.target == "/" {
        Response::from_status_code(StatusCode::OK)
    } else {
        Response::from_status_code(StatusCode::NotFound)
    };
    stream.write_all(&response.encode()?).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").await?;
    // eprintln!("Listening on {:?}", listener.local_addr()?);

    loop {
        let (stream, _) = listener.accept().await?;
        handle_connection(stream).await?;
    }
}
