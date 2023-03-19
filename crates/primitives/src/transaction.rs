use rlp::{Encodable, RlpStream};
use serde::{Deserialize, Serialize};

use crate::{signature::Signature, TxHash};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSigned {
    pub hash: TxHash,
    pub signature: Signature,
    pub transaction: Transaction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub nonce: u64,
}

impl Transaction {
    pub fn signature_hash(&self) -> [u8; 32] {
        let encoded = rlp::encode(self);
        keccak256(&encoded)
    }
}

pub fn keccak256(data: impl AsRef<[u8]>) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};

    let mut buf = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(data.as_ref());
    hasher.finalize(&mut buf);
    buf
}

impl Encodable for Transaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3);
        s.append(&self.nonce);
        s.append(&self.from);
        s.append(&self.to);
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct TransactionId {
    pub from: String,
    pub nonce: u64,
}

impl TransactionId {
    pub fn new(from: String, nonce: u64) -> Self {
        Self { from, nonce }
    }
}
