# soroban-devrpc

A lightweight, zero-Docker standalone Soroban JSON-RPC mock and ledger state snapshot engine for rapid local development and automated CI pipelines.

## Overview

Running full Stellar Quickstart Docker containers during local development and continuous integration can introduce container overhead, slow startup times, and complex environment setups. 

`soroban-devrpc` is a fast, standalone mock RPC daemon designed specifically for testing Soroban client applications, contract bindings, and frontend dApps without Docker dependencies.

### Key Capabilities

- **Zero-Docker Architecture**: Runs as a lightweight local binary or test fixture with near-instant startup (<50ms).
- **Core JSON-RPC Methods**: Implements standard Soroban RPC methods:
  - `getHealth`
  - `getNetwork`
  - `getLatestLedger`
  - `getLedgerEntries`
  - `simulateTransaction`
  - `sendTransaction`
  - `getTransaction`
- **State Snapshots**: Save and restore ledger state snapshots in portable JSON format to establish clean, reproducible test baselines.
- **Deterministic Simulation**: Simulates contract execution returns, authorization requirements, and resource fee calculations for offline client testing.

## Getting Started

### Installation

```bash
# Build from source
cargo install --path .
```

### Running the Daemon

```bash
soroban-devrpc --port 8000
```

### State Snapshots

Load a snapshot of pre-populated ledger entries:

```bash
soroban-devrpc --snapshot fixtures/vault_state.json --port 8000
```

## Roadmap

- [ ] Core JSON-RPC server and in-memory ledger entry storage
- [ ] Transaction simulation engine for contract execution
- [ ] Snapshot export and import utilities
- [ ] Integration harnesses for JavaScript/TypeScript and Rust SDKs

## License

Apache-2.0 or MIT
