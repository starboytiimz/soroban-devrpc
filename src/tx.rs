use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Pending,
    Success,
    Failed,
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub hash: String,
    pub status: TransactionStatus,
    pub envelope_xdr: String,
    pub result_xdr: Option<String>,
    pub result_meta_xdr: Option<String>,
    pub submission_ledger: u32,
    pub final_ledger: Option<u32>,
    pub error_message: Option<String>,
}

impl TransactionRecord {
    pub fn new(hash: String, envelope_xdr: String, submission_ledger: u32) -> Self {
        Self {
            hash,
            status: TransactionStatus::Pending,
            envelope_xdr,
            result_xdr: None,
            result_meta_xdr: None,
            submission_ledger,
            final_ledger: None,
            error_message: None,
        }
    }

    pub fn mark_success(&mut self, ledger: u32, result_xdr: String, meta_xdr: String) {
        self.status = TransactionStatus::Success;
        self.final_ledger = Some(ledger);
        self.result_xdr = Some(result_xdr);
        self.result_meta_xdr = Some(meta_xdr);
    }

    pub fn mark_failed(&mut self, ledger: u32, error: String) {
        self.status = TransactionStatus::Failed;
        self.final_ledger = Some(ledger);
        self.error_message = Some(error);
    }
}

pub fn compute_tx_hash(tx_xdr: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher1 = DefaultHasher::new();
    tx_xdr.hash(&mut hasher1);
    let h1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    (tx_xdr, "soroban-devrpc-salt-v1").hash(&mut hasher2);
    let h2 = hasher2.finish();

    let mut hasher3 = DefaultHasher::new();
    (tx_xdr, "soroban-devrpc-salt-v2").hash(&mut hasher3);
    let h3 = hasher3.finish();

    let mut hasher4 = DefaultHasher::new();
    (tx_xdr, "soroban-devrpc-salt-v3").hash(&mut hasher4);
    let h4 = hasher4.finish();

    format!("{:016x}{:016x}{:016x}{:016x}", h1, h2, h3, h4)
}
