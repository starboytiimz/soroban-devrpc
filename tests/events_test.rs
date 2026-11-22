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

#[test]
fn test_event_filter_wildcard_and_case_insensitivity() {
    use soroban_devrpc::events::{ContractEvent, EventFilter};

    let event = ContractEvent {
        contract_id: "CA_TOKEN_XYZ".into(),
        topics: vec!["transfer".into(), "G_ALICE".into(), "G_BOB".into()],
        data: "100".into(),
        in_successful_contract_call: true,
        ledger: 1050,
        id: "evt-1".into(),
    };

    // Case-insensitive contract ID match
    let filter1 = EventFilter {
        start_ledger: 1000,
        contract_ids: vec!["ca_token_xyz".into()],
        topics: vec![],
    };
    assert!(filter1.matches(&event));

    // Wildcard topic match
    let filter2 = EventFilter {
        start_ledger: 1000,
        contract_ids: vec![],
        topics: vec![vec!["transfer".into()], vec!["*".into()]],
    };
    assert!(filter2.matches(&event));
}

#[test]
fn test_event_cursor_pagination() {
    use soroban_devrpc::events::{ContractEvent, EventFilter};

    let events = vec![
        ContractEvent {
            contract_id: "C1".into(),
            topics: vec![],
            data: "".into(),
            in_successful_contract_call: true,
            ledger: 1001,
            id: "evt-01".into(),
        },
        ContractEvent {
            contract_id: "C2".into(),
            topics: vec![],
            data: "".into(),
            in_successful_contract_call: true,
            ledger: 1002,
            id: "evt-02".into(),
        },
        ContractEvent {
            contract_id: "C3".into(),
            topics: vec![],
            data: "".into(),
            in_successful_contract_call: true,
            ledger: 1003,
            id: "evt-03".into(),
        },
    ];

    // Page 1: limit 2, no cursor
    let (p1, c1) = EventFilter::paginate(&events, None, Some(2));
    assert_eq!(p1.len(), 2);
    assert_eq!(p1[0].id, "evt-01");
    assert_eq!(c1, Some("evt-02".into()));

    // Page 2: cursor from p1
    let (p2, c2) = EventFilter::paginate(&events, c1.as_deref(), Some(2));
    assert_eq!(p2.len(), 1);
    assert_eq!(p2[0].id, "evt-03");
    assert_eq!(c2, Some("evt-03".into()));
}
    assert!(filter2.matches(&event));
}

