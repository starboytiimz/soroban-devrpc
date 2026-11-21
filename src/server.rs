use crate::events::{ContractEvent, EventFilter};
use crate::ledger::InMemoryLedger;
use crate::rpc::{
    FeeStatRecord, GetEventsResponse, GetFeeStatsResponse, GetTransactionResponse, HealthResponse,
    JsonRpcRequest, JsonRpcResponse, LatestLedgerResponse, NetworkResponse, SendTransactionResponse,
    VersionResponse,
};
use crate::tx::{compute_tx_hash, TransactionRecord, TransactionStatus};
use serde_json::json;
use std::collections::HashMap;

pub struct DevRpcDaemon {
    pub ledger: InMemoryLedger,
    pub transactions: HashMap<String, TransactionRecord>,
    pub events: Vec<ContractEvent>,
}

impl DevRpcDaemon {
    pub fn new(passphrase: impl Into<String>) -> Self {
        Self {
            ledger: InMemoryLedger::new(passphrase),
            transactions: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// Clear simulated ledger and request history while retaining network configuration.
    pub fn reset(&mut self) {
        let passphrase = self.ledger.network_passphrase.clone();
        let protocol_version = self.ledger.protocol_version;
        self.ledger = InMemoryLedger::new(passphrase);
        self.ledger.protocol_version = protocol_version;
        self.transactions.clear();
        self.events.clear();
    }

    pub fn record_event(
        &mut self,
        contract_id: impl Into<String>,
        topics: Vec<String>,
        data: impl Into<String>,
    ) {
        let id = format!("{}-{}", self.ledger.sequence, self.events.len() + 1);
        self.events.push(ContractEvent {
            contract_id: contract_id.into(),
            topics,
            data: data.into(),
            in_successful_contract_call: true,
            ledger: self.ledger.sequence,
            id,
        });
    }

    pub fn advance_ledger(&mut self) -> u32 {
        let seq = self.ledger.advance_ledger();
        // Progress any pending transactions into final status in the new ledger
        for tx in self.transactions.values_mut() {
            if tx.status == TransactionStatus::Pending {
                if tx.envelope_xdr.starts_with("FAIL_") {
                    tx.mark_failed(seq, "Transaction simulation failed during ledger inclusion".to_string());
                } else {
                    tx.mark_success(
                        seq,
                        "AAAAAQAAAAAAAAAA".to_string(),
                        "AAAAAgAAAAAAAAAA".to_string(),
                    );
                }
            }
        }
        seq
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
            "getVersion" => {
                let v = VersionResponse {
                    app_version: env!("CARGO_PKG_VERSION").to_string(),
                    build_time_stamp: "2026-11-20T00:00:00Z".to_string(),
                    commit_hash: "mock-commit-hash".to_string(),
                    protocol_version: self.ledger.protocol_version,
                    version: env!("CARGO_PKG_VERSION").to_string(),
                };
                JsonRpcResponse::success(req.id, json!(v))
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
            "sendTransaction" => {
                let tx_xdr = req.params.get("transaction").and_then(|t| t.as_str());
                match tx_xdr {
                    Some(raw_xdr) if !raw_xdr.trim().is_empty() => {
                        let hash = compute_tx_hash(raw_xdr);
                        if self.transactions.contains_key(&hash) {
                            let res = SendTransactionResponse {
                                status: "DUPLICATE".to_string(),
                                hash,
                                latest_ledger: self.ledger.sequence,
                                latest_ledger_close_time: "1732014000".to_string(),
                            };
                            return JsonRpcResponse::success(req.id, json!(res));
                        }

                        let record = TransactionRecord::new(hash.clone(), raw_xdr.to_string(), self.ledger.sequence);
                        self.transactions.insert(hash.clone(), record);

                        let res = SendTransactionResponse {
                            status: "PENDING".to_string(),
                            hash,
                            latest_ledger: self.ledger.sequence,
                            latest_ledger_close_time: "1732014000".to_string(),
                        };
                        JsonRpcResponse::success(req.id, json!(res))
                    }
                    _ => JsonRpcResponse::error(
                        req.id,
                        -32602,
                        "Missing or empty 'transaction' parameter in sendTransaction",
                    ),
                }
            }
            "getTransaction" => {
                let hash = req.params.get("hash").and_then(|h| h.as_str());
                match hash {
                    Some(tx_hash) => {
                        if let Some(record) = self.transactions.get(tx_hash) {
                            let status_str = match record.status {
                                TransactionStatus::Pending => "PENDING",
                                TransactionStatus::Success => "SUCCESS",
                                TransactionStatus::Failed => "FAILED",
                                TransactionStatus::NotFound => "NOT_FOUND",
                            };
                            let res = GetTransactionResponse {
                                status: status_str.to_string(),
                                latest_ledger: self.ledger.sequence,
                                ledger: record.final_ledger,
                                envelope_xdr: Some(record.envelope_xdr.clone()),
                                result_xdr: record.result_xdr.clone(),
                                result_meta_xdr: record.result_meta_xdr.clone(),
                                error_message: record.error_message.clone(),
                            };
                            JsonRpcResponse::success(req.id, json!(res))
                        } else {
                            let res = GetTransactionResponse {
                                status: "NOT_FOUND".to_string(),
                                latest_ledger: self.ledger.sequence,
                                ledger: None,
                                envelope_xdr: None,
                                result_xdr: None,
                                result_meta_xdr: None,
                                error_message: None,
                            };
                            JsonRpcResponse::success(req.id, json!(res))
                        }
                    }
                    None => JsonRpcResponse::error(req.id, -32602, "Missing 'hash' parameter in getTransaction"),
                }
            }
            "getEvents" => {
                let start_ledger = req
                    .params
                    .get("startLedger")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                let filter = EventFilter {
                    start_ledger,
                    contract_ids: Vec::new(),
                    topics: Vec::new(),
                };

                let matched: Vec<ContractEvent> = self
                    .events
                    .iter()
                    .filter(|e| filter.matches(e))
                    .cloned()
                    .collect();

                let res = GetEventsResponse {
                    events: matched,
                    latest_ledger: self.ledger.sequence,
                };
                JsonRpcResponse::success(req.id, json!(res))
            }
            "getFeeStats" => {
                let res = GetFeeStatsResponse {
                    soroban_inclusion_fee: FeeStatRecord {
                        min: 100,
                        mode: 100,
                        p50: 100,
                        p90: 150,
                        p99: 300,
                    },
                    latest_ledger: self.ledger.sequence,
                };
                JsonRpcResponse::success(req.id, json!(res))
            }
            _ => JsonRpcResponse::error(req.id, -32601, format!("Method '{}' not found", req.method)),
        }
    }
}
