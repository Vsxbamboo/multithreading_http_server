use futures::stream::StreamExt;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader}; // Use tokio::io::BufReader, AsyncBufReadExt, and AsyncWriteExt
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::TcpListenerStream;
#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8878").await.unwrap();
    TcpListenerStream::new(listener)
        .for_each_concurrent(None, |stream_result| async {
            match stream_result {
                Ok(stream) => {
                    handle_connection(stream).await;
                }
                Err(e) => eprintln!("Connection error: {}", e),
            }
        })
        .await;
}

async fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let mut lines = buf_reader.lines();
    let request_line = lines.next_line().await.unwrap().unwrap();
    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "./src/hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "./src/404.html")
    };

    let contents = fs::read_to_string(filename).await.unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).await.unwrap();
}