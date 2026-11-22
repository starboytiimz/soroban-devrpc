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
        if !self.contract_ids.is_empty() {
            let matched_contract = self
                .contract_ids
                .iter()
                .any(|id| id.eq_ignore_ascii_case(&event.contract_id));
            if !matched_contract {
                return false;
            }
        }
        if !self.topics.is_empty() {
            for (idx, filter_segment) in self.topics.iter().enumerate() {
                if idx >= event.topics.len() {
                    return false;
                }
                if !filter_segment.is_empty()
                    && !filter_segment.iter().any(|t| t == "*" || t == &event.topics[idx])
                {
                    return false;
                }
            }
        }
        true
    }

    pub fn paginate<'a>(
        events: &'a [ContractEvent],
        cursor: Option<&str>,
        limit: Option<usize>,
    ) -> (&'a [ContractEvent], Option<String>) {
        let start_idx = match cursor {
            Some(c) => events
                .iter()
                .position(|e| e.id == c)
                .map(|pos| pos + 1)
                .unwrap_or(0),
            None => 0,
        };
        let effective_events = if start_idx >= events.len() {
            &[]
        } else {
            &events[start_idx..]
        };
        let max_items = limit.unwrap_or(100).min(effective_events.len());
        let slice = &effective_events[..max_items];
        let next_cursor = slice.last().map(|e| e.id.clone());
        (slice, next_cursor)
    }
}
