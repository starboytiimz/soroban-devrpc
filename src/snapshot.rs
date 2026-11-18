use crate::ledger::{InMemoryLedger, LedgerEntryData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerSnapshot {
    pub version: u32,
    pub network: String,
    pub sequence: u32,
    pub entries: HashMap<String, LedgerEntryData>,
}

impl LedgerSnapshot {
    pub fn from_ledger(ledger: &InMemoryLedger) -> Self {
        Self {
            version: 1,
            network: ledger.network_passphrase.clone(),
            sequence: ledger.sequence,
            entries: ledger.entries.clone(),
        }
    }

    pub fn to_ledger(&self) -> InMemoryLedger {
        InMemoryLedger {
            sequence: self.sequence,
            protocol_version: 21,
            network_passphrase: self.network.clone(),
            entries: self.entries.clone(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}
