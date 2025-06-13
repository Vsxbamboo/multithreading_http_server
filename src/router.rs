use crate::{
    config,
    http::{HttpRequest, HttpResponse},
};
use percent_encoding::NON_ALPHANUMERIC;
use percent_encoding::percent_encode;
use std::cmp::Ordering;
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::process::Command;
use tracing::{error, info, warn};

pub async fn router_request(req: &HttpRequest) -> HttpResponse {
    // 1. 验证请求方法
    if !is_valid_method(req) {
        warn!("Invalid method: {}", req.method);
        return HttpResponse::not_implemented();
    }

    // 2. 获取配置并构建安全路径
    let (public_path, full_path) = match prepare_path(req).await {
        Ok((p, fp)) => (p, fp),
        Err(response) => {
            error!("Failed to prepare path for request: {}", req.path);
            return response;
        }
    };

    // 3. 验证路径安全性
    if !is_path_safe(&public_path, &full_path) {
        warn!("Unsafe path access attempt: {}", req.path);
        return HttpResponse::not_found();
    }

    // 4. 根据路径类型处理请求
    handle_request_by_path_type(&full_path, req).await
}

// 辅助函数
async fn prepare_path(req: &HttpRequest) -> Result<(PathBuf, PathBuf), HttpResponse> {
    let static_dir = config::read_config()
        .await
        .map_err(|_| HttpResponse::internal_server_error())?
        .static_dir;
    let public_path = Path::new(&static_dir).to_path_buf();

    let mut full_path = PathBuf::from(req.path.trim_start_matches("/"));
    if let Ok(p) = full_path.canonicalize() {
        full_path = p;
    } else {
        return Err(HttpResponse::not_found());
    }

    Ok((public_path, full_path))
}

fn is_valid_method(req: &HttpRequest) -> bool {
    req.method == "GET"
}

fn is_path_safe(public_path: &Path, full_path: &Path) -> bool {
    // full_path.starts_with(env::current_dir().unwrap().join(public_path))
    match env::current_dir() {
        Ok(current_dir) => full_path.starts_with(current_dir.join(public_path)),
        Err(_) => {
            eprintln!("Get current_dir error");
            false
        }
    }
}

async fn handle_request_by_path_type(path: &Path, req: &HttpRequest) -> HttpResponse {
    if path.is_dir() {
        handle_directory_request(path).await
    } else if path.is_file() {
        handle_file_request(path, req).await
    } else {
        HttpResponse::bad_request()
    }
}

async fn handle_directory_request(path: &Path) -> HttpResponse {
    list_directory(path)
        .await
        .map(|content| HttpResponse::ok().body("html", content))
        .unwrap_or_else(|_| HttpResponse::internal_server_error())
}

pub async fn list_directory(path: &Path) -> tokio::io::Result<String> {
    let mut entries = fs::read_dir(path).await?;
    let current_dir = env::current_dir().map_err(|e| {
        eprintln!("Failed to get current directory: {}", e);
        tokio::io::Error::new(tokio::io::ErrorKind::Other, "Failed to get current directory")
    })?;
    
    let current_path = path.strip_prefix(current_dir).map_err(|e| {
        eprintln!("Failed to get relative path: {}", e);
        tokio::io::Error::new(tokio::io::ErrorKind::InvalidInput, "Failed to get relative path")
    })?.display().to_string();

    // 构建简单 HTML
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html>\n");
    html.push_str("<meta charset=\"UTF-8\">");
    html.push_str("<head><title>Directory listing for /");
    html.push_str(&current_path);
    html.push_str("</title></head>\n");
    html.push_str("<body>\n");
    html.push_str("<h1>Directory listing for /");
    html.push_str(&current_path);
    html.push_str("</h1>\n");
    html.push_str("<hr>\n");
    html.push_str("<pre>\n");

    // 添加返回上一级链接
    if current_path != "" {
        let parent_path = Path::new(&current_path)
            .parent()
            .map(|p| p.to_str().unwrap_or(""))
            .unwrap_or("");
        let encoded_parent = percent_encode(parent_path.as_bytes(), NON_ALPHANUMERIC);
        html.push_str(&format!(
            "<a href=\"/{}\">[Parent Directory]</a>\n",
            encoded_parent
        ));
    }

    // 收集所有条目并分类
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let file_type = entry.file_type().await?;
        let file_path = format!("{}/{}", current_path, file_name);

        if file_type.is_dir() {
            dirs.push((file_name, file_path));
        } else if file_type.is_file() {
            files.push((file_name, file_path));
        }
    }

    // 对目录按名称排序
    dirs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    // 对文件先按后缀排序，再按名称排序
    files.sort_by(|a, b| {
        let ext_a = Path::new(&a.0)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let ext_b = Path::new(&b.0)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext_a.to_lowercase().cmp(&ext_b.to_lowercase()) {
            Ordering::Equal => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
            other => other,
        }
    });

    // 先输出目录
    for (file_name, file_path) in dirs {
        // let encoded_path = percent_encode(file_path.as_bytes(), NON_ALPHANUMERIC);
        let encoded_path: String = Path::new(&file_path)
            .components()
            .map(|comp| {
                let part = comp.as_os_str().to_string_lossy();
                percent_encode(part.as_bytes(), NON_ALPHANUMERIC).to_string()
            })
            .collect::<Vec<_>>()
            .join("/");
        html.push_str(&format!(
            "<a href=\"/{}\">[DIR] {}/</a>\n",
            encoded_path, file_name
        ));
    }

    // 再输出文件
    for (file_name, file_path) in files {
        // let encoded_path = percent_encode(file_path.as_bytes(), NON_ALPHANUMERIC);
        let encoded_path: String = Path::new(&file_path)
            .components()
            .map(|comp| {
                let part = comp.as_os_str().to_string_lossy();
                percent_encode(part.as_bytes(), NON_ALPHANUMERIC).to_string()
            })
            .collect::<Vec<_>>()
            .join("/");
        html.push_str(&format!(
            "<a href=\"/{}\">{}</a>\n",
            encoded_path, file_name
        ));
    }

    html.push_str("</pre>\n");
    html.push_str("<hr>\n");
    html.push_str("</body>\n");
    html.push_str("</html>\n");

    Ok(html)
}

async fn handle_file_request(path: &Path, req: &HttpRequest) -> HttpResponse {
    if req.path.ends_with(".cgi") {
        handle_cgi_request(path).await
    } else {
        handle_regular_file_request(path).await
    }
}

async fn handle_cgi_request(path: &Path) -> HttpResponse {
    info!("Executing CGI script: {}", path.display());
    Command::new(path)
        .output()
        .await
        .map(|output| {
            info!("CGI script executed successfully");
            HttpResponse::ok().body("text/html", output.stdout)
        })
        .unwrap_or_else(|e| {
            error!("Failed to execute CGI script: {}", e);
            HttpResponse::internal_server_error()
        })
}

async fn handle_regular_file_request(path: &Path) -> HttpResponse {
    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    info!("Serving file: {} (mime type: {})", path.display(), mime_type);
    match tokio::fs::read(path).await {
        Ok(data) => {
            if mime_type.starts_with("text/") {
                String::from_utf8(data)
                    .map(|text| HttpResponse::ok().body(&mime_type, text))
                    .unwrap_or_else(|e| {
                        error!("Failed to decode text file: {}", e);
                        HttpResponse::internal_server_error()
                    })
            } else {
                HttpResponse::ok().body(&mime_type, data)
            }
        }
        Err(e) => {
            error!("Failed to read file: {}", e);
            HttpResponse::internal_server_error()
        }
    }
}
