
use crate::Transaction;
use serde_json::{json, Value};

pub fn has_consecutive_nonces(transactions: &Vec<Transaction>) -> bool {
    for i in 1..transactions.len() {
        if transactions[i].nonce != transactions[i - 1].nonce + 1 {
            return false;
        }
    }
    true
}

pub fn convert_transactions_to_json(transactions: &[Transaction]) -> Value {
    let limited_transactions = &transactions[..transactions.len().min(5)]; // Limit to 5 transactions

    let start_nonce = limited_transactions[0].nonce;
    let public_key = limited_transactions[0].id.as_str();
    let payload: Vec<String> = limited_transactions.iter().map(|tx| tx.payload.clone()).collect();
    let signature: Vec<String> = limited_transactions.iter().map(|tx| tx.signature.clone()).collect();

    // Create the JSON structure
    json!({
        "start_nonce": start_nonce,
        "payload": payload,
        "pub_key": public_key,
        "signature": signature
    })
}
