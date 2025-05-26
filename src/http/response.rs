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
