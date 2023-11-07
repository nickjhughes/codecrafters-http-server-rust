use anyhow::Result;
use std::collections::HashMap;

use crate::method::Method;

#[derive(Debug)]
pub struct Request<'request> {
    pub method: Method,
    pub target: &'request str,
    pub headers: HashMap<&'request str, &'request str>,
    pub body_len: usize,
    pub body: Option<&'request [u8]>,
}

impl<'request> Request<'request> {
    /// Try to parse a HTTP request header from the given slice.
    ///
    /// If the request is incomplete, return (input, None).
    /// If the request is complete, return the unparsed portion of the input, if any,
    /// and a `Request`, with `body_len` set to the length of the expected body.
    pub fn parse_header(input: &'request [u8]) -> Result<(&[u8], Option<Self>)> {
        let mut method = None;
        let mut target = None;
        let mut headers = HashMap::new();

        let mut rest = input;
        loop {
            let mut i = 0;
            while i < rest.len() && rest[i] != b'\r' {
                i += 1;
            }
            if i + 1 >= rest.len() {
                // Incomplete request
                return Ok((input, None));
            }
            assert_eq!(rest[i + 1], b'\n');
            i += 1;
            let line = std::str::from_utf8(&rest[0..i - 1])?;
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

        let body_len = headers
            .get("Content-Length")
            .map(|s| s.parse::<usize>().expect("invalid content-length header"))
            .unwrap_or(0);

        Ok((
            rest,
            Some(Request {
                method: method.unwrap(),
                target: target.unwrap(),
                headers,
                body_len,
                body: None,
            }),
        ))
    }

    pub fn set_body(&mut self, body: &'request [u8]) {
        assert_eq!(body.len(), self.body_len);
        self.body = Some(body);
    }
}

#[cfg(test)]
mod tests {
    use super::Request;
    use crate::method::Method;

    #[test]
    fn parse_no_headers_no_body() {
        let input = b"GET /index.html HTTP/1.1\r\n\r\n";
        let (rest, request) = Request::parse_header(input).unwrap();

        assert!(rest.is_empty());

        let request = request.unwrap();
        assert_eq!(request.method, Method::Get);
        assert_eq!(request.target, "/index.html");
        assert!(request.headers.is_empty());
        assert_eq!(request.body_len, 0);
        assert!(request.body.is_none());
    }

    #[test]
    fn parse_headers_no_body() {
        let input =
            b"GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\n\r\n";
        let (rest, request) = Request::parse_header(input).unwrap();

        assert!(rest.is_empty());

        let request = request.unwrap();
        assert_eq!(request.method, Method::Get);
        assert_eq!(request.target, "/index.html");
        assert_eq!(request.headers.len(), 2);
        assert_eq!(request.headers.get("Host"), Some(&"localhost:4221"));
        assert_eq!(request.headers.get("User-Agent"), Some(&"curl/7.64.1"));
        assert_eq!(request.body_len, 0);
        assert!(request.body.is_none());
    }

    #[test]
    fn parse_headers_body() {
        let input =
            b"GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\nContent-Length: 3\r\n\r\nfoo";
        let (rest, request) = Request::parse_header(input).unwrap();

        assert_eq!(rest, b"foo");

        let mut request = request.unwrap();
        assert_eq!(request.method, Method::Get);
        assert_eq!(request.target, "/index.html");
        assert_eq!(request.headers.len(), 3);
        assert_eq!(request.headers.get("Host"), Some(&"localhost:4221"));
        assert_eq!(request.headers.get("User-Agent"), Some(&"curl/7.64.1"));
        assert_eq!(request.headers.get("Content-Length"), Some(&"3"));
        assert_eq!(request.body_len, 3);
        assert!(request.body.is_none());

        request.set_body(rest);
        assert_eq!(request.body.unwrap(), b"foo");
    }

    #[test]
    fn parse_incomplete_header() {
        {
            let input = b"GET /index.html HTTP/1.1\r\nHost: localhos";
            let (rest, request) = Request::parse_header(input).unwrap();
            assert_eq!(rest, input);
            assert!(request.is_none());
        }

        {
            let input = b"GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\n";
            let (rest, request) = Request::parse_header(input).unwrap();
            assert_eq!(rest, input);
            assert!(request.is_none());
        }
    }
}
