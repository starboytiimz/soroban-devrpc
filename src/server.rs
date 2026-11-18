use crate::ledger::InMemoryLedger;
use crate::rpc::{HealthResponse, JsonRpcRequest, JsonRpcResponse, LatestLedgerResponse, NetworkResponse};
use serde_json::json;

pub struct DevRpcDaemon {
    pub ledger: InMemoryLedger,
}

impl DevRpcDaemon {
    pub fn new(passphrase: impl Into<String>) -> Self {
        Self {
            ledger: InMemoryLedger::new(passphrase),
        }
    }

    pub fn handle_request(&mut self, req: JsonRpcRequest) -> JsonRpcResponse {
        match req.method.as_str() {
            "getHealth" => {
                let health = HealthResponse {
                    status: "healthy".to_string(),
                    latest_ledger: self.ledger.sequence,
                    oldest_ledger: 1,
                    ledger_retention_window: 100_000,
                };
                JsonRpcResponse::success(req.id, json!(health))
            }
            "getNetwork" => {
                let net = NetworkResponse {
                    friendbot_url: Some("http://localhost:8000/friendbot".to_string()),
                    passphrase: self.ledger.network_passphrase.clone(),
                    protocol_version: self.ledger.protocol_version,
                };
                JsonRpcResponse::success(req.id, json!(net))
            }
            "getLatestLedger" => {
                let l = LatestLedgerResponse {
                    id: format!("ledger-hash-{}", self.ledger.sequence),
                    protocol_version: self.ledger.protocol_version,
                    sequence: self.ledger.sequence,
                };
                JsonRpcResponse::success(req.id, json!(l))
            }
            "getLedgerEntries" => {
                let keys = req.params.get("keys").and_then(|k| k.as_array());
                match keys {
                    Some(key_list) => {
                        let mut found_entries = Vec::new();
                        for k in key_list {
                            if let Some(s) = k.as_str() {
                                if let Some(entry) = self.ledger.get_entry(s) {
                                    found_entries.push(json!({
                                        "key": entry.key_xdr,
                                        "xdr": entry.val_xdr,
                                        "lastModifiedLedgerSeq": entry.last_modified_ledger_seq,
                                        "liveUntilLedgerSeq": entry.live_until_ledger_seq
                                    }));
                                }
                            }
                        }
                        JsonRpcResponse::success(
                            req.id,
                            json!({
                                "entries": found_entries,
                                "latestLedger": self.ledger.sequence
                            }),
                        )
                    }
                    None => JsonRpcResponse::error(req.id, -32602, "Missing 'keys' parameter in getLedgerEntries"),
                }
            }
            "simulateTransaction" => {
                // Mock execution simulation return
                JsonRpcResponse::success(
                    req.id,
                    json!({
                        "latestLedger": self.ledger.sequence,
                        "minResourceFee": "100",
                        "cost": {
                            "cpuInsns": "154200",
                            "memBytes": "32100"
                        },
                        "results": [{
                            "auth": [],
                            "xdr": "AAAAAg=="
                        }]
                    }),
                )
            }
            _ => JsonRpcResponse::error(req.id, -32601, format!("Method '{}' not found", req.method)),
        }
    }
}
