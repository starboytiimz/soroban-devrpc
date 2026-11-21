use soroban_devrpc::{http, DevRpcDaemon};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut port = 8000_u16;
    let mut passphrase = "Test SDF Network ; September 2015".to_string();
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--port" => port = args.next().ok_or("--port requires a value")?.parse()?,
            "--passphrase" => passphrase = args.next().ok_or("--passphrase requires a value")?,
            "--help" | "-h" => {
                println!("Usage: soroban-devrpc [--port PORT] [--passphrase PASSPHRASE]");
                println!("Serves mock Soroban JSON-RPC over HTTP on 127.0.0.1.");
                return Ok(());
            }
            other => return Err(format!("unknown argument: {other}; use --help").into()),
        }
    }

    let address = format!("127.0.0.1:{port}");
    eprintln!("Soroban DevRPC listening at http://{address}/soroban-rpc");
    http::serve(&address, DevRpcDaemon::new(passphrase))?;
    Ok(())
}
