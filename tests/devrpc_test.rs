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

#[test]
fn test_send_and_get_transaction_lifecycle() {
    let mut daemon = DevRpcDaemon::new("Standalone Testnet");

    // 1. Submit valid transaction
    let send_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("tx-submit-1"),
        method: "sendTransaction".to_string(),
        params: json!({
            "transaction": "AAAAAgAAAABiX+t66P7s8u3K2h83z..."
        }),
    };
    let send_res = daemon.handle_request(send_req);
    assert_eq!(send_res.jsonrpc, "2.0");
    let result = send_res.result.unwrap();
    assert_eq!(result["status"], "PENDING");
    let tx_hash = result["hash"].as_str().unwrap();
    assert_eq!(tx_hash.len(), 64);

    // 2. Poll transaction before ledger close
    let poll_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("tx-poll-1"),
        method: "getTransaction".to_string(),
        params: json!({
            "hash": tx_hash
        }),
    };
    let poll_res = daemon.handle_request(poll_req);
    let poll_data = poll_res.result.unwrap();
    assert_eq!(poll_data["status"], "PENDING");
    assert!(poll_data["ledger"].is_null());

    // 3. Close ledger (progress pending transactions)
    let new_seq = daemon.advance_ledger();
    assert_eq!(new_seq, 1001);

    // 4. Poll transaction after ledger close -> SUCCESS
    let poll_req2 = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("tx-poll-2"),
        method: "getTransaction".to_string(),
        params: json!({
            "hash": tx_hash
        }),
    };
    let poll_res2 = daemon.handle_request(poll_req2);
    let poll_data2 = poll_res2.result.unwrap();
    assert_eq!(poll_data2["status"], "SUCCESS");
    assert_eq!(poll_data2["ledger"], 1001);
    assert!(poll_data2["resultXdr"].as_str().is_some());
    assert!(poll_data2["resultMetaXdr"].as_str().is_some());
}

#[test]
fn test_transaction_failure_and_missing_checks() {
    let mut daemon = DevRpcDaemon::new("Standalone Testnet");

    // Query nonexistent transaction
    let missing_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("poll-missing"),
        method: "getTransaction".to_string(),
        params: json!({
            "hash": "0000000000000000000000000000000000000000000000000000000000000000"
        }),
    };
    let missing_res = daemon.handle_request(missing_req);
    let missing_data = missing_res.result.unwrap();
    assert_eq!(missing_data["status"], "NOT_FOUND");

    // Submit a failing transaction
    let fail_tx_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("fail-submit"),
        method: "sendTransaction".to_string(),
        params: json!({
            "transaction": "FAIL_INVALID_INVOCATION_AUTH"
        }),
    };
    let fail_submit_res = daemon.handle_request(fail_tx_req);
    let fail_hash = fail_submit_res.result.unwrap()["hash"].as_str().unwrap().to_string();

    daemon.advance_ledger();

    let fail_poll_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("fail-poll"),
        method: "getTransaction".to_string(),
        params: json!({
            "hash": fail_hash
        }),
    };
    let fail_poll_res = daemon.handle_request(fail_poll_req);
    let fail_poll_data = fail_poll_res.result.unwrap();
    assert_eq!(fail_poll_data["status"], "FAILED");
    assert!(fail_poll_data["errorMessage"].as_str().unwrap().contains("failed"));

    // Validation error on empty transaction
    let bad_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("bad-submit"),
        method: "sendTransaction".to_string(),
        params: json!({
            "transaction": "   "
        }),
    };
    let bad_res = daemon.handle_request(bad_req);
    assert!(bad_res.error.is_some());
    assert_eq!(bad_res.error.unwrap().code, -32602);
}

#[test]
fn test_snapshot_with_transaction_history() {
    let mut daemon = DevRpcDaemon::new("Standalone Testnet");
    let send_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("tx-1"),
        method: "sendTransaction".to_string(),
        params: json!({ "transaction": "AAAAAg..." }),
    };
    let send_res = daemon.handle_request(send_req);
    let hash = send_res.result.unwrap()["hash"].as_str().unwrap().to_string();

    let snapshot = LedgerSnapshot::from_daemon(&daemon);
    let json_str = snapshot.to_json().unwrap();

    let restored = LedgerSnapshot::from_json(&json_str).unwrap();
    let new_daemon = restored.to_daemon();
    assert!(new_daemon.transactions.contains_key(&hash));
}
