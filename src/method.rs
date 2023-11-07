use anyhow::Result;

#[derive(Debug, PartialEq)]
pub enum Method {
    Connect,
    Delete,
    Get,
    Head,
    Options,
    Patch,
    Post,
    Put,
    Trace,
}

impl Method {
    pub fn from_str(input: &str) -> Result<Self> {
        match input {
            "CONNECT" => Ok(Method::Connect),
            "DELETE" => Ok(Method::Delete),
            "GET" => Ok(Method::Get),
            "HEAD" => Ok(Method::Head),
            "OPTIONS" => Ok(Method::Options),
            "PATCH" => Ok(Method::Patch),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "TRACE" => Ok(Method::Trace),
            _ => Err(anyhow::format_err!("invalid request method")),
        }
    }
}
