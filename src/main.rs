use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tracing::{error, info, warn};

mod http;
use http::HttpRequest;

mod router;
use router::router_request;

mod config;

mod shutdown;
use shutdown::ShutdownError;

mod logger;

#[tokio::main]
async fn main() {
    // init log
    logger::init_logger("./logs");

    // read config
    info!("Reading config from ./config.json");
    let config = match config::read_config().await {
        Ok(c) => c,
        Err(e) => {
            error!("Reading config fail: {:?}", e);
            panic!("Reading config fail, {:?}", e);
        }
    };
    let addr = format!("{}:{}", config.host, config.port);
    // bind address
    let listener = match TcpListener::bind(&addr).await {
        Ok(tcp_listener) => tcp_listener,
        Err(e) => {
            error!("Cannot bind to address {}: {}", addr, e);
            panic!("Cannot bind to address {}, {}", addr, e);
        }
    };
    let notify_shutdown = match shutdown::start_shutdown_listener() {
        Ok(val) => val,
        Err(e) => {
            match e {
                ShutdownError::SignalBindFail => warn!("Ctrl C bind Error"),
            }
            Arc::new(tokio::sync::Notify::new())
        }
    };
    info!("Listening on {}", addr);
    loop {
        tokio::select! {
            _ = notify_shutdown.notified() => {
                info!("Shutting down...");
                break;
            }
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((socket, addr)) => {
                        info!("New connection from {}", addr);
                        tokio::spawn(async move {
                            handle_connection(socket).await;
                        });
                    }
                    Err(e) => {
                        error!("Accept fail: {}", e);
                    }
                }
            }
        }
    }
}

async fn handle_connection(mut socket: TcpStream) {
    let (reader, mut writer) = socket.split();
    let reader = BufReader::new(reader);
    // request
    let request = match HttpRequest::try_from_reader(reader).await {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to parse request: {:#?}", e);
            return;
        }
    };
    info!("Request received: {} {}", request.method, request.path);
    let response = router_request(&request).await;
    info!("Response status: {}", response.code);
    if let Err(e) = writer.write_all(&response.gen_resp_bytes()).await {
        match e.kind() {
            tokio::io::ErrorKind::NotConnected => {}
            _ => {
                error!("Failed to write response: {:#?}", e);
            }
        }
    }
    if let Err(e) = writer.shutdown().await {
        match e.kind() {
            tokio::io::ErrorKind::NotConnected => {}
            _ => {
                error!("Failed to shutdown connection: {:#?}", e);
            }
        }
    }
}
