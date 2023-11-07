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
            while rest[i] != b'\n' {
                i += 1;
            }
            let line = std::str::from_utf8(&rest[0..i])?;
            if line.is_empty() {
                rest = &rest[i + 1..];
                break;
            } else {
                rest = &rest[i + 1..];
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
