use anyhow::Result;
use std::collections::HashMap;

use crate::method::Method;

#[derive(Debug)]
pub struct Request<'request> {
    pub method: Method,
    pub target: &'request str,
    pub headers: HashMap<&'request str, &'request str>,
    pub body: Option<&'request [u8]>,
}

impl<'request> Request<'request> {
    pub fn parse(input: &'request [u8]) -> Result<Self> {
        let mut method = None;
        let mut target = None;
        let mut headers = HashMap::new();

        let mut rest = input;
        loop {
            let mut i = 0;
            while rest[i] != b'\r' {
                i += 1;
            }
            assert_eq!(rest[i + 1], b'\n');
            i += 1;
            let line = std::str::from_utf8(&rest[0..i - 1])?;
            println!("Line: {:?}", line);
            if line.is_empty() {
                rest = &rest[i + 1..];
                break;
            } else {
                if method.is_none() {
                    // Start line
                    let (method_, remainder) = line.split_once(' ').unwrap();
                    let (target_, version) = remainder.split_once(' ').unwrap();
                    assert_eq!(version, "HTTP/1.1");
                    method = Some(Method::from_str(method_)?);
                    target = Some(target_);
                } else {
                    // Header
                    let (key, value) = line.split_once(": ").unwrap();
                    headers.insert(key, value);
                }

                rest = &rest[i + 1..];
            }
        }

        let body = if !rest.is_empty() { Some(rest) } else { None };

        Ok(Request {
            method: method.unwrap(),
            target: target.unwrap(),
            headers,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Request;
    use crate::method::Method;

    #[test]
    fn parse_no_headers_no_body() {
        let input = b"GET /index.html HTTP/1.1\r\n\r\n";
        let request = Request::parse(input).unwrap();

        assert_eq!(request.method, Method::Get);
        assert_eq!(request.target, "/index.html");
        assert!(request.headers.is_empty());
        assert!(request.body.is_none());
    }

    #[test]
    fn parse_headers_no_body() {
        let input =
            b"GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\n\r\n";
        let request = Request::parse(input).unwrap();

        assert_eq!(request.method, Method::Get);
        assert_eq!(request.target, "/index.html");
        assert_eq!(request.headers.len(), 2);
        assert_eq!(request.headers.get("Host"), Some(&"localhost:4221"));
        assert_eq!(request.headers.get("User-Agent"), Some(&"curl/7.64.1"));
        assert!(request.body.is_none());
    }

    #[test]
    fn parse_no_headers_body() {
        let input = b"GET /index.html HTTP/1.1\r\n\r\nfoo";
        let request = Request::parse(input).unwrap();

        assert_eq!(request.method, Method::Get);
        assert_eq!(request.target, "/index.html");
        assert!(request.headers.is_empty());
        assert_eq!(request.body.unwrap(), b"foo");
    }

    #[test]
    fn parse_headers_body() {
        let input =
            b"GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\n\r\nfoo";
        let request = Request::parse(input).unwrap();

        assert_eq!(request.method, Method::Get);
        assert_eq!(request.target, "/index.html");
        assert_eq!(request.headers.len(), 2);
        assert_eq!(request.headers.get("Host"), Some(&"localhost:4221"));
        assert_eq!(request.headers.get("User-Agent"), Some(&"curl/7.64.1"));
        assert_eq!(request.body.unwrap(), b"foo");
    }
}
