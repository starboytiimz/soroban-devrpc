use serde_json::json;
use soroban_devrpc::{DevRpcDaemon, JsonRpcRequest, LedgerSnapshot};

#[test]
fn test_get_health_and_network() {
    let mut daemon = DevRpcDaemon::new("Test SDF Network ; September 2015");

    let req_health = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "getHealth".to_string(),
        params: json!({}),
    };
    let res = daemon.handle_request(req_health);
    assert_eq!(res.jsonrpc, "2.0");
    let result = res.result.unwrap();
    assert_eq!(result["status"], "healthy");
    assert_eq!(result["latest_ledger"], 1000);

    let req_net = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(2),
        method: "getNetwork".to_string(),
        params: json!({}),
    };
    let res_net = daemon.handle_request(req_net);
    let net_res = res_net.result.unwrap();
    assert_eq!(net_res["passphrase"], "Test SDF Network ; September 2015");
}

#[test]
fn test_ledger_entries_query() {
    let mut daemon = DevRpcDaemon::new("Standalone Testnet");
    daemon.ledger.set_entry("ENTRY_KEY_1", "VAL_XDR_DATA", 2000);

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("req-1"),
        method: "getLedgerEntries".to_string(),
        params: json!({
            "keys": ["ENTRY_KEY_1", "NON_EXISTENT_KEY"]
        }),
    };
    let res = daemon.handle_request(req);
    let result = res.result.unwrap();
    let entries = result["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["key"], "ENTRY_KEY_1");
    assert_eq!(entries[0]["xdr"], "VAL_XDR_DATA");
}

#[test]
fn test_transaction_simulation() {
    let mut daemon = DevRpcDaemon::new("Standalone Testnet");
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(42),
        method: "simulateTransaction".to_string(),
        params: json!({
            "transaction": "AAAAAg..."
        }),
    };
    let res = daemon.handle_request(req);
    let result = res.result.unwrap();
    assert_eq!(result["minResourceFee"], "100");
    assert!(result["cost"]["cpuInsns"].as_str().is_some());
}

#[test]
fn test_snapshot_export_and_restore() {
    let mut daemon = DevRpcDaemon::new("Snapshot Testnet");
    daemon.ledger.set_entry("CONTRACT_DATA_KEY", "WASM_HASH_XDR", 5000);
    daemon.ledger.advance_ledger();

    // Export snapshot
    let snapshot = LedgerSnapshot::from_ledger(&daemon.ledger);
    let json_str = snapshot.to_json().expect("Snapshot serialization failed");

    // Restore to new daemon instance
    let restored_snapshot = LedgerSnapshot::from_json(&json_str).expect("Deserialization failed");
    let restored_ledger = restored_snapshot.to_ledger();

    assert_eq!(restored_ledger.sequence, 1001);
    assert_eq!(restored_ledger.network_passphrase, "Snapshot Testnet");
    let entry = restored_ledger.get_entry("CONTRACT_DATA_KEY").unwrap();
    assert_eq!(entry.val_xdr, "WASM_HASH_XDR");
}
