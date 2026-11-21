# Soroban DevRPC API Reference

`soroban-devrpc` is a lightweight, local mock JSON-RPC server implementing the standard Stellar Soroban RPC specification for rapid smart contract development, unit testing, and simulation harnesses without running a full standalone network validator.

---

## 1. Supported JSON-RPC 2.0 Methods

| Method | Description |
|---|---|
| `getHealth` | Returns server health status, current sequence, oldest sequence, and retention window. |
| `getVersion` | Returns Soroban RPC protocol version, build timestamp, commit hash, and app semver. |
| `getNetwork` | Returns network passphrase and protocol version. |
| `getLatestLedger` | Returns latest ledger sequence, synthetic ledger hash ID, protocol, and close time. |
| `getLedgerEntries` | Queries simulated ledger storage entries by base64-encoded `LedgerKey` XDR. |
| `sendTransaction` | Submits a transaction envelope XDR to simulated mempool with duplicate hash detection. |
| `getTransaction` | Polls transaction submission status (`SUCCESS`, `NOT_FOUND`, `FAILED`). |
| `simulateTransaction` | Dry-run execution measuring min resource fees, footprint bounds, and auth requirements. |
| `getEvents` | Queries contract emission event logs with topic matching and sequence pagination. |
| `getFeeStats` | Calculates fee distributions across simulated blocks. |

---

## 2. Curl Examples

### Health Check
```bash
curl -X POST http://127.0.0.1:8000/ \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
```

### Query Latest Ledger
```bash
curl -X POST http://127.0.0.1:8000/ \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getLatestLedger"}'
```
