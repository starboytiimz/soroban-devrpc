use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub latest_ledger: u32,
    pub oldest_ledger: u32,
    pub ledger_retention_window: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponse {
    pub friendbot_url: Option<String>,
    pub passphrase: String,
    pub protocol_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestLedgerResponse {
    pub id: String,
    pub protocol_version: u32,
    pub sequence: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendTransactionResponse {
    pub status: String,
    pub hash: String,
    #[serde(rename = "latestLedger")]
    pub latest_ledger: u32,
    #[serde(rename = "latestLedgerCloseTime")]
    pub latest_ledger_close_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionResponse {
    pub status: String,
    #[serde(rename = "latestLedger")]
    pub latest_ledger: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger: Option<u32>,
    #[serde(rename = "envelopeXdr", skip_serializing_if = "Option::is_none")]
    pub envelope_xdr: Option<String>,
    #[serde(rename = "resultXdr", skip_serializing_if = "Option::is_none")]
    pub result_xdr: Option<String>,
    #[serde(rename = "resultMetaXdr", skip_serializing_if = "Option::is_none")]
    pub result_meta_xdr: Option<String>,
    #[serde(rename = "errorMessage", skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEventsResponse {
    pub events: Vec<crate::events::ContractEvent>,
    #[serde(rename = "latestLedger")]
    pub latest_ledger: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStatRecord {
    pub min: u64,
    pub mode: u64,
    pub p50: u64,
    pub p90: u64,
    pub p99: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFeeStatsResponse {
    pub soroban_inclusion_fee: FeeStatRecord,
    #[serde(rename = "latestLedger")]
    pub latest_ledger: u32,
}
