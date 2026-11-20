use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LedgerEntryData {
    pub key_xdr: String,
    pub val_xdr: String,
    pub live_until_ledger_seq: u32,
    pub last_modified_ledger_seq: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InMemoryLedger {
    pub sequence: u32,
    pub protocol_version: u32,
    pub network_passphrase: String,
    pub entries: HashMap<String, LedgerEntryData>,
}

impl InMemoryLedger {
    pub fn new(network_passphrase: impl Into<String>) -> Self {
        Self {
            sequence: 1000,
            protocol_version: 21,
            network_passphrase: network_passphrase.into(),
            entries: HashMap::new(),
        }
    }

    pub fn set_entry(&mut self, key_xdr: impl Into<String>, val_xdr: impl Into<String>, ttl_seq: u32) {
        let key = key_xdr.into();
        let val = val_xdr.into();
        self.entries.insert(
            key.clone(),
            LedgerEntryData {
                key_xdr: key,
                val_xdr: val,
                live_until_ledger_seq: ttl_seq,
                last_modified_ledger_seq: self.sequence,
            },
        );
    }

    pub fn get_entry(&self, key_xdr: &str) -> Option<&LedgerEntryData> {
        self.entries.get(key_xdr)
    }

    pub fn bump_entry_ttl(&mut self, key_xdr: &str, ledgers_to_extend: u32) -> Option<u32> {
        if let Some(entry) = self.entries.get_mut(key_xdr) {
            let target_seq = self.sequence + ledgers_to_extend;
            if target_seq > entry.live_until_ledger_seq {
                entry.live_until_ledger_seq = target_seq;
            }
            entry.last_modified_ledger_seq = self.sequence;
            Some(entry.live_until_ledger_seq)
        } else {
            None
        }
    }

    pub fn advance_ledger(&mut self) -> u32 {
        self.sequence += 1;
        self.sequence
    }
}
