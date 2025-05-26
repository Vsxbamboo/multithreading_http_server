use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

mod http;
use http::HttpRequest;

mod router;
use router::router_request;

mod utils;

#[tokio::main]
async fn main() {
    const ADDR: &str = "0.0.0.0:8080";
    let listener = match TcpListener::bind(ADDR).await {
        Ok(tcp_listener) => tcp_listener,
        Err(e) => {
            panic!("Cannot bind to address {}, {}", ADDR, e);
        }
    };
    println!("Listening on {}", ADDR);
    loop {
        let (socket, _socket_addr) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}

async fn handle_connection(mut socket: TcpStream) {
    let (reader, mut writer) = socket.split();
    let reader = BufReader::new(reader);
    // request
    let request = match HttpRequest::try_from_reader(reader).await {
        Ok(r) => r,
        Err(e) => {
            println!("Err: {:#?}", e);
            return;
        }
    };
    // println!("request received:\n{:#?}", request);
    let response = router_request(&request).await;
    // println!("response sent:\n{:#?}", response);
    if let Err(e) = writer.write_all(&response.gen_resp_bytes()).await {
        println!("Err: {:#?}", e);
    }
    if let Err(e) = writer.shutdown().await {
        println!("Err: {:#?}", e);
    }
}
