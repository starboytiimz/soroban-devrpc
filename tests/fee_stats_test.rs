use serde_json::json;
use soroban_devrpc::{DevRpcDaemon, JsonRpcRequest};

#[test]
fn test_get_fee_stats_calculation_across_simulated_blocks() {
    let mut daemon = DevRpcDaemon::new("Standalone Testnet");
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("fees-1"),
        method: "getFeeStats".to_string(),
        params: json!({}),
    };
    let res = daemon.handle_request(req);
    let data = res.result.unwrap();
    let fees = &data["soroban_inclusion_fee"];
    assert_eq!(fees["min"], 100);
    assert_eq!(fees["p90"], 150);
}
