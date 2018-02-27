use std::string::String;
use std::fmt::{Display, Formatter, Result};

pub struct Request {
    pub path: String, 
    pub kind: RequestKind,
    pub data: String,
    pub host: String,
}

pub enum RequestKind {
    Get,
    Post,
    Head,
    Put,
    Delete,
    Options,
}

impl RequestKind {
    fn from(string: &str) -> Option<RequestKind> { 
        match &*string { 
            "GET" => Some(RequestKind::Get),
            "POST" => Some(RequestKind::Post),
            "HEAD" => Some(RequestKind::Head),
            "PUT" => Some(RequestKind::Put),
            "DELETE" => Some(RequestKind::Delete),
            "OPTIONS" => Some(RequestKind::Options),
            _ => None
        }
    }
}

impl Request {
    pub fn from(request_data: String) -> Request {
        let (request_type, path, _http_version, host_name) = parse(&request_data);
        Request {
            path,
            data: request_data,
            kind: RequestKind::from(&request_type).unwrap(),
            host: host_name,
        }
    }
}

/// Parses the request as a string
/// Returns (HTTP Method, Route, HTTP Version)
fn parse(request_string: &str) -> (String, String, String, String) {
    let mut line_split = request_string.split('\n');
    let first_line: Vec<&str> = line_split.nth(0).unwrap().split(' ').collect();
    let second_line: Vec<&str> = line_split.nth(1).unwrap().split(' ').collect();
    (first_line[0].to_string(), first_line[1].to_string(), first_line[2].to_string(), second_line[1].to_string())
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "HTTP Request: {} - {}", self.kind, self.path)
    }
}
impl Display for RequestKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            RequestKind::Get => { write!(f, "{}", "GET") },
            RequestKind::Post => { write!(f, "{}", "POST") },
            RequestKind::Head => { write!(f, "{}", "HEAD") },
            RequestKind::Put => { write!(f, "{}", "PUT") },
            RequestKind::Delete => { write!(f, "{}", "DELETE") },
            RequestKind::Options => { write!(f, "{}", "OPTIONS") },
        }
    }
}
