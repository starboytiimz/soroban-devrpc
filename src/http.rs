use crate::{DevRpcDaemon, JsonRpcRequest};
use serde_json::Value;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Serve JSON-RPC over HTTP on the supplied address. Use a loopback address for local development.
pub fn serve(address: &str, daemon: DevRpcDaemon) -> io::Result<()> {
    let listener = TcpListener::bind(address)?;
    let shared = Arc::new(Mutex::new(daemon));
    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                let shared = Arc::clone(&shared);
                thread::spawn(move || {
                    if let Err(error) = handle_connection(stream, &shared) {
                        eprintln!("JSON-RPC connection failed: {error}");
                    }
                });
            }
            Err(error) => eprintln!("JSON-RPC accept failed: {error}"),
        }
    }
    Ok(())
}

fn handle_connection(stream: TcpStream, daemon: &Arc<Mutex<DevRpcDaemon>>) -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    if request_line.is_empty() {
        return Ok(());
    }

    let mut content_length = None;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line == "\r\n" || line == "\n" || line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse::<usize>().ok();
            }
        }
    }

    let (status, response) = if !request_line.starts_with("POST ") {
        (
            "405 Method Not Allowed",
            json_rpc_error(Value::Null, -32600, "JSON-RPC requires POST"),
        )
    } else {
        match content_length {
            Some(length) if length <= MAX_BODY_BYTES => {
                let mut body = vec![0; length];
                reader.read_exact(&mut body)?;
                match serde_json::from_slice::<JsonRpcRequest>(&body) {
                    Ok(request) => {
                        let response = daemon
                            .lock()
                            .unwrap_or_else(|error| error.into_inner())
                            .handle_request(request);
                        ("200 OK", response)
                    }
                    Err(error) => (
                        "200 OK",
                        json_rpc_error(Value::Null, -32700, error.to_string()),
                    ),
                }
            }
            Some(_) => (
                "413 Payload Too Large",
                json_rpc_error(Value::Null, -32600, "Request body exceeds 1 MiB"),
            ),
            None => (
                "411 Length Required",
                json_rpc_error(Value::Null, -32600, "Content-Length header is required"),
            ),
        }
    };

    let body = serde_json::to_vec(&response).map_err(io::Error::other)?;
    let mut stream = stream;
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(&body)?;
    stream.flush()
}

fn json_rpc_error(id: Value, code: i32, message: impl Into<String>) -> crate::rpc::JsonRpcResponse {
    crate::rpc::JsonRpcResponse::error(id, code, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_post_returns_camel_case_soroban_health_response() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let daemon = Arc::new(Mutex::new(DevRpcDaemon::new("Testnet")));
        let server_daemon = Arc::clone(&daemon);
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream, &server_daemon).unwrap();
        });

        let mut client = TcpStream::connect(address).unwrap();
        let body = br#"{"jsonrpc":"2.0","id":7,"method":"getHealth","params":{}}"#;
        write!(
            client,
            "POST /soroban-rpc HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        client.write_all(body).unwrap();

        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        server.join().unwrap();
        let payload = response.split_once("\r\n\r\n").unwrap().1;
        let json: Value = serde_json::from_str(payload).unwrap();
        assert_eq!(json["id"], 7);
        assert_eq!(json["result"]["status"], "healthy");
        assert_eq!(json["result"]["latestLedger"], 1000);
        assert_eq!(json["result"]["oldestLedger"], 1);
        assert_eq!(json["result"]["ledgerRetentionWindow"], 100_000);
    }

    #[test]
    fn invalid_json_returns_a_parse_error_response() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let daemon = Arc::new(Mutex::new(DevRpcDaemon::new("Testnet")));
        let server_daemon = Arc::clone(&daemon);
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream, &server_daemon).unwrap();
        });

        let mut client = TcpStream::connect(address).unwrap();
        let body = b"{";
        write!(
            client,
            "POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        client.write_all(body).unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        server.join().unwrap();
        let payload = response.split_once("\r\n\r\n").unwrap().1;
        let json: Value = serde_json::from_str(payload).unwrap();
        assert_eq!(json["error"]["code"], -32700);
        assert!(json["error"]["message"].as_str().is_some());
    }
}
