use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Transaction {
    pub id: String,         // Unique transaction identifier (Public Wallet Address)
    pub signature: String,  // Signature for verification
    pub nonce: u64,         // Nonce value for batching
    pub payload: String,    // Raw transaction data
}

impl Transaction {
    pub fn new(id: String, signature: String, nonce: u64, payload: String) -> Self {
        Transaction {
            id,
            signature,
            nonce,
            payload,
        }
    }
}
