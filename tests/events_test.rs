use serde_json::json;
use soroban_devrpc::{DevRpcDaemon, JsonRpcRequest};

#[test]
fn test_get_events_pagination_and_topic_filter() {
    let mut daemon = DevRpcDaemon::new("Standalone Testnet");
    daemon.record_event(
        "CD_TOKEN",
        vec!["transfer".to_string(), "G_ALICE".to_string()],
        "AAAAEAAAAAE=",
    );
    daemon.record_event(
        "CA_DEX",
        vec!["swap".to_string()],
        "AAAAEAAAAAI=",
    );

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("events-1"),
        method: "getEvents".to_string(),
        params: json!({
            "startLedger": 1000
        }),
    };
    let res = daemon.handle_request(req);
    let data = res.result.unwrap();
    let events = data["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["contract_id"], "CD_TOKEN");
    assert_eq!(events[1]["contract_id"], "CA_DEX");
}
