use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractEvent {
    pub contract_id: String,
    pub topics: Vec<String>,
    pub data: String,
    pub in_successful_contract_call: bool,
    pub ledger: u32,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventFilter {
    pub start_ledger: u32,
    pub contract_ids: Vec<String>,
    pub topics: Vec<Vec<String>>,
}

impl EventFilter {
    pub fn matches(&self, event: &ContractEvent) -> bool {
        if event.ledger < self.start_ledger {
            return false;
        }
        if !self.contract_ids.is_empty() && !self.contract_ids.contains(&event.contract_id) {
            return false;
        }
        if !self.topics.is_empty() {
            for (idx, filter_segment) in self.topics.iter().enumerate() {
                if idx >= event.topics.len() {
                    return false;
                }
                if !filter_segment.is_empty() && !filter_segment.contains(&event.topics[idx]) {
                    return false;
                }
            }
        }
        true
    }
}
