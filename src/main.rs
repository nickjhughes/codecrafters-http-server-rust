use anyhow::Result;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use request::Request;
use response::Response;
use status_code::StatusCode;

mod method;
mod request;
mod response;
mod status_code;

fn generate_response<'a>(request: &'a Request) -> Response<'a> {
    if request.target == "/" {
        Response::from_status_code(StatusCode::OK)
    } else if request.target.starts_with("/echo/") {
        let random_string = request.target.trim_start_matches("/echo/");
        Response::from_body(random_string.as_bytes())
    } else if request.target == "/user-agent" {
        Response::from_body(request.headers.get("User-Agent").unwrap().as_bytes())
    } else {
        Response::from_status_code(StatusCode::NotFound)
    }
}

async fn process(mut stream: TcpStream) {
    eprintln!("{:?}: New connection", stream.peer_addr().unwrap());

    let mut buf = Vec::with_capacity(1024);

    loop {
        match stream.try_read_buf(&mut buf) {
            Ok(0) => {
                eprintln!("{:?}: Read 0 bytes", stream.peer_addr().unwrap());
                break;
            }
            Ok(n) => {
                eprintln!("{:?}: Read {n} bytes", stream.peer_addr().unwrap());

                let (rest, request) = Request::parse_header(&buf).unwrap();
                if let Some(mut request) = request {
                    eprintln!("{:?}: Got complete request", stream.peer_addr().unwrap());

                    // Got complete request header, now read body
                    let mut body = rest.to_vec();
                    if request.body_len > 0 {
                        body.resize(request.body_len, 0);
                        stream.read_exact(&mut body[rest.len()..]).await.unwrap();
                        request.set_body(&body);
                    }

                    // And reply with response
                    let response = generate_response(&request);
                    eprintln!(
                        "{:?}: Replying with {:?}",
                        response,
                        stream.peer_addr().unwrap()
                    );
                    stream.write_all(&response.encode().unwrap()).await.unwrap();
                    // stream.shutdown().await?;
                    // eprintln!("{:?}: Shut down connection", stream.peer_addr().unwrap());

                    break;
                } else {
                    // Incomplete request
                    continue;
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                panic!("failed to read from stream: {:?}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").await?;
    // eprintln!("Listening on {:?}", listener.local_addr()?);

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            process(stream).await;
        });
    }
}
