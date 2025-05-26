use std::collections::HashMap;

use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpRequest {
    pub fn new() -> Self {
        HttpRequest {
            method: String::new(),
            path: String::new(),
            version: String::new(),
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub async fn try_from_reader<T>(mut reader: BufReader<T>) -> Result<Self, HttpRequestError>
    where
        T: AsyncRead + Unpin,
    {
        let mut request = HttpRequest::new();
        // request_line
        let mut request_line = String::new();
        reader
            .read_line(&mut request_line)
            .await
            .map_err(|_| HttpRequestError::InvalidRequestLine)?;
        // println!("request_line\n{:#?}", request_line);
        let words: Vec<&str> = request_line.split_whitespace().collect();
        if words.len() != 3 {
            return Err(HttpRequestError::InvalidRequestLine);
        }
        let (method, path, version) = (words[0], words[1], words[2]);
        request.method = method.to_string();
        request.path = path.to_string();
        request.version = version.to_string();
        // headers
        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .await
                .map_err(|_| HttpRequestError::InvalidHeader)?;
            line = line.trim().to_string();
            // println!("header: {:#?}", line);
            if line.is_empty() {
                break;
            }
            let words: Vec<&str> = line.split(": ").collect();
            if words.len() != 2 {
                return Err(HttpRequestError::InvalidHeader);
            }
            let (key, val) = (words[0], words[1]);
            request.headers.insert(key.to_string(), val.to_string());
        }
        // body
        if let Some(len) = request.headers.get("Content-Length") {
            let len = len
                .parse::<usize>()
                .map_err(|_| HttpRequestError::InvalidHeader)?;
            let mut body = vec![0u8; len];
            reader.read_exact(&mut body).await.ok();
            request.body = String::from_utf8(body).map_err(|_| HttpRequestError::InvalidBody)?;
        }
        Ok(request)
    }
}

#[derive(Debug)]
pub enum HttpRequestError {
    InvalidRequestLine,
    InvalidHeader,
    InvalidBody,
}
