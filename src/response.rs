extern crate chrono;

use std::string::String;
use std::fmt;

use self::chrono::Local;

/// HTTP Response
pub struct Response {
    kind: ResponseType,
    headers: Vec<String>,
    body: Option<String>,
    file: Option<Vec<u8>>,
}

/// The HTTP response status represented as an enum.
pub enum ResponseType {
    NotFound,
    HTTPOk,
    #[allow(dead_code)]
    Redirection,
    ServerError,
}

impl Response {

    /// Creates a new response where the body is text based.
    fn new_text(body: String, headers: Option<Vec<String>>, kind: ResponseType) -> Response {
        Response {
            body: Some(body),
            headers: headers.unwrap_or_default(),
            kind,
            file: None,
        }
    }

    /// Creates a new response where the body is a buffer of bytes.
    pub fn new_file(file: Vec<u8>, headers: Option<Vec<String>>, kind: ResponseType)-> Response {
        Response {
            body: None,
            headers: headers.unwrap_or_default(),
            kind,
            file: Some(file),
        }
    }

    #[allow(dead_code)]
    /// An HTTP 200 response with a text body.
    pub fn http_ok(body: String) -> Response {
        let mut response = Response::new_text(body, None, ResponseType::HTTPOk);
        response.default_headers();
        response
    }

    /// An HTTP 200 response with a file body.
    pub fn http_ok_file(file: Vec<u8>) -> Response {
        let mut response = Response::new_file(file, None, ResponseType::HTTPOk); 
        response.default_headers();
        response
    }

    /// An HTTP 404 response. The body is provided.
    pub fn not_found() -> Response {
        let body = r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>Page Not Found</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        * {
            line-height: 1.2;
            margin: 0;
        }
        html {
            color: #888;
            display: table;
            font-family: sans-serif;
            height: 100%;
            text-align: center;
            width: 100%;
        }
        body {
            display: table-cell;
            vertical-align: middle;
            margin: 2em auto;
        }
        h1 {
            color: #555;
            font-size: 2em;
            font-weight: 400;
        }
        p {
            margin: 0 auto;
            width: 280px;
        }
        @media only screen and (max-width: 280px) {
            body, p {
                width: 95%;
            }
            h1 {
                font-size: 1.5em;
                margin: 0 0 0.3em;
            }
        }
    </style>
</head>
<body>
    <h1>Page Not Found</h1>
    <p>Sorry, but the page you were trying to view does not exist.</p>
</body>
</html>
<!-- IE needs 512+ bytes: https://blogs.msdn.microsoft.com/ieinternals/2010/08/18/friendly-http-error-pages/ -->"#.to_string();
        let mut response = Response::new_text(body, None, ResponseType::NotFound);
        response.default_headers();
        response
    }

    /// An HTTP 500 response. The body is provided.
    pub fn server_error() -> Response {
        let mut r = Response {
            body: Some(String::from("<html><body><h1>rust-http-server: Internal server error</h1></body></html>")),
            headers: Vec::new(),
            kind: ResponseType::ServerError,
            file: None,
        };
        r.default_headers();
        r
    }

    /// Creates the default headers for every response.
    /// Status line, Date, Server name
    fn default_headers(&mut self) {
        let status = format!("{} {} {}", self.http_version(), self.code(), self.kind);
        let date = format!("Date: {}", Local::now().to_rfc2822());
        let server = format!("Server: {}", self.server());
        self.add_response_headers(&status);
        self.add_response_headers(&date);
        self.add_response_headers(&server);
    }

    pub fn add_response_headers(&mut self, header: &str) {
        self.headers.push(header.to_string());
    }

    /// Returns HTTP status code for ResponseType
    pub fn code(&self) -> u16 {
        match self.kind {
            ResponseType::HTTPOk => { 200 },
            ResponseType::Redirection => { 300 },
            ResponseType::NotFound => { 404 },
            ResponseType::ServerError => { 500 },
        }
    }

    /// Returns the HTTP version. (Currently 1.1)
    pub fn http_version(&self) -> &str {
        "HTTP/1.1"
    }
    /// Returns the webserver name
    pub fn server(&self) -> String {
        String::from(env!("CARGO_PKG_NAME"))
    }

    pub fn to_string(&self) -> String {
        let mut headers = self.headers_only();
        let body_copy = self.body.clone();
        headers.push_str(&body_copy.unwrap());
        headers
    }

    /// Returns a `String` of the response containing only the headers. The body can be immediately
    /// append to this.
    fn headers_only(&self) -> String {
        let mut result = self.headers.join("\r\n").to_string();
        result.push_str("\r\n\r\n");
        result
    }

    /// Returns a byte representation of the response
    pub fn to_bytes(&self) -> Vec<u8> {
        vprintln!("Converting response to binary");
        let string_rep = self.headers_only();
        let mut result = string_rep.into_bytes();
        let mut body = self.file.clone().unwrap();
        result.append(&mut body);
        result
    }

    #[allow(dead_code)]
    fn mime(&self) -> &str {
        // TODO: Implement mime type header
        unimplemented!();
    }

}

impl fmt::Display for ResponseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ResponseType::HTTPOk => write!(f, "{}", "OK"),
            ResponseType::Redirection => write!(f, "{}", "Redirection"),
            ResponseType::NotFound => write!(f, "{}", "Not Found"),
            ResponseType::ServerError => write!(f, "{}", "Internal Server Error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Ensures the response is a valid HTTP response
    #[test]
    fn test_response_structure() {
        let status_line = "HTTP/1.1 200 OK";
        let server_header = format!("Server: {}", env!("CARGO_PKG_NAME"));
        let body = "This is the body";

        let test_response = Response::http_ok(body.to_string());

        let result = test_response.to_string();
        
        let return_newline = result.matches("\r\n").count();

        assert_eq!(4, return_newline);

        let split_response: Vec<&str> = result.split("\r\n").collect();

        assert_eq!(status_line, split_response[0]);
        assert_eq!(server_header, split_response[2]);
        assert_eq!(body, split_response[split_response.len() - 1]);
    }
}
