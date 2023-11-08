use anyhow::Result;
use std::{collections::HashMap, io::Write};

use crate::status_code::StatusCode;

#[derive(Debug)]
pub struct Response {
    pub status_code: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

impl Response {
    pub fn from_status_code(status_code: StatusCode) -> Self {
        Response {
            status_code,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn from_body(body: Vec<u8>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".into(), "text/plain".into());
        headers.insert("Content-Length".into(), body.len().to_string());

        Response {
            status_code: StatusCode::OK,
            headers,
            body: Some(body),
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut output = Vec::new();

        let status_code_u16: u16 = self.status_code.into();
        let status_line = format!(
            "HTTP/1.1 {} {}\r\n",
            status_code_u16,
            self.status_code.text()
        );

        output.write_all(status_line.as_bytes())?;
        for (key, value) in self.headers.iter() {
            output.write_all(format!("{}: {}\r\n", key, value).as_bytes())?;
        }
        output.write_all(b"\r\n")?;
        if let Some(body) = self.body.as_ref() {
            output.write_all(body)?;
        }

        Ok(output)
    }
}
