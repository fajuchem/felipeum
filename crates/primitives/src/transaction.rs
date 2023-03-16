use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: TransactionId,
    pub sender: String,
    pub hash: String,
    pub nonce: u64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct TransactionId {
    pub sender: u64,
    pub nonce: u64,
}

impl TransactionId {
    pub fn new(sender: u64, nonce: u64) -> Self {
        Self { sender, nonce }
    }
}
