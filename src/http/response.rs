use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpResponse {
    pub version: String,
    pub code: String,
    pub reason: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new() -> Self {
        HttpResponse {
            version: String::new(),
            code: String::new(),
            reason: String::new(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }
    #[allow(dead_code)]
    pub fn with_status(
        code: &str,
        reason: &str,
        content_type: &str,
        body: impl Into<Vec<u8>>,
    ) -> Self {
        let mut resp = Self::new();
        resp.version = "HTTP/1.1".to_string();
        resp.code = code.to_string();
        resp.reason = reason.to_string();
        resp.body = body.into();
        if !resp.body.is_empty() {
            resp.headers
                .insert("Content-Type".to_string(), content_type.to_string());
            resp.headers
                .insert("Content-Length".to_string(), resp.body.len().to_string());
        }
        resp.headers
            .insert("Connection".to_string(), "close".to_string());
        resp
    }
    pub fn gen_resp_bytes(&self) -> Vec<u8> {
        let mut response = format!("{} {} {}\r\n", self.version, self.code, self.reason);
        for (key, val) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, val));
        }
        response.push_str("\r\n");
        let mut response = response.into_bytes();
        response.extend(&self.body);
        response
    }
}

impl HttpResponse {
    /// 200 OK
    pub fn ok() -> Self {
        Self::new_with_status("200", "OK")
    }

    /// 400 Bad Request
    pub fn bad_request() -> Self {
        Self::new_with_status("400", "Bad Request")
    }

    /// 404 Not Found
    pub fn not_found() -> Self {
        Self::new_with_status("404", "Not Found")
    }

    /// 500 Internal Server Error
    pub fn internal_server_error() -> Self {
        Self::new_with_status("500", "Internal Server Error")
    }
    
    /// 501 Not Implemented
    pub fn not_implemented() -> Self {
        Self::new_with_status("501", "Not Implemented")
    }

    fn new_with_status(code: &str, reason: &str) -> Self {
        let mut resp = Self::new();
        resp.version = "HTTP/1.1".to_string();
        resp.code = code.to_string();
        resp.reason = reason.to_string();
        resp.headers
            .insert("Connection".to_string(), "close".to_string());
        resp
    }

    /// 设置响应体，自动设置 Content-Type 和 Content-Length
    pub fn body(mut self, content_type: &str, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self.headers
            .insert("Content-Type".to_string(), content_type.to_string());
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        self
    }
}

