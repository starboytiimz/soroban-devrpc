# soroban-devrpc

A lightweight, zero-Docker standalone Soroban JSON-RPC mock and ledger state snapshot engine for rapid local development and automated CI pipelines.

## Overview

Running full Stellar Quickstart Docker containers during local development and continuous integration can introduce container overhead, slow startup times, and complex environment setups. 

`soroban-devrpc` is a local mock RPC daemon for testing basic client connectivity and deterministic application flows without Docker. It is not a Stellar Core replacement: transaction execution and several RPC responses are simulated, and contract calls are not executed by Soroban host.

### Key Capabilities

- **Local HTTP JSON-RPC endpoint**: Binds to `127.0.0.1` by default and runs as a lightweight local binary.
- **Mock RPC methods**: Provides deterministic responses for:
  - `getHealth`
  - `getNetwork`
  - `getLatestLedger`
  - `getLedgerEntries`
  - `simulateTransaction`
  - `sendTransaction`
  - `getTransaction`
- **Library snapshots**: Save and restore ledger snapshots in portable JSON format through the Rust API.
- **Deterministic Simulation**: Simulates contract execution returns, authorization requirements, and resource fee calculations for offline client testing.

The JavaScript SDK smoke example verifies only `getHealth` and `getNetwork`. Other methods use mock data and are not yet validated for full SDK or Stellar RPC specification compatibility.

## Getting Started

### Installation

```bash
# Build from source
cargo install --path .
```

### Running the Daemon

```bash
cargo run -- --port 8000
```

The server is available at `http://127.0.0.1:8000/soroban-rpc`.

### JavaScript SDK smoke

With the daemon running in another terminal:

```bash
cd examples/js_client
npm ci
npm run smoke
```

This uses the pinned `@stellar/stellar-sdk` and makes real HTTP JSON-RPC requests to the local mock.

## Roadmap

- [x] Local HTTP JSON-RPC server and in-memory ledger entry storage
- [ ] Soroban-host-backed transaction simulation and full RPC schema compatibility
- [ ] Snapshot export and import utilities
- [x] JavaScript SDK smoke for health and network calls
- [ ] Broader JavaScript and Rust SDK integration coverage

## License

Apache-2.0 or MIT
