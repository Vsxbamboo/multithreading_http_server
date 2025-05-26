use std::env;
use std::path::Path;
use std::path::PathBuf;

use tokio::process::Command;

use mime_guess;

use crate::http::HttpRequest;
use crate::http::HttpResponse;

use crate::utils;

pub async fn router_request(req: &HttpRequest) -> HttpResponse {
    if req.method != "GET" {
        return HttpResponse::with_status("501", "Not Implemented", "", "");
    }
    let public_path = Path::new("public");
    let mut full_path = PathBuf::from(req.path.trim_start_matches("/"));
    if let Ok(p) = full_path.canonicalize() {
        full_path = p;
    } else {
        return HttpResponse::with_status("404", "Not Found", "", "");
    }
    if !full_path.starts_with(env::current_dir().unwrap().join(public_path)) {
        return HttpResponse::with_status("404", "Not Found", "", "");
    }
    // println!("{}", full_path.display());
    if full_path.is_dir() {
        if let Ok(dir_content) = utils::list_directory(&full_path).await {
            return HttpResponse::with_status("200", "OK", "text/plain; charset=utf8", dir_content);
        } else {
            return HttpResponse::with_status("500", "Server Internal Error", "", "");
        }
    } else if full_path.is_file() {
        // cgi
        if req.path.ends_with(".cgi") {
            // println!("is a cgi");
            let output = Command::new(full_path.to_str().unwrap())
                .output()
                .await
                .unwrap();
            println!("{}", { String::from_utf8_lossy(&output.stdout) });
            return HttpResponse::with_status(
                "200",
                "OK",
                "text/plain; charset=utf8",
                output.stdout,
            );
        }

        // file
        // println!("is a file");
        let mime_type = mime_guess::from_path(&full_path)
            .first_or_octet_stream()
            .to_string();
        return match tokio::fs::read(&full_path).await {
            Ok(data) => {
                if mime_type.starts_with("text/") {
                    match String::from_utf8(data) {
                        Ok(text) => HttpResponse::with_status("200", "OK", &mime_type, text),
                        Err(_) => {
                            return HttpResponse::with_status(
                                "500",
                                "Server Internal Error",
                                "",
                                "",
                            );
                        }
                    }
                } else {
                    HttpResponse::with_status("200", "OK", &mime_type, data)
                }
            }
            Err(_) => {
                return HttpResponse::with_status(
                    "500",
                    "Server Internal Error",
                    "",
                    "".as_bytes(),
                );
            }
        };
    }
    HttpResponse::with_status("400", "Bad Request", "", "")
}
